pub mod coach;
pub mod compare;
pub mod create_profile;
pub mod dashboard;
pub mod explain;
pub mod help;
pub mod import_url;
pub mod logs;
pub mod profile_view;
pub mod saved_profiles;
pub mod settings;

pub enum Screen {
    Dashboard,
    CreateProfile,
    ProfileView(String), // profile_id
    Explain(String),      // profile_id - shows reasoning behind profile
    ImportUrl(String),    // profile_id - import from social media URL
    Coach,
    Compare,
    SavedProfiles,
    Settings,
    Logs,
    Help,
}

impl Screen {
    pub fn as_str(&self) -> &str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::CreateProfile => "Create Profile",
            Screen::ProfileView(_) => "Profile View",
            Screen::Explain(_) => "Explain Profile",
            Screen::ImportUrl(_) => "Import from URL",
            Screen::Coach => "Coach",
            Screen::Compare => "Compare",
            Screen::SavedProfiles => "Saved Profiles",
            Screen::Settings => "Settings",
            Screen::Logs => "Logs",
            Screen::Help => "Help",
        }
    }
}
