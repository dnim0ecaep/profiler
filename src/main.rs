use anyhow::Result;
use profiler::app::App;
use profiler::config::Config;
use profiler::storage::Storage;
use profiler::ui::terminal::{init_terminal, restore_terminal};
use std::io;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("profiler")
        .join("logs");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "profiler.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .init();

    info!("Starting profiler application");

    // Load configuration
    let config = Config::load_or_default()?;
    info!("Configuration loaded");

    // Initialize storage
    let storage = Storage::new(&config.data_path)?;
    storage.initialize()?;
    info!("Storage initialized");

    // Initialize terminal
    let mut terminal = init_terminal()?;

    // Create and run app
    let mut app = App::new(config, storage);
    let result = app.run(&mut terminal).await;

    // Restore terminal
    restore_terminal()?;

    result
}
