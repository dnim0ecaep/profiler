use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

use chrono::Utc;
use uuid::Uuid;

use crate::models::{
    AlignmentArea, CompatibilityReport, CommunicationPreferences, DiscStyle, EvidenceCategory,
    EvidenceSnippet, FrictionPoint, MessageLength, Motivators, PersonProfile, ProfileReasoning,
    ProfileType, ResponseUrgency, StressTriggers, TraitReasoning, TraitScores,
};

/// Rule-based fallback analyzer when LLM is unavailable
pub struct FallbackAnalyzer;

impl FallbackAnalyzer {
    pub fn analyze_text(text: &str, name: String) -> PersonProfile {
        let mut profile = PersonProfile::new(name, ProfileType::TextInference);
        
        // Analyze various text characteristics
        let metrics = TextMetrics::analyze(text);
        
        // Infer DISC style
        profile.trait_scores = Self::infer_traits(&metrics);
        
        // Infer communication preferences
        profile.communication_preferences = Self::infer_communication_prefs(&metrics);
        
        // Add motivators and stress triggers
        profile.motivators = Self::infer_motivators(&metrics);
        profile.stress_triggers = Self::infer_stress_triggers(&metrics);
        
        // Add strengths and blind spots
        profile.strengths = Self::infer_strengths(&profile.trait_scores);
        profile.blind_spots = Self::infer_blind_spots(&profile.trait_scores);
        
        // Generate evidence
        profile.evidence = Self::generate_evidence(text, &metrics);
        
        // Generate reasoning
        profile.reasoning = Some(Self::generate_reasoning(text, &metrics, &profile.trait_scores));
        
        // Set confidence (lower for fallback)
        profile.confidence = 0.4;
        
        profile
    }
    
    fn infer_traits(metrics: &TextMetrics) -> TraitScores {
        let mut traits = TraitScores::default();
        
        // Directness based on sentence structure
        traits.directness = if metrics.avg_sentence_length < 15.0 && metrics.imperative_ratio > 0.1 {
            0.75
        } else if metrics.hedging_ratio > 0.15 {
            0.3
        } else {
            0.5
        };
        
        // Pace based on urgency markers
        traits.pace = if metrics.urgency_ratio > 0.1 {
            0.75
        } else if metrics.avg_sentence_length > 25.0 {
            0.3
        } else {
            0.5
        };
        
        // People vs task orientation
        traits.people_vs_task = if metrics.emotional_language_ratio > 0.1 {
            0.7
        } else if metrics.technical_ratio > 0.15 {
            0.3
        } else {
            0.5
        };
        
        // Detail orientation
        traits.detail_orientation = if metrics.detail_markers > 5 {
            0.75
        } else if metrics.avg_sentence_length < 12.0 {
            0.3
        } else {
            0.5
        };
        
        // Risk tolerance
        traits.risk_tolerance = if metrics.cautious_markers > 3 {
            0.3
        } else if metrics.action_verbs_ratio > 0.15 {
            0.7
        } else {
            0.5
        };
        
        // Formality
        traits.formality = if metrics.formal_markers > 2 {
            0.7
        } else if metrics.casual_markers > 2 {
            0.3
        } else {
            0.5
        };
        
        // Primary DISC style
        traits.primary_style = Self::determine_disc_style(&traits);
        
        traits
    }
    
    fn determine_disc_style(traits: &TraitScores) -> DiscStyle {
        // Simple heuristic based on trait combinations
        if traits.directness > 0.6 && traits.pace > 0.6 {
            DiscStyle::Dominance
        } else if traits.people_vs_task > 0.6 && traits.pace > 0.6 {
            DiscStyle::Influence
        } else if traits.people_vs_task > 0.6 && traits.pace < 0.5 {
            DiscStyle::Steadiness
        } else {
            DiscStyle::Conscientiousness
        }
    }
    
    fn infer_communication_prefs(metrics: &TextMetrics) -> CommunicationPreferences {
        let mut prefs = CommunicationPreferences::default();
        
        prefs.message_length = if metrics.avg_sentence_length < 12.0 {
            MessageLength::Brief
        } else if metrics.avg_sentence_length > 25.0 {
            MessageLength::Detailed
        } else {
            MessageLength::Medium
        };
        
        prefs.response_urgency = if metrics.urgency_ratio > 0.1 {
            ResponseUrgency::High
        } else if metrics.leisurely_markers > 2 {
            ResponseUrgency::Low
        } else {
            ResponseUrgency::Medium
        };
        
        prefs.preferred_tone = vec!["professional".to_string()];
        prefs.do_list = vec!["Be clear and concise".to_string()];
        prefs.dont_list = vec!["Avoid ambiguity".to_string()];
        
        prefs
    }
    
    fn infer_motivators(metrics: &TextMetrics) -> Motivators {
        let mut motivators = Motivators::default();
        
        if metrics.achievement_markers > 2 {
            motivators.primary.push("Achievement".to_string());
        }
        if metrics.emotional_language_ratio > 0.1 {
            motivators.primary.push("Relationships".to_string());
        }
        if metrics.detail_markers > 5 {
            motivators.primary.push("Accuracy".to_string());
        }
        
        motivators
    }
    
    fn infer_stress_triggers(metrics: &TextMetrics) -> StressTriggers {
        let mut triggers = StressTriggers::default();
        
        if metrics.urgency_ratio > 0.1 {
            triggers.situations.push("Delays".to_string());
        }
        if metrics.detail_markers > 5 {
            triggers.situations.push("Lack of information".to_string());
        }
        
        triggers
    }
    
    fn infer_strengths(traits: &TraitScores) -> Vec<String> {
        let mut strengths = Vec::new();
        
        if traits.directness > 0.6 {
            strengths.push("Direct communication".to_string());
        }
        if traits.detail_orientation > 0.6 {
            strengths.push("Attention to detail".to_string());
        }
        if traits.people_vs_task > 0.6 {
            strengths.push("Relationship building".to_string());
        }
        
        strengths
    }
    
    fn infer_blind_spots(traits: &TraitScores) -> Vec<String> {
        let mut blind_spots = Vec::new();
        
        if traits.directness > 0.7 {
            blind_spots.push("May come across as blunt".to_string());
        }
        if traits.detail_orientation < 0.4 {
            blind_spots.push("May overlook details".to_string());
        }
        
        blind_spots
    }
    
    fn generate_evidence(text: &str, metrics: &TextMetrics) -> Vec<EvidenceSnippet> {
        let mut evidence = Vec::new();
        
        evidence.push(EvidenceSnippet {
            text: format!("Average sentence length: {:.1} words", metrics.avg_sentence_length),
            category: EvidenceCategory::Other("Structure".to_string()),
            weight: 0.5,
            source: "fallback_analysis".to_string(),
        });
        
        if metrics.urgency_ratio > 0.1 {
            evidence.push(EvidenceSnippet {
                text: "Text contains urgency markers".to_string(),
                category: EvidenceCategory::Urgency,
                weight: 0.6,
                source: "fallback_analysis".to_string(),
            });
        }
        
        evidence
    }
    
    /// Fallback comparison when LLM is unavailable — uses trait score differences
    pub fn compare_profiles(p1: &PersonProfile, p2: &PersonProfile) -> CompatibilityReport {
        let t1 = &p1.trait_scores;
        let t2 = &p2.trait_scores;

        // Calculate trait differences
        let diffs = [
            ("Directness", t1.directness, t2.directness),
            ("Pace", t1.pace, t2.pace),
            ("People vs Task", t1.people_vs_task, t2.people_vs_task),
            ("Detail Orientation", t1.detail_orientation, t2.detail_orientation),
            ("Risk Tolerance", t1.risk_tolerance, t2.risk_tolerance),
            ("Formality", t1.formality, t2.formality),
        ];

        let mut alignment_areas = Vec::new();
        let mut friction_points = Vec::new();

        for (name, v1, v2) in &diffs {
            let diff = (v1 - v2).abs();
            if diff < 0.2 {
                alignment_areas.push(AlignmentArea {
                    dimension: name.to_string(),
                    description: format!(
                        "Both {} and {} are closely aligned on {} ({:.0}% vs {:.0}%)",
                        p1.name, p2.name, name, v1 * 100.0, v2 * 100.0
                    ),
                });
            } else if diff > 0.35 {
                let mitigation = match *name {
                    "Directness" => "Be explicit about expectations — one prefers direct communication while the other is more indirect.",
                    "Pace" => "Agree on timelines upfront — one works faster while the other prefers a measured pace.",
                    "People vs Task" => "Balance meetings with both relationship check-ins and task progress updates.",
                    "Detail Orientation" => "Provide summaries with optional detail — one wants the big picture, the other wants specifics.",
                    "Risk Tolerance" => "Discuss risk/reward trade-offs explicitly when making decisions together.",
                    "Formality" => "Mirror each other's communication style — one is more formal, the other more casual.",
                    _ => "Acknowledge the difference and find a middle ground.",
                };
                friction_points.push(FrictionPoint {
                    dimension: name.to_string(),
                    description: format!(
                        "{} and {} differ significantly on {} ({:.0}% vs {:.0}%)",
                        p1.name, p2.name, name, v1 * 100.0, v2 * 100.0
                    ),
                    mitigation: mitigation.to_string(),
                });
            }
        }

        // DISC style compatibility
        let disc_compat = match (&t1.primary_style, &t2.primary_style) {
            (DiscStyle::Dominance, DiscStyle::Influence) | (DiscStyle::Influence, DiscStyle::Dominance) => 0.7,
            (DiscStyle::Steadiness, DiscStyle::Conscientiousness) | (DiscStyle::Conscientiousness, DiscStyle::Steadiness) => 0.7,
            (DiscStyle::Dominance, DiscStyle::Steadiness) | (DiscStyle::Steadiness, DiscStyle::Dominance) => 0.5,
            (DiscStyle::Influence, DiscStyle::Conscientiousness) | (DiscStyle::Conscientiousness, DiscStyle::Influence) => 0.5,
            _ if std::mem::discriminant(&t1.primary_style) == std::mem::discriminant(&t2.primary_style) => 0.85,
            _ => 0.6,
        };

        // Overall score: weighted average of DISC compatibility and trait alignment
        let avg_diff: f32 = diffs.iter().map(|(_, v1, v2)| (v1 - v2).abs()).sum::<f32>() / diffs.len() as f32;
        let trait_compat = 1.0 - avg_diff;
        let compatibility_score = (disc_compat * 0.4 + trait_compat * 0.6).clamp(0.0, 1.0);

        let recommendations = vec![
            format!(
                "{} ({}) and {} ({}) — {}",
                p1.name, t1.primary_style.as_str(),
                p2.name, t2.primary_style.as_str(),
                if compatibility_score > 0.7 { "naturally complementary styles" }
                else if compatibility_score > 0.5 { "workable with some adaptation" }
                else { "will need deliberate effort to align" }
            ),
            "Set clear expectations early in any collaboration".to_string(),
            "Check in periodically on communication satisfaction".to_string(),
        ];

        let meeting_strategy = vec![
            "Start with a brief personal check-in".to_string(),
            "Use a shared agenda to keep both parties aligned".to_string(),
            "End with clear action items and owners".to_string(),
        ];

        CompatibilityReport {
            id: Uuid::new_v4().to_string(),
            profile1_id: p1.id.clone(),
            profile2_id: p2.id.clone(),
            profile1_name: p1.name.clone(),
            profile2_name: p2.name.clone(),
            compatibility_score,
            alignment_areas,
            friction_points,
            recommendations,
            meeting_strategy,
            created_at: Utc::now(),
        }
    }

    fn generate_reasoning(text: &str, metrics: &TextMetrics, traits: &TraitScores) -> ProfileReasoning {
        let mut explanations = Vec::new();
        let text_lower = text.to_lowercase();
        
        // Helper: find matching phrases in text for a word list
        let find_phrases = |words: &[&str]| -> Vec<String> {
            let sentences: Vec<&str> = text.split(|c: char| c == '.' || c == '!' || c == '?')
                .filter(|s| !s.trim().is_empty())
                .collect();
            let mut found = Vec::new();
            for word in words {
                for sentence in &sentences {
                    if sentence.to_lowercase().contains(word) {
                        let trimmed = sentence.trim().to_string();
                        if !found.contains(&trimmed) {
                            found.push(trimmed);
                        }
                    }
                }
            }
            found.truncate(3); // Limit to 3 supporting phrases per trait
            found
        };
        
        // Primary DISC Style reasoning
        let disc_reasoning = match traits.primary_style {
            DiscStyle::Dominance => "High directness combined with fast pace suggests a Dominance style — focused on results and action.",
            DiscStyle::Influence => "Strong people orientation combined with fast pace suggests an Influence style — energetic and relationship-driven.",
            DiscStyle::Steadiness => "People-oriented with a measured pace suggests a Steadiness style — values harmony and consistency.",
            DiscStyle::Conscientiousness => "Balanced or task-oriented traits with measured pace suggest a Conscientiousness style — values accuracy and quality.",
        };
        explanations.push(TraitReasoning {
            trait_name: "Primary DISC Style".to_string(),
            value_chosen: traits.primary_style.as_str().to_string(),
            reasoning: disc_reasoning.to_string(),
            supporting_phrases: Vec::new(),
        });
        
        // Directness
        let (dir_reason, dir_words): (&str, &[&str]) = if metrics.avg_sentence_length < 15.0 && metrics.imperative_ratio > 0.1 {
            ("Short sentences and imperative commands indicate a very direct communication style.", &["do", "make", "create", "build", "execute", "implement"])
        } else if metrics.hedging_ratio > 0.15 {
            ("Frequent hedging language (maybe, perhaps, might) suggests an indirect, cautious style.", &["maybe", "perhaps", "might", "could", "possibly", "probably"])
        } else {
            ("Sentence length and language patterns suggest a moderate level of directness.", &[])
        };
        explanations.push(TraitReasoning {
            trait_name: "Directness".to_string(),
            value_chosen: format!("{:.2}", traits.directness),
            reasoning: dir_reason.to_string(),
            supporting_phrases: find_phrases(dir_words),
        });
        
        // Pace
        let (pace_reason, pace_words): (&str, &[&str]) = if metrics.urgency_ratio > 0.1 {
            ("Urgency markers (asap, immediately, quickly) indicate a fast-paced communicator.", &["urgent", "asap", "immediately", "quickly", "now", "rush"])
        } else if metrics.avg_sentence_length > 25.0 {
            ("Long, detailed sentences suggest a deliberate, slower-paced communicator.", &[])
        } else {
            ("No strong urgency or deliberation signals — moderate pace inferred.", &[])
        };
        explanations.push(TraitReasoning {
            trait_name: "Pace".to_string(),
            value_chosen: format!("{:.2}", traits.pace),
            reasoning: pace_reason.to_string(),
            supporting_phrases: find_phrases(pace_words),
        });
        
        // People vs Task
        let (pvt_reason, pvt_words): (&str, &[&str]) = if metrics.emotional_language_ratio > 0.1 {
            ("Emotional language (feel, love, excited) indicates a people-oriented focus.", &["feel", "love", "hate", "excited", "worried", "happy"])
        } else if metrics.technical_ratio > 0.15 {
            ("Technical terminology (algorithm, implementation, system) suggests a task-oriented focus.", &["algorithm", "implementation", "function", "data", "system"])
        } else {
            ("Balanced use of emotional and technical language — moderate people/task orientation.", &[])
        };
        explanations.push(TraitReasoning {
            trait_name: "People vs Task".to_string(),
            value_chosen: format!("{:.2}", traits.people_vs_task),
            reasoning: pvt_reason.to_string(),
            supporting_phrases: find_phrases(pvt_words),
        });
        
        // Detail Orientation
        let (det_reason, det_words): (&str, &[&str]) = if metrics.detail_markers > 5 {
            ("Frequent use of numbers and specific data points shows high detail orientation.", &[])
        } else if metrics.avg_sentence_length < 12.0 {
            ("Very short sentences suggest a big-picture communicator who avoids excessive detail.", &[])
        } else {
            ("Moderate sentence length and detail markers suggest balanced detail orientation.", &[])
        };
        explanations.push(TraitReasoning {
            trait_name: "Detail Orientation".to_string(),
            value_chosen: format!("{:.2}", traits.detail_orientation),
            reasoning: det_reason.to_string(),
            supporting_phrases: find_phrases(det_words),
        });
        
        // Risk Tolerance
        let (risk_reason, risk_words): (&str, &[&str]) = if metrics.cautious_markers > 3 {
            ("Cautious language (careful, ensure, verify) indicates low risk tolerance.", &["careful", "ensure", "verify", "check", "confirm"])
        } else if metrics.action_verbs_ratio > 0.15 {
            ("Frequent action verbs (do, make, build) suggest a bias toward action and higher risk tolerance.", &["do", "make", "create", "build", "execute", "implement"])
        } else {
            ("No strong cautious or action-biased language — moderate risk tolerance inferred.", &[])
        };
        explanations.push(TraitReasoning {
            trait_name: "Risk Tolerance".to_string(),
            value_chosen: format!("{:.2}", traits.risk_tolerance),
            reasoning: risk_reason.to_string(),
            supporting_phrases: find_phrases(risk_words),
        });
        
        // Formality
        let (form_reason, form_words): (&str, &[&str]) = if metrics.formal_markers > 2 {
            ("Formal connectors (furthermore, therefore, consequently) indicate a formal communication style.", &["furthermore", "therefore", "consequently"])
        } else if metrics.casual_markers > 2 {
            ("Casual language (yeah, ok, cool) indicates an informal communication style.", &["yeah", "ok", "cool", "awesome"])
        } else {
            ("Language formality is moderate — neither highly formal nor casual.", &[])
        };
        explanations.push(TraitReasoning {
            trait_name: "Formality".to_string(),
            value_chosen: format!("{:.2}", traits.formality),
            reasoning: form_reason.to_string(),
            supporting_phrases: find_phrases(form_words),
        });
        
        let overall_summary = format!(
            "Profile generated using rule-based text analysis (fallback mode). \
            Analyzed {} words across {} sentences. Key signals: avg sentence length {:.1} words, \
            hedging ratio {:.1}%, urgency ratio {:.1}%.",
            metrics.word_count,
            metrics.sentence_count,
            metrics.avg_sentence_length,
            metrics.hedging_ratio * 100.0,
            metrics.urgency_ratio * 100.0,
        );
        
        ProfileReasoning {
            trait_explanations: explanations,
            overall_summary,
            caveats: vec![
                "Generated using rule-based heuristics (LLM was unavailable)".to_string(),
                "Confidence is lower than LLM-based analysis".to_string(),
                format!("Based on {} words — more text improves accuracy", metrics.word_count),
            ],
        }
    }
}

struct TextMetrics {
    word_count: usize,
    sentence_count: usize,
    avg_sentence_length: f32,
    hedging_ratio: f32,
    imperative_ratio: f32,
    urgency_ratio: f32,
    emotional_language_ratio: f32,
    technical_ratio: f32,
    action_verbs_ratio: f32,
    detail_markers: usize,
    cautious_markers: usize,
    formal_markers: usize,
    casual_markers: usize,
    achievement_markers: usize,
    leisurely_markers: usize,
}

impl TextMetrics {
    fn analyze(text: &str) -> Self {
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        
        let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?').collect();
        let sentence_count = sentences.iter().filter(|s| !s.trim().is_empty()).count().max(1);
        
        let avg_sentence_length = word_count as f32 / sentence_count as f32;
        
        // Pattern matching
        let hedging_words = vec!["maybe", "perhaps", "might", "could", "possibly", "probably"];
        let urgency_words = vec!["urgent", "asap", "immediately", "quickly", "now", "rush"];
        let emotional_words = vec!["feel", "love", "hate", "excited", "worried", "happy"];
        let technical_words = vec!["algorithm", "implementation", "function", "data", "system"];
        let action_verbs = vec!["do", "make", "create", "build", "execute", "implement"];
        let cautious_words = vec!["careful", "ensure", "verify", "check", "confirm"];
        let formal_words = vec!["furthermore", "therefore", "consequently"];
        let casual_words = vec!["yeah", "ok", "cool", "awesome"];
        let achievement_words = vec!["achieve", "goal", "success", "win", "accomplish"];
        
        let text_lower = text.to_lowercase();
        
        let hedging_ratio = Self::count_words(&text_lower, &hedging_words) as f32 / word_count.max(1) as f32;
        let urgency_ratio = Self::count_words(&text_lower, &urgency_words) as f32 / word_count.max(1) as f32;
        let emotional_language_ratio = Self::count_words(&text_lower, &emotional_words) as f32 / word_count.max(1) as f32;
        let technical_ratio = Self::count_words(&text_lower, &technical_words) as f32 / word_count.max(1) as f32;
        let action_verbs_ratio = Self::count_words(&text_lower, &action_verbs) as f32 / word_count.max(1) as f32;
        
        let cautious_markers = Self::count_words(&text_lower, &cautious_words);
        let formal_markers = Self::count_words(&text_lower, &formal_words);
        let casual_markers = Self::count_words(&text_lower, &casual_words);
        let achievement_markers = Self::count_words(&text_lower, &achievement_words);
        
        // Count sentences starting with imperative verbs
        let imperative_count = sentences
            .iter()
            .filter(|s| {
                let first_word = s.trim().split_whitespace().next().unwrap_or("");
                action_verbs.contains(&first_word.to_lowercase().as_str())
            })
            .count();
        let imperative_ratio = imperative_count as f32 / sentence_count as f32;
        
        // Count detail markers (numbers, specific terms)
        let detail_markers = text.matches(char::is_numeric).count() / 5;
        
        let leisurely_markers = Self::count_words(&text_lower, &vec!["relaxed", "casual", "flexible"]);
        
        Self {
            word_count,
            sentence_count,
            avg_sentence_length,
            hedging_ratio,
            imperative_ratio,
            urgency_ratio,
            emotional_language_ratio,
            technical_ratio,
            action_verbs_ratio,
            detail_markers,
            cautious_markers,
            formal_markers,
            casual_markers,
            achievement_markers,
            leisurely_markers,
        }
    }
    
    fn count_words(text: &str, words: &[&str]) -> usize {
        words.iter().filter(|w| text.contains(*w)).count()
    }
}
