use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::config::Config;
use crate::inference::InferencePipeline;
use crate::models::{PersonProfile, ProfileType};
use crate::ollama::OllamaClient;
use crate::scraper;
use crate::storage::Storage;
use crate::ui::components::LoadingState;
use crate::ui::screens::{
    coach::CoachState,
    create_profile::{CreateMode, CreateProfileState},
    import_url::{ImportUrlMode, ImportUrlState},
    settings::SettingsState,
    Screen,
};
use crate::ui::terminal::AppTerminal;
use crate::ui::ColorPalette;

/// Results from background tasks
enum TaskResult {
    ProfileCreated(Result<PersonProfile>),
    ProfileReanalyzed(Result<PersonProfile>),
    DraftAnalyzed(Result<crate::models::DraftAnalysis>),
    ComparisonDone(Result<crate::models::CompatibilityReport>),
    UrlScraped {
        result: Result<String>,
        profile_id: String,
        platform: String,
    },
}

pub struct App {
    config: Config,
    storage: Storage,
    ollama: OllamaClient,
    pipeline: InferencePipeline,
    current_screen: Screen,
    should_quit: bool,
    ollama_connected: bool,
    palette: ColorPalette,
    
    previous_screen: Option<Box<Screen>>, // For help screen return
    // Screen states
    recent_profiles: Vec<PersonProfile>,
    all_profiles: Vec<PersonProfile>,
    create_profile_state: CreateProfileState,
    coach_state: CoachState,
    settings_state: SettingsState,
    selected_profile_id: Option<String>,
    logs: Vec<crate::models::LlmRequestLog>,
    dashboard_selected: usize,
    create_profile_selected: usize,
    saved_profiles_selected: usize,
    coach_profile_selected: usize,
    // Compare state
    compare_profile1_id: Option<String>,
    compare_profile2_id: Option<String>,
    compare_selecting: usize, // 0 = selecting first, 1 = selecting second
    compare_selected: usize,  // cursor position in profile list
    compare_report: Option<crate::models::CompatibilityReport>,
    explain_scroll_offset: usize,
    // Import URL state
    import_url_state: ImportUrlState,
    // Global loading state for animated spinners
    loading_state: Option<LoadingState>,
    // Background task channel
    task_tx: mpsc::UnboundedSender<TaskResult>,
    task_rx: mpsc::UnboundedReceiver<TaskResult>,
}

impl App {
    pub fn new(config: Config, storage: Storage) -> Self {
        let ollama = OllamaClient::new(config.ollama_host.clone());
        let pipeline_storage = Storage::new(&config.data_path)
            .expect("Failed to create pipeline storage");
        let pipeline = InferencePipeline::new(
            ollama.clone(),
            config.chat_model.clone(),
            pipeline_storage,
        );
        let palette = config.theme.to_palette();
        let (task_tx, task_rx) = mpsc::unbounded_channel();
        
        Self {
            config,
            storage,
            ollama,
            pipeline,
            current_screen: Screen::Dashboard,
            should_quit: false,
            ollama_connected: false,
            palette,
            previous_screen: None,
            recent_profiles: Vec::new(),
            all_profiles: Vec::new(),
            create_profile_state: CreateProfileState::default(),
            coach_state: CoachState::default(),
            settings_state: SettingsState::default(),
            selected_profile_id: None,
            logs: Vec::new(),
            dashboard_selected: 0,
            create_profile_selected: 0,
            saved_profiles_selected: 0,
            coach_profile_selected: 0,
            compare_profile1_id: None,
            compare_profile2_id: None,
            compare_selecting: 0,
            compare_selected: 0,
            compare_report: None,
            explain_scroll_offset: 0,
            import_url_state: ImportUrlState::default(),
            loading_state: None,
            task_tx,
            task_rx,
        }
    }
    
    pub async fn run(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        info!("Starting application loop");
        
        // Check Ollama connection
        self.ollama_connected = self.ollama.check_connection().await.unwrap_or(false);
        info!("Ollama connected: {}", self.ollama_connected);
        
        // Load initial data
        self.load_profiles()?;
        self.load_logs()?;
        
        loop {
            // Tick loading spinner if active
            if let Some(ref mut loading) = self.loading_state {
                loading.tick();
            }
            if let Some(ref mut loading) = self.import_url_state.loading {
                loading.tick();
            }
            
            terminal.draw(|frame| self.render(frame))?;
            
            // Poll for background task results (non-blocking)
            self.poll_task_results()?;
            
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => {
                        self.handle_key_event(key.code, key.modifiers).await?;
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_event(mouse).await?;
                    }
                    _ => {}
                }
            }
            
            if self.should_quit {
                break;
            }
        }
        
        info!("Application loop ended");
        Ok(())
    }
    
    /// Poll for completed background tasks and handle their results
    fn poll_task_results(&mut self) -> Result<()> {
        while let Ok(result) = self.task_rx.try_recv() {
            match result {
                TaskResult::ProfileCreated(res) => {
                    self.loading_state = None;
                    match res {
                        Ok(profile) => {
                            self.storage.save_profile(&profile)?;
                            self.load_profiles()?;
                            self.create_profile_state.status_message = "Profile created successfully!".to_string();
                            self.selected_profile_id = Some(profile.id.clone());
                            self.current_screen = Screen::ProfileView(profile.id.clone());
                        }
                        Err(e) => {
                            error!("Failed to create profile: {}", e);
                            self.create_profile_state.status_message = format!("Error: {}", e);
                            self.create_profile_state.mode = CreateMode::EnterText;
                        }
                    }
                }
                TaskResult::ProfileReanalyzed(res) => {
                    self.loading_state = None;
                    match res {
                        Ok(updated) => {
                            self.storage.save_profile(&updated)?;
                            self.load_profiles()?;
                            info!("Profile '{}' reanalyzed successfully", updated.name);
                            self.current_screen = Screen::ProfileView(updated.id.clone());
                        }
                        Err(e) => {
                            error!("Failed to reanalyze profile: {}", e);
                        }
                    }
                }
                TaskResult::DraftAnalyzed(res) => {
                    self.loading_state = None;
                    match res {
                        Ok(analysis) => {
                            info!("Draft analysis complete: score {:.0}%", analysis.overall_score * 100.0);
                            self.coach_state.analysis = Some(analysis);
                            self.coach_state.mode = crate::ui::screens::coach::CoachMode::ViewAnalysis;
                        }
                        Err(e) => {
                            error!("Failed to analyze draft: {}", e);
                            self.coach_state.mode = crate::ui::screens::coach::CoachMode::EnterDraft;
                        }
                    }
                }
                TaskResult::ComparisonDone(res) => {
                    self.loading_state = None;
                    match res {
                        Ok(report) => {
                            info!("Comparison complete: {:.0}%", report.compatibility_score * 100.0);
                            self.compare_report = Some(report);
                        }
                        Err(e) => {
                            error!("Failed to compare profiles: {}", e);
                            self.compare_profile2_id = None;
                            self.compare_selecting = 1;
                        }
                    }
                }
                TaskResult::UrlScraped { result, profile_id, platform } => {
                    match result {
                        Ok(text) => {
                            let char_count = text.len();
                            let filename = format!("{}.txt", platform);
                            match self.storage.save_source_file(&profile_id, &filename, &text) {
                                Ok(()) => {
                                    info!("Saved {} chars from {} to {}/{}", char_count, platform, profile_id, filename);
                                    self.import_url_state.finish_success(char_count, &filename);
                                }
                                Err(e) => {
                                    error!("Failed to save scraped text: {}", e);
                                    self.import_url_state.finish_error(&format!("Failed to save: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            error!("Scraping failed: {}", e);
                            self.import_url_state.finish_error(&e.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.size();
        
        match &self.current_screen {
            Screen::Dashboard => {
                crate::ui::screens::dashboard::render(
                    frame,
                    area,
                    &self.recent_profiles,
                    self.ollama_connected,
                    &self.palette,
                    self.dashboard_selected,
                );
            }
            Screen::CreateProfile => {
                crate::ui::screens::create_profile::render(
                    frame, area, &self.create_profile_state,
                    &self.palette, self.create_profile_selected,
                    self.loading_state.as_ref(),
                );
            }
            Screen::ProfileView(profile_id) => {
                if let Some(profile) = self.find_profile(profile_id) {
                    crate::ui::screens::profile_view::render(
                        frame, area, profile, &self.palette,
                        self.loading_state.as_ref(),
                    );
                }
            }
            Screen::Coach => {
                crate::ui::screens::coach::render(
                    frame,
                    area,
                    &self.coach_state,
                    &self.all_profiles,
                    &self.palette,
                    self.coach_profile_selected,
                    self.loading_state.as_ref(),
                );
            }
            Screen::Compare => {
                crate::ui::screens::compare::render(
                    frame,
                    area,
                    &self.all_profiles,
                    self.compare_profile1_id.as_deref(),
                    self.compare_profile2_id.as_deref(),
                    self.compare_selecting,
                    self.compare_selected,
                    self.compare_report.as_ref(),
                    &self.palette,
                    self.loading_state.as_ref(),
                );
            }
            Screen::ImportUrl(profile_id) => {
                let profile_name = self.find_profile(profile_id)
                    .map(|p| p.name.as_str())
                    .unwrap_or(profile_id.as_str());
                crate::ui::screens::import_url::render(
                    frame, area, &self.import_url_state, profile_name, &self.palette,
                );
            }
            Screen::SavedProfiles => {
                crate::ui::screens::saved_profiles::render(frame, area, &self.all_profiles, self.saved_profiles_selected, &self.palette);
            }
            Screen::Settings => {
                crate::ui::screens::settings::render(frame, area, &self.config, &self.settings_state, &self.palette);
            }
            Screen::Logs => {
                crate::ui::screens::logs::render(frame, area, &self.logs, &self.palette);
            }
            Screen::Explain(profile_id) => {
                if let Some(profile) = self.find_profile(profile_id) {
                    crate::ui::screens::explain::render(
                        frame,
                        area,
                        profile,
                        &self.palette,
                        self.explain_scroll_offset,
                    );
                }
            }
            Screen::Help => {
                crate::ui::screens::help::render(frame, area, &self.palette);
            }
        }
    }
    
    async fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        // Global shortcuts
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('t') | KeyCode::Char('T') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Cycle theme
                self.config.theme = self.config.theme.next();
                self.palette = self.config.theme.to_palette();
                let _ = self.config.save();
                return Ok(());
            }
            KeyCode::Char('?') => {
                // Toggle help screen
                if matches!(self.current_screen, Screen::Help) {
                    self.current_screen = Screen::Dashboard;
                } else {
                    self.current_screen = Screen::Help;
                }
                return Ok(());
            }
            KeyCode::Esc => {
                if matches!(self.current_screen, Screen::Help) {
                    self.current_screen = Screen::Dashboard;
                } else if let Screen::Explain(ref pid) = self.current_screen {
                    let pid = pid.clone();
                    self.current_screen = Screen::ProfileView(pid);
                } else if let Screen::ImportUrl(ref pid) = self.current_screen {
                    // Go back to profile view from import URL
                    let pid = pid.clone();
                    self.import_url_state = ImportUrlState::default();
                    self.current_screen = Screen::ProfileView(pid);
                } else {
                    self.current_screen = Screen::Dashboard;
                    self.loading_state = None;
                }
                return Ok(());
            }
            _ => {}
        }
        
        // Screen-specific handling
        match &self.current_screen {
            Screen::Dashboard => self.handle_dashboard_keys(key).await?,
            Screen::CreateProfile => self.handle_create_profile_keys(key, modifiers).await?,
            Screen::ProfileView(_) => self.handle_profile_view_keys(key).await?,
            Screen::Coach => self.handle_coach_keys(key, modifiers).await?,
            Screen::Compare => self.handle_compare_keys(key).await?,
            Screen::ImportUrl(_) => self.handle_import_url_keys(key).await?,
            Screen::SavedProfiles => self.handle_saved_profiles_keys(key)?,
            Screen::Settings => self.handle_settings_keys(key, modifiers)?,
            Screen::Explain(_) => self.handle_explain_keys(key)?,
            _ => {}
        }
        
        Ok(())
    }
    
    async fn handle_dashboard_keys(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up => {
                if self.dashboard_selected > 0 {
                    self.dashboard_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.dashboard_selected < 5 {
                    self.dashboard_selected += 1;
                }
            }
            KeyCode::Enter => {
                match self.dashboard_selected {
                    0 => {
                        self.current_screen = Screen::CreateProfile;
                        self.create_profile_state = CreateProfileState::default();
                    }
                    1 => {
                        self.current_screen = Screen::Coach;
                        self.coach_state = CoachState::default();
                    }
                    2 => {
                        self.current_screen = Screen::SavedProfiles;
                    }
                    3 => {
                        self.current_screen = Screen::Compare;
                    }
                    4 => {
                        self.current_screen = Screen::Settings;
                    }
                    5 => {
                        self.load_logs()?;
                        self.current_screen = Screen::Logs;
                    }
                    _ => {}
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.current_screen = Screen::CreateProfile;
                self.create_profile_state = CreateProfileState::default();
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.current_screen = Screen::Coach;
                self.coach_state = CoachState::default();
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.current_screen = Screen::SavedProfiles;
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.compare_profile1_id = None;
                self.compare_profile2_id = None;
                self.compare_selecting = 0;
                self.compare_selected = 0;
                self.compare_report = None;
                self.current_screen = Screen::Compare;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.current_screen = Screen::Settings;
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.load_logs()?;
                self.current_screen = Screen::Logs;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_create_profile_keys(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match self.create_profile_state.mode {
            CreateMode::SelectType => {
                match key {
                    KeyCode::Up => {
                        if self.create_profile_selected > 0 {
                            self.create_profile_selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.create_profile_selected < 2 {
                            self.create_profile_selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        self.create_profile_state.mode = CreateMode::EnterName;
                    }
                    _ => {}
                }
            }
            CreateMode::EnterName => {
                match key {
                    KeyCode::Char(c) => {
                        self.create_profile_state.name_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.create_profile_state.name_input.pop();
                    }
                    KeyCode::Enter if !self.create_profile_state.name_input.is_empty() => {
                        self.create_profile_state.mode = CreateMode::EnterText;
                    }
                    _ => {}
                }
            }
            CreateMode::EnterText => {
                match key {
                    KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                        self.create_profile_state.text_input.push(c);
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') if modifiers.contains(KeyModifiers::CONTROL) => {
                        // Process profile in background
                        self.create_profile_state.mode = CreateMode::Processing;
                        let name = self.create_profile_state.name_input.clone();
                        self.loading_state = Some(LoadingState::new(
                            "Analyzing Profile",
                            &format!("Inferring personality traits for {}...", name),
                        ));
                        self.spawn_profile_creation();
                    }
                    KeyCode::Backspace => {
                        self.create_profile_state.text_input.pop();
                    }
                    KeyCode::Enter => {
                        self.create_profile_state.text_input.push('\n');
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_profile_view_keys(&mut self, key: KeyCode) -> Result<()> {
        let profile_id = self.selected_profile_id.clone();
        match key {
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if let Some(ref pid) = profile_id {
                    if let Some(profile) = self.find_profile(pid).cloned() {
                        self.coach_state = CoachState::default();
                        self.coach_state.selected_profile = Some(profile);
                        self.coach_state.mode = crate::ui::screens::coach::CoachMode::EnterDraft;
                        self.current_screen = Screen::Coach;
                    }
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(profile_id) = self.selected_profile_id.clone() {
                    let _ = self.storage.delete_profile(&profile_id);
                    self.load_profiles()?;
                    self.current_screen = Screen::SavedProfiles;
                }
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                if let Some(ref pid) = profile_id {
                    self.explain_scroll_offset = 0;
                    self.current_screen = Screen::Explain(pid.clone());
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if let Some(ref pid) = profile_id {
                    let pid = pid.clone();
                    let source_text = self.storage.read_all_sources(&pid)?;
                    if !source_text.is_empty() {
                        if let Some(existing) = self.find_profile(&pid).cloned() {
                            let source_count = self.storage.list_source_files(&pid)?.len();
                            self.loading_state = Some(LoadingState::new(
                                "Reanalyzing Profile",
                                &format!("Reanalyzing {} from {} source files...", existing.name, source_count),
                            ));
                            self.spawn_reanalyze(existing, source_text);
                        }
                    }
                }
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                // Import from URL
                if let Some(ref pid) = profile_id {
                    self.import_url_state = ImportUrlState::default();
                    self.current_screen = Screen::ImportUrl(pid.clone());
                }
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                if let Some(profile_id) = &self.selected_profile_id {
                    self.compare_profile1_id = Some(profile_id.clone());
                    self.compare_profile2_id = None;
                    self.compare_selecting = 1;
                    self.compare_selected = 0;
                    self.compare_report = None;
                    self.current_screen = Screen::Compare;
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_import_url_keys(&mut self, key: KeyCode) -> Result<()> {
        match self.import_url_state.mode {
            ImportUrlMode::EnterUrl => {
                match key {
                    KeyCode::Char(c) => {
                        self.import_url_state.url_input.push(c);
                        self.import_url_state.update_platform_detection();
                    }
                    KeyCode::Backspace => {
                        self.import_url_state.url_input.pop();
                        self.import_url_state.update_platform_detection();
                    }
                    KeyCode::Enter if !self.import_url_state.url_input.is_empty() => {
                        // Start scraping
                        if let Screen::ImportUrl(ref pid) = self.current_screen {
                            let profile_id = pid.clone();
                            let url = self.import_url_state.url_input.clone();
                            let platform = scraper::detect_platform(&url);
                            self.import_url_state.start_scraping();
                            self.spawn_url_scrape(url, profile_id, platform.as_str().to_string());
                        }
                    }
                    _ => {}
                }
            }
            ImportUrlMode::Scraping => {
                // Don't allow input during scraping
            }
            ImportUrlMode::Done => {
                match key {
                    KeyCode::Enter => {
                        // New URL
                        self.import_url_state = ImportUrlState::default();
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        // Reanalyze profile with new source
                        if let Screen::ImportUrl(ref pid) = self.current_screen {
                            let pid = pid.clone();
                            let source_text = self.storage.read_all_sources(&pid)?;
                            if !source_text.is_empty() {
                                if let Some(existing) = self.find_profile(&pid).cloned() {
                                    let source_count = self.storage.list_source_files(&pid)?.len();
                                    self.loading_state = Some(LoadingState::new(
                                        "Reanalyzing Profile",
                                        &format!("Reanalyzing {} from {} source files...", existing.name, source_count),
                                    ));
                                    self.current_screen = Screen::ProfileView(pid.clone());
                                    self.spawn_reanalyze(existing, source_text);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            ImportUrlMode::Error => {
                match key {
                    KeyCode::Enter => {
                        // Retry - go back to URL input
                        let url = self.import_url_state.url_input.clone();
                        self.import_url_state = ImportUrlState::default();
                        self.import_url_state.url_input = url;
                        self.import_url_state.update_platform_detection();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    
    async fn handle_compare_keys(&mut self, key: KeyCode) -> Result<()> {
        // If we have a report, just show it
        if self.compare_report.is_some() {
            match key {
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.compare_profile1_id = None;
                    self.compare_profile2_id = None;
                    self.compare_selecting = 0;
                    self.compare_selected = 0;
                    self.compare_report = None;
                }
                _ => {}
            }
            return Ok(());
        }
        
        // Don't allow input during comparison
        if self.loading_state.is_some() {
            return Ok(());
        }
        
        match key {
            KeyCode::Up => {
                if self.compare_selected > 0 {
                    self.compare_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.compare_selected < self.all_profiles.len().saturating_sub(1) {
                    self.compare_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(profile) = self.all_profiles.get(self.compare_selected) {
                    if self.compare_selecting == 0 {
                        self.compare_profile1_id = Some(profile.id.clone());
                        self.compare_selecting = 1;
                        self.compare_selected = 0;
                    } else {
                        self.compare_profile2_id = Some(profile.id.clone());
                        // Both selected - run comparison in background
                        let p1_id = self.compare_profile1_id.clone().unwrap();
                        let p2_id = self.compare_profile2_id.clone().unwrap();
                        let p1 = self.find_profile(&p1_id).cloned();
                        let p2 = self.find_profile(&p2_id).cloned();
                        if let (Some(p1), Some(p2)) = (p1, p2) {
                            self.loading_state = Some(LoadingState::new(
                                "Comparing Profiles",
                                &format!("Analyzing compatibility: {} vs {}...", p1.name, p2.name),
                            ));
                            self.spawn_comparison(p1, p2);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_coach_keys(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match self.coach_state.mode {
            crate::ui::screens::coach::CoachMode::SelectProfile => {
                match key {
                    KeyCode::Up => {
                        if self.coach_profile_selected > 0 {
                            self.coach_profile_selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.coach_profile_selected < self.all_profiles.len().saturating_sub(1) {
                            self.coach_profile_selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(profile) = self.all_profiles.get(self.coach_profile_selected) {
                            self.coach_state.selected_profile = Some(profile.clone());
                            self.coach_state.mode = crate::ui::screens::coach::CoachMode::EnterDraft;
                        }
                    }
                    _ => {}
                }
            }
            crate::ui::screens::coach::CoachMode::EnterDraft => {
                match key {
                    KeyCode::Char('s') | KeyCode::Char('S') if modifiers.contains(KeyModifiers::CONTROL) => {
                        if !self.coach_state.draft_input.is_empty() && self.coach_state.selected_profile.is_some() {
                            self.coach_state.mode = crate::ui::screens::coach::CoachMode::Processing;
                            let profile_name = self.coach_state.selected_profile.as_ref()
                                .map(|p| p.name.clone()).unwrap_or_default();
                            self.loading_state = Some(LoadingState::new(
                                "Analyzing Draft",
                                &format!("Analyzing draft against {}'s profile...", profile_name),
                            ));
                            self.spawn_draft_analysis();
                        }
                    }
                    KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                        self.coach_state.draft_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.coach_state.draft_input.pop();
                    }
                    KeyCode::Enter => {
                        self.coach_state.draft_input.push('\n');
                    }
                    _ => {}
                }
            }
            _ => {
                match key {
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.coach_state.draft_input.clear();
                        self.coach_state.analysis = None;
                        self.coach_state.mode = crate::ui::screens::coach::CoachMode::EnterDraft;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    
    fn handle_saved_profiles_keys(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up => {
                if self.saved_profiles_selected > 0 {
                    self.saved_profiles_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.saved_profiles_selected < self.all_profiles.len().saturating_sub(1) {
                    self.saved_profiles_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(profile) = self.all_profiles.get(self.saved_profiles_selected) {
                    self.selected_profile_id = Some(profile.id.clone());
                    self.current_screen = Screen::ProfileView(profile.id.clone());
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.current_screen = Screen::CreateProfile;
                self.create_profile_state = CreateProfileState::default();
            }
            _ => {}
        }
        Ok(())
    }
    
    // ---- Background task spawning ----
    
    fn spawn_profile_creation(&self) {
        let pipeline = self.pipeline.clone();
        let name = self.create_profile_state.name_input.clone();
        let text = self.create_profile_state.text_input.clone();
        let tx = self.task_tx.clone();
        
        tokio::spawn(async move {
            let result = pipeline.infer_profile_from_text(
                name, text, ProfileType::TextInference,
            ).await;
            let _ = tx.send(TaskResult::ProfileCreated(result));
        });
    }
    
    fn spawn_reanalyze(&self, existing: PersonProfile, source_text: String) {
        let pipeline = self.pipeline.clone();
        let tx = self.task_tx.clone();
        
        tokio::spawn(async move {
            let result = pipeline.reanalyze_profile(&existing, source_text).await;
            let _ = tx.send(TaskResult::ProfileReanalyzed(result));
        });
    }
    
    fn spawn_draft_analysis(&self) {
        let pipeline = self.pipeline.clone();
        let draft = self.coach_state.draft_input.clone();
        let profile = self.coach_state.selected_profile.clone().unwrap();
        let tx = self.task_tx.clone();
        
        tokio::spawn(async move {
            let result = pipeline.analyze_draft(draft, &profile).await;
            let _ = tx.send(TaskResult::DraftAnalyzed(result));
        });
    }
    
    fn spawn_comparison(&self, p1: PersonProfile, p2: PersonProfile) {
        let pipeline = self.pipeline.clone();
        let tx = self.task_tx.clone();
        
        tokio::spawn(async move {
            let result = pipeline.compare_profiles(&p1, &p2).await;
            let _ = tx.send(TaskResult::ComparisonDone(result));
        });
    }
    
    fn spawn_url_scrape(&self, url: String, profile_id: String, platform: String) {
        let tx = self.task_tx.clone();
        
        // Auto-detect or use configured Firefox profile for authenticated scraping
        let firefox_profile = scraper::find_firefox_profile(
            self.config.firefox_profile_path.as_deref()
        );
        
        // URL scraping is async (fantoccini/geckodriver)
        tokio::spawn(async move {
            let result = scraper::scrape_url(&url, firefox_profile).await;
            let _ = tx.send(TaskResult::UrlScraped {
                result,
                profile_id,
                platform,
            });
        });
    }
    
    // ---- Utility methods ----
    
    fn load_profiles(&mut self) -> Result<()> {
        self.recent_profiles = self.storage.list_profiles(Some(10))?;
        self.all_profiles = self.storage.list_profiles(None)?;
        Ok(())
    }
    
    fn load_logs(&mut self) -> Result<()> {
        self.logs = self.storage.get_recent_logs(50)?;
        Ok(())
    }
    
    fn find_profile(&self, id: &str) -> Option<&PersonProfile> {
        self.all_profiles.iter().find(|p| p.id == id)
    }
    
    fn handle_explain_keys(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up => {
                if self.explain_scroll_offset > 0 {
                    self.explain_scroll_offset -= 1;
                }
            }
            KeyCode::Down => {
                self.explain_scroll_offset += 1;
            }
            KeyCode::PageUp => {
                self.explain_scroll_offset = self.explain_scroll_offset.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.explain_scroll_offset += 10;
            }
            KeyCode::Home => {
                self.explain_scroll_offset = 0;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_settings_keys(&mut self, _key: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        Ok(())
    }
    
    async fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(_button) => {
                let (col, row) = (mouse.column, mouse.row);
                match &self.current_screen {
                    Screen::Dashboard => {
                        self.handle_dashboard_click(col, row).await?;
                    }
                    Screen::SavedProfiles => {
                        self.handle_profiles_click(col, row)?;
                    }
                    _ => {}
                }
            }
            MouseEventKind::ScrollDown => {}
            MouseEventKind::ScrollUp => {}
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_dashboard_click(&mut self, col: u16, row: u16) -> Result<()> {
        if row >= 5 && row <= 12 && col >= 50 {
            let action_row = row - 5;
            match action_row {
                2 => {
                    self.current_screen = Screen::CreateProfile;
                    self.create_profile_state = CreateProfileState::default();
                }
                3 => {
                    self.current_screen = Screen::Coach;
                    self.coach_state = CoachState::default();
                }
                4 => {
                    self.current_screen = Screen::SavedProfiles;
                }
                5 => {
                    self.current_screen = Screen::Compare;
                }
                6 => {
                    self.current_screen = Screen::Settings;
                }
                7 => {
                    self.load_logs()?;
                    self.current_screen = Screen::Logs;
                }
                _ => {}
            }
        }
        
        if row >= 5 && row < 15 && col < 50 && !self.recent_profiles.is_empty() {
            let profile_index = (row - 5) as usize;
            if let Some(profile) = self.recent_profiles.get(profile_index) {
                self.selected_profile_id = Some(profile.id.clone());
                self.current_screen = Screen::ProfileView(profile.id.clone());
            }
        }
        
        Ok(())
    }
    
    fn handle_profiles_click(&mut self, col: u16, row: u16) -> Result<()> {
        if row >= 4 && !self.all_profiles.is_empty() {
            let profile_index = (row - 4) as usize;
            if let Some(profile) = self.all_profiles.get(profile_index) {
                self.selected_profile_id = Some(profile.id.clone());
                self.current_screen = Screen::ProfileView(profile.id.clone());
            }
        }
        Ok(())
    }
}
