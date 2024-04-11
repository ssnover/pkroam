/// Entrypoint for a CLI for testing the backend systems manually, or just convenient scripting perhaps.
use clap::{Parser, Subcommand};
use pkroam_backend::{
    app_paths::get_app_paths,
    cli_handlers::{handle_deposit, handle_list_mons, handle_list_saves},
    database::DbConn,
    logging,
};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[arg(long, short = 'c')]
    config_dir: Option<PathBuf>,
    #[arg(long, default_value = "true")]
    enable_debug: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Deposit {
        #[arg(long)]
        save: u32,
        #[arg(long)]
        box_number: u8,
        #[arg(long)]
        box_position: u8,
    },
    ListSaves,
    ListMons {
        #[arg(long)]
        save: Option<u32>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let app_paths = get_app_paths(args.config_dir)?;
    logging::initialize(args.enable_debug, &app_paths.get_log_path())?;
    let db_handle = DbConn::new(&app_paths.get_database_path())?;

    match args.command {
        Commands::Deposit {
            save,
            box_number,
            box_position,
        } => handle_deposit(db_handle, save, box_number, box_position),
        Commands::ListSaves => handle_list_saves(db_handle),
        Commands::ListMons { save } => handle_list_mons(db_handle, save),
    }
}
