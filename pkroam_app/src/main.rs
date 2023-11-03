use app::start_app_backend;
use clap::Parser;
use std::path::PathBuf;

mod app;
mod app_paths;
mod database;
mod logging;
mod types;
mod ui;

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    config_dir: Option<PathBuf>,
    #[arg(long)]
    enable_debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let app_paths = match app_paths::get_app_paths(&args) {
        Ok(app_paths) => app_paths,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    logging::initialize(args.enable_debug, &app_paths.get_log_path())?;
    if args.enable_debug {
        println!("Logging to path: {}", &app_paths.get_log_path().display());
    }
    let db_handle = database::DbConn::new(&app_paths.get_database_path())?;

    let (backend_handle, event_sender, app_state) = start_app_backend(db_handle)?;
    ui::run_app_ui(app_state, event_sender)?;

    backend_handle.quit();

    Ok(())
}
