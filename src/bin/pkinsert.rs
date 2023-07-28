use std::path::PathBuf;

use clap::Parser;
use pkroam::save::SaveFile;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    sav: PathBuf,
    #[arg(long)]
    box_number: u8,
    #[arg(long)]
    slot: u8,
    #[arg(long)]
    pk3: PathBuf,
    #[arg(short, long)]
    force: Option<bool>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let mut save_file = SaveFile::new(&args.sav)?;
    save_file.verify_sections()?;

    let pk3_data = std::fs::read(args.pk3)?;
    if save_file.put_pokemon_in_box(
        args.box_number,
        args.slot,
        &pk3_data[..],
        args.force.unwrap_or(false),
    )? {
        save_file.write_to_file(&args.sav)?;
        println!("Wrote Pokemon into save file");
    } else {
        eprintln!("That box position is occupied!");
    }

    Ok(())
}
