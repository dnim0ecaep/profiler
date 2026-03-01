use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::models::{
    CompatibilityReport, DraftAnalysis, LlmRequestLog, PersonProfile,
};

#[derive(Clone)]
pub struct Storage {
    profiles_dir: PathBuf,
    logs_dir: PathBuf,
}

impl Storage {
    pub fn new(base_path: &Path) -> Result<Self> {
        let profiles_dir = base_path.join("profiles");
        let logs_dir = base_path.join("logs");
        
        // Create directories
        fs::create_dir_all(&profiles_dir)?;
        fs::create_dir_all(&logs_dir)?;
        
        Ok(Self {
            profiles_dir,
            logs_dir,
        })
    }
    
    pub fn initialize(&self) -> Result<()> {
        info!("Initializing file-based storage");
        // Directories already created in new()
        Ok(())
    }
    
    // Profile operations
    
    pub fn save_profile(&self, profile: &PersonProfile) -> Result<()> {
        let profile_dir = self.profiles_dir.join(&profile.id);
        fs::create_dir_all(&profile_dir)?;
        
        // Save profile metadata using serde_json to toml Value
        let profile_meta = serde_json::json!({
            "id": profile.id,
            "name": profile.name,
            "profile_type": format!("{:?}", profile.profile_type),
            "confidence": profile.confidence,
            "tags": profile.tags,
            "created_at": profile.created_at.to_rfc3339(),
            "updated_at": profile.updated_at.to_rfc3339(),
        });
        let profile_toml: toml::Value = serde_json::from_value(profile_meta)?;
        fs::write(
            profile_dir.join("profile.toml"),
            toml::to_string_pretty(&profile_toml)?
        )?;
        
        // Save traits
        let mut traits_map = serde_json::Map::new();
        traits_map.insert("primary_style".to_string(), serde_json::json!(profile.trait_scores.primary_style.as_str()));
        if let Some(ref secondary) = profile.trait_scores.secondary_style {
            traits_map.insert("secondary_style".to_string(), serde_json::json!(secondary.as_str()));
        }
        traits_map.insert("directness".to_string(), serde_json::json!(profile.trait_scores.directness));
        traits_map.insert("pace".to_string(), serde_json::json!(profile.trait_scores.pace));
        traits_map.insert("people_vs_task".to_string(), serde_json::json!(profile.trait_scores.people_vs_task));
        traits_map.insert("detail_orientation".to_string(), serde_json::json!(profile.trait_scores.detail_orientation));
        traits_map.insert("risk_tolerance".to_string(), serde_json::json!(profile.trait_scores.risk_tolerance));
        traits_map.insert("formality".to_string(), serde_json::json!(profile.trait_scores.formality));
        
        let traits = serde_json::Value::Object(traits_map);
        let traits_toml: toml::Value = serde_json::from_value(traits)?;
        fs::write(
            profile_dir.join("traits.toml"),
            toml::to_string_pretty(&traits_toml)?
        )?;
        
        // Save preferences
        let prefs = serde_json::json!({
            "preferred_tone": profile.communication_preferences.preferred_tone,
            "message_length": format!("{:?}", profile.communication_preferences.message_length),
            "response_urgency": format!("{:?}", profile.communication_preferences.response_urgency),
            "meeting_style": profile.communication_preferences.meeting_style,
            "do_list": profile.communication_preferences.do_list,
            "dont_list": profile.communication_preferences.dont_list,
        });
        let prefs_toml: toml::Value = serde_json::from_value(prefs)?;
        fs::write(
            profile_dir.join("preferences.toml"),
            toml::to_string_pretty(&prefs_toml)?
        )?;
        
        // Save motivators
        let motivators = serde_json::json!({
            "primary": profile.motivators.primary,
            "recognition_style": profile.motivators.recognition_style,
            "work_environment": profile.motivators.work_environment,
        });
        let motivators_toml: toml::Value = serde_json::from_value(motivators)?;
        fs::write(
            profile_dir.join("motivators.toml"),
            toml::to_string_pretty(&motivators_toml)?
        )?;
        
        // Save stress triggers
        let stress = serde_json::json!({
            "situations": profile.stress_triggers.situations,
            "communication_styles": profile.stress_triggers.communication_styles,
            "environmental": profile.stress_triggers.environmental,
        });
        let stress_toml: toml::Value = serde_json::from_value(stress)?;
        fs::write(
            profile_dir.join("stress.toml"),
            toml::to_string_pretty(&stress_toml)?
        )?;
        
        // Save analysis results
        let analysis = serde_json::json!({
            "strengths": profile.strengths,
            "blind_spots": profile.blind_spots,
        });
        let analysis_toml: toml::Value = serde_json::from_value(analysis)?;
        fs::write(
            profile_dir.join("analysis.toml"),
            toml::to_string_pretty(&analysis_toml)?
        )?;
        
        // Save evidence as JSON for complex structure
        if !profile.evidence.is_empty() {
            let evidence_json = serde_json::to_string_pretty(&profile.evidence)?;
            fs::write(profile_dir.join("evidence.json"), evidence_json)?;
        }
        
        // Save reasoning as JSON for complex structure
        if let Some(ref reasoning) = profile.reasoning {
            let reasoning_json = serde_json::to_string_pretty(reasoning)?;
            fs::write(profile_dir.join("reasoning.json"), reasoning_json)?;
        }
        
        // Create sources directory
        fs::create_dir_all(profile_dir.join("sources"))?;
        
        debug!("Saved profile: {} to {}", profile.id, profile_dir.display());
        Ok(())
    }
    
    pub fn get_profile(&self, id: &str) -> Result<Option<PersonProfile>> {
        let profile_dir = self.profiles_dir.join(id);
        if !profile_dir.exists() {
            return Ok(None);
        }
        
        let profile = self.load_profile_from_dir(&profile_dir)?;
        Ok(Some(profile))
    }
    
    fn load_profile_from_dir(&self, profile_dir: &Path) -> Result<PersonProfile> {
        // Load profile metadata
        let profile_toml = fs::read_to_string(profile_dir.join("profile.toml"))?;
        let profile_data: toml::Value = toml::from_str(&profile_toml)?;
        
        // Load traits
        let traits_toml = fs::read_to_string(profile_dir.join("traits.toml"))?;
        let traits_data: toml::Value = toml::from_str(&traits_toml)?;
        
        // Load preferences
        let prefs_toml = fs::read_to_string(profile_dir.join("preferences.toml"))?;
        let prefs_data: toml::Value = toml::from_str(&prefs_toml)?;
        
        // Load motivators
        let motivators_toml = fs::read_to_string(profile_dir.join("motivators.toml"))?;
        let motivators_data: toml::Value = toml::from_str(&motivators_toml)?;
        
        // Load stress triggers
        let stress_toml = fs::read_to_string(profile_dir.join("stress.toml"))?;
        let stress_data: toml::Value = toml::from_str(&stress_toml)?;
        
        // Load analysis
        let analysis_toml = fs::read_to_string(profile_dir.join("analysis.toml"))?;
        let analysis_data: toml::Value = toml::from_str(&analysis_toml)?;
        
        // Load evidence if exists
        let evidence: Vec<crate::models::EvidenceSnippet> = if profile_dir.join("evidence.json").exists() {
            let evidence_json = fs::read_to_string(profile_dir.join("evidence.json"))?;
            serde_json::from_str(&evidence_json)?
        } else {
            Vec::new()
        };
        
        // Load reasoning if exists
        let reasoning: Option<crate::models::ProfileReasoning> = if profile_dir.join("reasoning.json").exists() {
            let reasoning_json = fs::read_to_string(profile_dir.join("reasoning.json"))?;
            serde_json::from_str(&reasoning_json).ok()
        } else {
            None
        };
        
        // Reconstruct PersonProfile
        let mut profile_json = serde_json::json!({
            "id": profile_data["id"].as_str().unwrap(),
            "name": profile_data["name"].as_str().unwrap(),
            "profile_type": profile_data["profile_type"].as_str().unwrap(),
            "trait_scores": {
                "primary_style": traits_data["primary_style"].as_str().unwrap(),
                "secondary_style": traits_data.get("secondary_style").and_then(|v| v.as_str()),
                "directness": traits_data["directness"].as_float().unwrap(),
                "pace": traits_data["pace"].as_float().unwrap(),
                "people_vs_task": traits_data["people_vs_task"].as_float().unwrap(),
                "detail_orientation": traits_data["detail_orientation"].as_float().unwrap(),
                "risk_tolerance": traits_data["risk_tolerance"].as_float().unwrap(),
                "formality": traits_data["formality"].as_float().unwrap(),
            },
            "communication_preferences": {
                "preferred_tone": prefs_data["preferred_tone"].as_array().unwrap(),
                "message_length": prefs_data["message_length"].as_str().unwrap(),
                "response_urgency": prefs_data["response_urgency"].as_str().unwrap(),
                "meeting_style": prefs_data["meeting_style"].as_array().unwrap(),
                "do_list": prefs_data["do_list"].as_array().unwrap(),
                "dont_list": prefs_data["dont_list"].as_array().unwrap(),
            },
            "motivators": {
                "primary": motivators_data["primary"].as_array().unwrap(),
                "recognition_style": motivators_data["recognition_style"].as_array().unwrap(),
                "work_environment": motivators_data["work_environment"].as_array().unwrap(),
            },
            "stress_triggers": {
                "situations": stress_data["situations"].as_array().unwrap(),
                "communication_styles": stress_data["communication_styles"].as_array().unwrap(),
                "environmental": stress_data["environmental"].as_array().unwrap(),
            },
            "strengths": analysis_data["strengths"].as_array().unwrap(),
            "blind_spots": analysis_data["blind_spots"].as_array().unwrap(),
            "confidence": profile_data["confidence"].as_float().unwrap(),
            "evidence": evidence,
            "tags": profile_data["tags"].as_array().unwrap(),
            "created_at": profile_data["created_at"].as_str().unwrap(),
            "updated_at": profile_data["updated_at"].as_str().unwrap(),
            "source_files": [],
        });
        
        // Add reasoning if loaded
        if let Some(reasoning) = reasoning {
            if let Some(obj) = profile_json.as_object_mut() {
                obj.insert("reasoning".to_string(), serde_json::to_value(reasoning)?);
            }
        }
        
        let profile: PersonProfile = serde_json::from_value(profile_json)?;
        Ok(profile)
    }
    
    pub fn list_profiles(&self, limit: Option<usize>) -> Result<Vec<PersonProfile>> {
        let mut profiles = Vec::new();
        
        for entry in fs::read_dir(&self.profiles_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Ok(profile) = self.load_profile_from_dir(&entry.path()) {
                    profiles.push(profile);
                }
            }
        }
        
        // Sort by updated_at descending
        profiles.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        if let Some(n) = limit {
            profiles.truncate(n);
        }
        
        Ok(profiles)
    }
    
    pub fn search_profiles(&self, query: &str) -> Result<Vec<PersonProfile>> {
        let query_lower = query.to_lowercase();
        let all_profiles = self.list_profiles(None)?;
        
        let filtered: Vec<PersonProfile> = all_profiles
            .into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower) ||
                p.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect();
        
        Ok(filtered)
    }
    
    pub fn delete_profile(&self, id: &str) -> Result<()> {
        let profile_dir = self.profiles_dir.join(id);
        if profile_dir.exists() {
            fs::remove_dir_all(profile_dir)?;
            debug!("Deleted profile: {}", id);
        }
        Ok(())
    }
    
    /// List all source files in a profile's sources directory
    pub fn list_source_files(&self, profile_id: &str) -> Result<Vec<PathBuf>> {
        let sources_dir = self.profiles_dir.join(profile_id).join("sources");
        let mut files = Vec::new();
        
        if sources_dir.exists() {
            for entry in fs::read_dir(sources_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    files.push(entry.path());
                }
            }
        }
        
        Ok(files)
    }
    
    /// Save a source file to a profile's sources directory
    pub fn save_source_file(&self, profile_id: &str, filename: &str, content: &str) -> Result<()> {
        let sources_dir = self.profiles_dir.join(profile_id).join("sources");
        fs::create_dir_all(&sources_dir)?;
        
        let file_path = sources_dir.join(filename);
        fs::write(&file_path, content)?;
        
        info!("Saved source file: {} ({} chars)", file_path.display(), content.len());
        Ok(())
    }
    
    /// Read and concatenate all source files for a profile
    pub fn read_all_sources(&self, profile_id: &str) -> Result<String> {
        let files = self.list_source_files(profile_id)?;
        let mut combined = String::new();
        
        for file_path in &files {
            let filename = file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy();
            
            if let Ok(content) = fs::read_to_string(file_path) {
                if !combined.is_empty() {
                    combined.push_str("\n\n");
                }
                combined.push_str(&format!("--- Source: {} ---\n", filename));
                combined.push_str(&content);
            } else {
                debug!("Skipping non-text file: {}", filename);
            }
        }
        
        Ok(combined)
    }
    
    // Draft analysis operations
    
    pub fn save_draft_analysis(&self, analysis: &DraftAnalysis) -> Result<()> {
        let analyses_dir = self.logs_dir.join("draft-analyses");
        fs::create_dir_all(&analyses_dir)?;
        
        let analysis_json = serde_json::to_string_pretty(analysis)?;
        fs::write(analyses_dir.join(format!("{}.json", analysis.id)), analysis_json)?;
        
        debug!("Saved draft analysis: {}", analysis.id);
        Ok(())
    }
    
    pub fn get_draft_analyses(&self, profile_id: &str, limit: usize) -> Result<Vec<DraftAnalysis>> {
        let analyses_dir = self.logs_dir.join("draft-analyses");
        let mut analyses = Vec::new();
        
        if analyses_dir.exists() {
            for entry in fs::read_dir(analyses_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let content = fs::read_to_string(entry.path())?;
                    if let Ok(analysis) = serde_json::from_str::<DraftAnalysis>(&content) {
                        if analysis.target_profile_id == profile_id {
                            analyses.push(analysis);
                        }
                    }
                }
            }
        }
        
        analyses.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        analyses.truncate(limit);
        
        Ok(analyses)
    }
    
    // Compatibility report operations
    
    pub fn save_compatibility_report(&self, report: &CompatibilityReport) -> Result<()> {
        let reports_dir = self.logs_dir.join("compatibility-reports");
        fs::create_dir_all(&reports_dir)?;
        
        let report_json = serde_json::to_string_pretty(report)?;
        fs::write(reports_dir.join(format!("{}.json", report.id)), report_json)?;
        
        debug!("Saved compatibility report: {}", report.id);
        Ok(())
    }
    
    pub fn get_compatibility_report(&self, profile1_id: &str, profile2_id: &str) -> Result<Option<CompatibilityReport>> {
        let reports_dir = self.logs_dir.join("compatibility-reports");
        
        if !reports_dir.exists() {
            return Ok(None);
        }
        
        for entry in fs::read_dir(reports_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let content = fs::read_to_string(entry.path())?;
                if let Ok(report) = serde_json::from_str::<CompatibilityReport>(&content) {
                    if (report.profile1_id == profile1_id && report.profile2_id == profile2_id) ||
                       (report.profile1_id == profile2_id && report.profile2_id == profile1_id) {
                        return Ok(Some(report));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    // LLM request logging
    
    pub fn log_llm_request(&self, log: &LlmRequestLog) -> Result<()> {
        let logs_file = self.logs_dir.join("llm-requests.jsonl");
        
        let log_json = serde_json::to_string(log)?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logs_file)?;
        
        use std::io::Write;
        writeln!(file, "{}", log_json)?;
        
        Ok(())
    }
    
    pub fn get_recent_logs(&self, limit: usize) -> Result<Vec<LlmRequestLog>> {
        let logs_file = self.logs_dir.join("llm-requests.jsonl");
        
        if !logs_file.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(logs_file)?;
        let mut logs: Vec<LlmRequestLog> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        logs.truncate(limit);
        
        Ok(logs)
    }
}
