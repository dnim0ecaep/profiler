use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::models::*;
use crate::ollama::{Message, OllamaClient};
use crate::storage::Storage;

use super::fallback::FallbackAnalyzer;
use super::prompts::*;

#[derive(Clone)]
pub struct InferencePipeline {
    ollama: OllamaClient,
    chat_model: String,
    storage: Storage,
}

#[derive(Debug, Deserialize)]
struct ProfileInferenceResponse {
    primary_style: String,
    secondary_style: Option<String>,
    directness: f32,
    pace: f32,
    people_vs_task: f32,
    detail_orientation: f32,
    risk_tolerance: f32,
    formality: f32,
    preferred_tone: Vec<String>,
    motivators: Vec<String>,
    stress_triggers: Vec<String>,
    strengths: Vec<String>,
    blind_spots: Vec<String>,
    do_list: Vec<String>,
    dont_list: Vec<String>,
    confidence: f32,
    evidence: Vec<EvidenceResponse>,
    #[serde(default)]
    caveats: Vec<String>,
    #[serde(default)]
    reasoning: Option<ReasoningResponse>,
}

#[derive(Debug, Deserialize)]
struct EvidenceResponse {
    text: String,
    category: String,
}

#[derive(Debug, Deserialize)]
struct ReasoningResponse {
    #[serde(default)]
    overall_summary: String,
    #[serde(default)]
    trait_explanations: Vec<TraitReasoningResponse>,
}

#[derive(Debug, Deserialize)]
struct TraitReasoningResponse {
    trait_name: String,
    value_chosen: String,
    reasoning: String,
    #[serde(default)]
    supporting_phrases: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DraftAnalysisResponse {
    overall_score: f32,
    clarity: f32,
    tone_fit: f32,
    directness_fit: f32,
    detail_fit: f32,
    warmth_fit: f32,
    risky_phrases: Vec<RiskyPhraseResponse>,
    explanation: String,
}

#[derive(Debug, Deserialize)]
struct RiskyPhraseResponse {
    phrase: String,
    reason: String,
    suggestion: String,
}

#[derive(Debug, Deserialize)]
struct RewriteResponse {
    rewritten_text: String,
    explanation: String,
}

#[derive(Debug, Deserialize)]
struct CompatibilityResponse {
    compatibility_score: f32,
    alignment_areas: Vec<AlignmentResponse>,
    friction_points: Vec<FrictionResponse>,
    recommendations: Vec<String>,
    meeting_strategy: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AlignmentResponse {
    dimension: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct FrictionResponse {
    dimension: String,
    description: String,
    mitigation: String,
}

impl InferencePipeline {
    pub fn new(ollama: OllamaClient, chat_model: String, storage: Storage) -> Self {
        Self {
            ollama,
            chat_model,
            storage,
        }
    }
    
    pub async fn infer_profile_from_text(
        &self,
        name: String,
        text: String,
        profile_type: ProfileType,
    ) -> Result<PersonProfile> {
        debug!("Inferring profile from text for: {}", name);
        
        // Try LLM inference first
        match self.llm_infer_profile(&name, &text, profile_type.clone()).await {
            Ok(profile) => {
                debug!("Successfully inferred profile using LLM");
                Ok(profile)
            }
            Err(e) => {
                warn!("LLM inference failed, using fallback: {}", e);
                
                // Log the failure
                let _ = self.storage.log_llm_request(&LlmRequestLog {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    model: self.chat_model.clone(),
                    request_type: "profile_inference".to_string(),
                    duration_ms: 0,
                    success: false,
                    error: Some(e.to_string()),
                    tokens_used: None,
                });
                
                // Use rule-based fallback
                let mut profile = FallbackAnalyzer::analyze_text(&text, name);
                profile.profile_type = profile_type;
                Ok(profile)
            }
        }
    }
    
    async fn llm_infer_profile(
        &self,
        name: &str,
        text: &str,
        profile_type: ProfileType,
    ) -> Result<PersonProfile> {
        let messages = vec![
            Message::system(PROFILE_INFERENCE_SYSTEM),
            Message::user(profile_inference_prompt(text)),
        ];
        
        let start = std::time::Instant::now();
        let (response, duration) = self.ollama.chat(&self.chat_model, messages, true).await?;
        
        // Log the request
        let _ = self.storage.log_llm_request(&LlmRequestLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            model: self.chat_model.clone(),
            request_type: "profile_inference".to_string(),
            duration_ms: duration.as_millis() as u64,
            success: true,
            error: None,
            tokens_used: None,
        });
        
        // Parse JSON response
        let inference: ProfileInferenceResponse = serde_json::from_str(&response)
            .context("Failed to parse profile inference response")?;
        
        // Convert to PersonProfile
        let mut profile = PersonProfile::new(name.to_string(), profile_type);
        
        profile.trait_scores = TraitScores {
            primary_style: parse_disc_style(&inference.primary_style),
            secondary_style: inference.secondary_style.as_ref().map(|s| parse_disc_style(s)),
            directness: inference.directness.clamp(0.0, 1.0),
            pace: inference.pace.clamp(0.0, 1.0),
            people_vs_task: inference.people_vs_task.clamp(0.0, 1.0),
            detail_orientation: inference.detail_orientation.clamp(0.0, 1.0),
            risk_tolerance: inference.risk_tolerance.clamp(0.0, 1.0),
            formality: inference.formality.clamp(0.0, 1.0),
        };
        
        profile.communication_preferences.preferred_tone = inference.preferred_tone;
        profile.communication_preferences.do_list = inference.do_list;
        profile.communication_preferences.dont_list = inference.dont_list;
        
        profile.motivators.primary = inference.motivators;
        profile.stress_triggers.situations = inference.stress_triggers;
        
        profile.strengths = inference.strengths;
        profile.blind_spots = inference.blind_spots;
        profile.confidence = inference.confidence.clamp(0.0, 1.0);
        
        profile.evidence = inference
            .evidence
            .into_iter()
            .map(|e| EvidenceSnippet {
                text: e.text,
                category: parse_evidence_category(&e.category),
                weight: 0.5,
                source: "llm_inference".to_string(),
            })
            .collect();
        
        // Convert reasoning
        let caveats = inference.caveats;
        profile.reasoning = inference.reasoning.map(|r| ProfileReasoning {
            overall_summary: r.overall_summary,
            trait_explanations: r
                .trait_explanations
                .into_iter()
                .map(|t| TraitReasoning {
                    trait_name: t.trait_name,
                    value_chosen: t.value_chosen,
                    reasoning: t.reasoning,
                    supporting_phrases: t.supporting_phrases,
                })
                .collect(),
            caveats,
        });
        
        Ok(profile)
    }
    
    pub async fn analyze_draft(
        &self,
        draft: String,
        target_profile: &PersonProfile,
    ) -> Result<DraftAnalysis> {
        debug!("Analyzing draft for profile: {}", target_profile.id);
        
        let profile_summary = format!(
            "Style: {}\nDirectness: {:.1}\nPreferred Tone: {}\nDo: {}\nDon't: {}",
            target_profile.trait_scores.primary_style.as_str(),
            target_profile.trait_scores.directness,
            target_profile.communication_preferences.preferred_tone.join(", "),
            target_profile.communication_preferences.do_list.join("; "),
            target_profile.communication_preferences.dont_list.join("; "),
        );
        
        let messages = vec![
            Message::system(DRAFT_ANALYSIS_SYSTEM),
            Message::user(draft_analysis_prompt(&draft, &profile_summary)),
        ];
        
        let (response, duration) = self.ollama.chat(&self.chat_model, messages, true).await?;
        
        // Log the request
        let _ = self.storage.log_llm_request(&LlmRequestLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            model: self.chat_model.clone(),
            request_type: "draft_analysis".to_string(),
            duration_ms: duration.as_millis() as u64,
            success: true,
            error: None,
            tokens_used: None,
        });
        
        let analysis_resp: DraftAnalysisResponse = serde_json::from_str(&response)
            .context("Failed to parse draft analysis response")?;
        
        let analysis = DraftAnalysis {
            id: Uuid::new_v4().to_string(),
            draft_text: draft.clone(),
            target_profile_id: target_profile.id.clone(),
            overall_score: analysis_resp.overall_score.clamp(0.0, 1.0),
            subscores: DraftSubscores {
                clarity: analysis_resp.clarity.clamp(0.0, 1.0),
                tone_fit: analysis_resp.tone_fit.clamp(0.0, 1.0),
                directness_fit: analysis_resp.directness_fit.clamp(0.0, 1.0),
                detail_fit: analysis_resp.detail_fit.clamp(0.0, 1.0),
                warmth_fit: analysis_resp.warmth_fit.clamp(0.0, 1.0),
            },
            risky_phrases: analysis_resp
                .risky_phrases
                .into_iter()
                .map(|r| RiskyPhrase {
                    phrase: r.phrase,
                    reason: r.reason,
                    suggestion: r.suggestion,
                })
                .collect(),
            rewrites: Vec::new(),
            explanation: analysis_resp.explanation,
            created_at: Utc::now(),
        };
        
        Ok(analysis)
    }
    
    pub async fn rewrite_draft(
        &self,
        draft: &str,
        style: RewriteStyle,
        target_profile: &PersonProfile,
    ) -> Result<RewriteVariant> {
        debug!("Rewriting draft in style: {:?}", style);
        
        let profile_summary = format!(
            "Style: {}\nDirectness: {:.1}\nPreferred Tone: {}",
            target_profile.trait_scores.primary_style.as_str(),
            target_profile.trait_scores.directness,
            target_profile.communication_preferences.preferred_tone.join(", "),
        );
        
        let messages = vec![
            Message::system(REWRITE_SYSTEM),
            Message::user(rewrite_prompt(draft, style.as_str(), &profile_summary)),
        ];
        
        let (response, _) = self.ollama.chat(&self.chat_model, messages, true).await?;
        
        let rewrite_resp: RewriteResponse = serde_json::from_str(&response)
            .context("Failed to parse rewrite response")?;
        
        Ok(RewriteVariant {
            style,
            text: rewrite_resp.rewritten_text,
            explanation: rewrite_resp.explanation,
        })
    }
    
    pub async fn compare_profiles(
        &self,
        profile1: &PersonProfile,
        profile2: &PersonProfile,
    ) -> Result<CompatibilityReport> {
        debug!("Comparing profiles: {} and {}", profile1.id, profile2.id);
        
        // Try LLM comparison first, fall back to algorithmic comparison
        match self.llm_compare_profiles(profile1, profile2).await {
            Ok(report) => {
                debug!("Successfully compared profiles using LLM");
                Ok(report)
            }
            Err(e) => {
                warn!("LLM comparison failed, using fallback: {}", e);
                
                // Log the failure
                let _ = self.storage.log_llm_request(&LlmRequestLog {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    model: self.chat_model.clone(),
                    request_type: "profile_comparison".to_string(),
                    duration_ms: 0,
                    success: false,
                    error: Some(e.to_string()),
                    tokens_used: None,
                });
                
                // Use rule-based fallback comparison
                Ok(FallbackAnalyzer::compare_profiles(profile1, profile2))
            }
        }
    }
    
    async fn llm_compare_profiles(
        &self,
        profile1: &PersonProfile,
        profile2: &PersonProfile,
    ) -> Result<CompatibilityReport> {
        let summary1 = self.summarize_profile(profile1);
        let summary2 = self.summarize_profile(profile2);
        
        let messages = vec![
            Message::system(COMPATIBILITY_SYSTEM),
            Message::user(compatibility_prompt(&summary1, &summary2)),
        ];
        
        let (response, duration) = self.ollama.chat(&self.chat_model, messages, true).await?;
        
        // Log the request
        let _ = self.storage.log_llm_request(&LlmRequestLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            model: self.chat_model.clone(),
            request_type: "profile_comparison".to_string(),
            duration_ms: duration.as_millis() as u64,
            success: true,
            error: None,
            tokens_used: None,
        });
        
        let compat_resp: CompatibilityResponse = serde_json::from_str(&response)
            .context("Failed to parse compatibility response")?;
        
        Ok(CompatibilityReport {
            id: Uuid::new_v4().to_string(),
            profile1_id: profile1.id.clone(),
            profile2_id: profile2.id.clone(),
            profile1_name: profile1.name.clone(),
            profile2_name: profile2.name.clone(),
            compatibility_score: compat_resp.compatibility_score.clamp(0.0, 1.0),
            alignment_areas: compat_resp
                .alignment_areas
                .into_iter()
                .map(|a| AlignmentArea {
                    dimension: a.dimension,
                    description: a.description,
                })
                .collect(),
            friction_points: compat_resp
                .friction_points
                .into_iter()
                .map(|f| FrictionPoint {
                    dimension: f.dimension,
                    description: f.description,
                    mitigation: f.mitigation,
                })
                .collect(),
            recommendations: compat_resp.recommendations,
            meeting_strategy: compat_resp.meeting_strategy,
            created_at: Utc::now(),
        })
    }
    
    /// Reanalyze a profile from source text, preserving identity fields
    pub async fn reanalyze_profile(
        &self,
        existing: &PersonProfile,
        source_text: String,
    ) -> Result<PersonProfile> {
        debug!("Reanalyzing profile: {}", existing.id);
        
        // Run inference on the new text
        let mut updated = self.infer_profile_from_text(
            existing.name.clone(),
            source_text,
            existing.profile_type.clone(),
        ).await?;
        
        // Preserve identity fields from the original profile
        updated.id = existing.id.clone();
        updated.tags = existing.tags.clone();
        updated.created_at = existing.created_at;
        updated.source_files = existing.source_files.clone();
        updated.updated_at = Utc::now();
        
        Ok(updated)
    }
    
    fn summarize_profile(&self, profile: &PersonProfile) -> String {
        format!(
            "Name: {}\nStyle: {}\nDirectness: {:.1}\nPace: {:.1}\nFormality: {:.1}\nStrengths: {}\nMotivators: {}",
            profile.name,
            profile.trait_scores.primary_style.as_str(),
            profile.trait_scores.directness,
            profile.trait_scores.pace,
            profile.trait_scores.formality,
            profile.strengths.join(", "),
            profile.motivators.primary.join(", "),
        )
    }
}

fn parse_disc_style(s: &str) -> DiscStyle {
    match s.to_lowercase().as_str() {
        "dominance" | "d" => DiscStyle::Dominance,
        "influence" | "i" => DiscStyle::Influence,
        "steadiness" | "s" => DiscStyle::Steadiness,
        "conscientiousness" | "c" => DiscStyle::Conscientiousness,
        _ => DiscStyle::Steadiness,
    }
}

fn parse_evidence_category(s: &str) -> EvidenceCategory {
    match s.to_lowercase().as_str() {
        "tone" => EvidenceCategory::Tone,
        "directness" => EvidenceCategory::Directness,
        "detail" | "detail_level" => EvidenceCategory::DetailLevel,
        "urgency" => EvidenceCategory::Urgency,
        "social" | "social_style" => EvidenceCategory::SocialStyle,
        "formality" => EvidenceCategory::Formality,
        "decision" | "decision_making" => EvidenceCategory::DecisionMaking,
        other => EvidenceCategory::Other(other.to_string()),
    }
}
