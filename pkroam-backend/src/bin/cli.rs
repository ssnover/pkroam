/// Entrypoint for a CLI for testing the backend systems manually, or just convenient scripting perhaps.
use clap::{Parser, Subcommand};
use pkroam_backend::{
    app_paths::get_app_paths,
    cli_handlers::{handle_deposit, handle_list_mons, handle_list_saves, handle_withdraw},
    database::DbConn,
    //logging,
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
        #[arg(long)]
        dest_box: u32,
        #[arg(long)]
        dest_position: u32,
    },
    ListSaves,
    ListMons {
        #[arg(long)]
        save: Option<u32>,
    },
    Withdraw {
        #[arg(long)]
        mon_id: u64,
        #[arg(long)]
        save_id: u32,
        #[arg(long)]
        box_number: u8,
        #[arg(long)]
        box_position: u8,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    env_logger::init_from_env(env_logger::Env::new().filter("RUST_LOG"));

    let app_paths = get_app_paths(args.config_dir)?;
    //logging::initialize(args.enable_debug, &app_paths.get_log_path())?;
    let db_handle = DbConn::new(&app_paths.get_database_path())?;

    match args.command {
        Commands::Deposit {
            save,
            box_number,
            box_position,
            dest_box,
            dest_position,
        } => handle_deposit(
            db_handle,
            save,
            box_number,
            box_position,
            dest_box,
            dest_position,
        ),
        Commands::ListSaves => handle_list_saves(db_handle),
        Commands::ListMons { save } => handle_list_mons(db_handle, save),
        Commands::Withdraw {
            mon_id,
            save_id,
            box_number,
            box_position,
        } => handle_withdraw(db_handle, mon_id, save_id, box_number, box_position),
    }
    .map_err(|err| {
        eprintln!("Failed to execute command: {err}");
        err
    })
}
