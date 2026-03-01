use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DISC-style personality dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscStyle {
    Dominance,
    Influence,
    Steadiness,
    Conscientiousness,
}

impl DiscStyle {
    pub fn as_str(&self) -> &str {
        match self {
            DiscStyle::Dominance => "Dominance",
            DiscStyle::Influence => "Influence",
            DiscStyle::Steadiness => "Steadiness",
            DiscStyle::Conscientiousness => "Conscientiousness",
        }
    }
}

/// A complete personality profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonProfile {
    pub id: String,
    pub name: String,
    pub profile_type: ProfileType,
    pub trait_scores: TraitScores,
    pub communication_preferences: CommunicationPreferences,
    pub motivators: Motivators,
    pub stress_triggers: StressTriggers,
    pub strengths: Vec<String>,
    pub blind_spots: Vec<String>,
    pub confidence: f32,
    pub evidence: Vec<EvidenceSnippet>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]  // Backward compatibility with old profiles
    pub source_files: Vec<SourceFile>,  // Track source documents
    #[serde(default)]  // Backward compatibility with old profiles
    pub reasoning: Option<ProfileReasoning>,  // How the profile was determined
}

impl PersonProfile {
    pub fn new(name: String, profile_type: ProfileType) -> Self {
        let now = Utc::now();
        Self {
            id: Self::slugify(&name),
            name,
            profile_type,
            trait_scores: TraitScores::default(),
            communication_preferences: CommunicationPreferences::default(),
            motivators: Motivators::default(),
            stress_triggers: StressTriggers::default(),
            strengths: Vec::new(),
            blind_spots: Vec::new(),
            confidence: 0.0,
            evidence: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            source_files: Vec::new(),
            reasoning: None,
        }
    }
    
    /// Convert name to filesystem-safe slug
    pub fn slugify(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
    
    /// Get the folder path for this profile
    pub fn folder_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("./data/profiles").join(&self.id)
    }
    
    /// Get the sources folder path
    pub fn sources_path(&self) -> std::path::PathBuf {
        self.folder_path().join("sources")
    }
}

/// Type of profile creation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileType {
    SelfAssessment,
    TextInference,
    FileImport,
    ManualNotes,
}

/// Trait scores and dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitScores {
    pub primary_style: DiscStyle,
    pub secondary_style: Option<DiscStyle>,
    pub directness: f32,           // 0.0 - 1.0
    pub pace: f32,                 // 0.0 (slow) - 1.0 (fast)
    pub people_vs_task: f32,       // 0.0 (task) - 1.0 (people)
    pub detail_orientation: f32,   // 0.0 (big picture) - 1.0 (detail)
    pub risk_tolerance: f32,       // 0.0 (risk-averse) - 1.0 (risk-taking)
    pub formality: f32,            // 0.0 (casual) - 1.0 (formal)
}

impl Default for TraitScores {
    fn default() -> Self {
        Self {
            primary_style: DiscStyle::Steadiness,
            secondary_style: None,
            directness: 0.5,
            pace: 0.5,
            people_vs_task: 0.5,
            detail_orientation: 0.5,
            risk_tolerance: 0.5,
            formality: 0.5,
        }
    }
}

/// Communication preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPreferences {
    pub preferred_tone: Vec<String>,
    pub message_length: MessageLength,
    pub response_urgency: ResponseUrgency,
    pub meeting_style: Vec<String>,
    pub do_list: Vec<String>,
    pub dont_list: Vec<String>,
}

impl Default for CommunicationPreferences {
    fn default() -> Self {
        Self {
            preferred_tone: vec!["professional".to_string()],
            message_length: MessageLength::Medium,
            response_urgency: ResponseUrgency::Medium,
            meeting_style: vec!["structured".to_string()],
            do_list: Vec::new(),
            dont_list: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageLength {
    Brief,
    Medium,
    Detailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseUrgency {
    Low,
    Medium,
    High,
}

/// What motivates this person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Motivators {
    pub primary: Vec<String>,
    pub recognition_style: Vec<String>,
    pub work_environment: Vec<String>,
}

impl Default for Motivators {
    fn default() -> Self {
        Self {
            primary: Vec::new(),
            recognition_style: Vec::new(),
            work_environment: Vec::new(),
        }
    }
}

/// What causes stress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTriggers {
    pub situations: Vec<String>,
    pub communication_styles: Vec<String>,
    pub environmental: Vec<String>,
}

impl Default for StressTriggers {
    fn default() -> Self {
        Self {
            situations: Vec::new(),
            communication_styles: Vec::new(),
            environmental: Vec::new(),
        }
    }
}

/// Evidence supporting profile inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSnippet {
    pub text: String,
    pub category: EvidenceCategory,
    pub weight: f32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceCategory {
    Tone,
    Directness,
    DetailLevel,
    Urgency,
    SocialStyle,
    Formality,
    DecisionMaking,
    Other(String),
}

/// Analysis of a draft message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftAnalysis {
    pub id: String,
    pub draft_text: String,
    pub target_profile_id: String,
    pub overall_score: f32,
    pub subscores: DraftSubscores,
    pub risky_phrases: Vec<RiskyPhrase>,
    pub rewrites: Vec<RewriteVariant>,
    pub explanation: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftSubscores {
    pub clarity: f32,
    pub tone_fit: f32,
    pub directness_fit: f32,
    pub detail_fit: f32,
    pub warmth_fit: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskyPhrase {
    pub phrase: String,
    pub reason: String,
    pub suggestion: String,
}

/// A rewritten version of a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteVariant {
    pub style: RewriteStyle,
    pub text: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewriteStyle {
    Concise,
    Warm,
    Executive,
    Sales,
    ConflictSensitive,
    Direct,
}

impl RewriteStyle {
    pub fn as_str(&self) -> &str {
        match self {
            RewriteStyle::Concise => "Concise",
            RewriteStyle::Warm => "Warm",
            RewriteStyle::Executive => "Executive",
            RewriteStyle::Sales => "Sales",
            RewriteStyle::ConflictSensitive => "Conflict-Sensitive",
            RewriteStyle::Direct => "Direct",
        }
    }
}

/// Compatibility analysis between two profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub id: String,
    pub profile1_id: String,
    pub profile2_id: String,
    pub profile1_name: String,
    pub profile2_name: String,
    pub compatibility_score: f32,
    pub alignment_areas: Vec<AlignmentArea>,
    pub friction_points: Vec<FrictionPoint>,
    pub recommendations: Vec<String>,
    pub meeting_strategy: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentArea {
    pub dimension: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionPoint {
    pub dimension: String,
    pub description: String,
    pub mitigation: String,
}

/// Reasoning behind profile inference decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileReasoning {
    pub trait_explanations: Vec<TraitReasoning>,
    pub overall_summary: String,
    pub caveats: Vec<String>,
}

/// Explanation of how a specific trait value was determined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitReasoning {
    pub trait_name: String,
    pub value_chosen: String,
    pub reasoning: String,
    pub supporting_phrases: Vec<String>,
}

/// Source file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub last_modified: DateTime<Utc>,
    pub analyzed_at: Option<DateTime<Utc>>,
}

/// LLM request log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequestLog {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub request_type: String,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
    pub tokens_used: Option<u32>,
}
