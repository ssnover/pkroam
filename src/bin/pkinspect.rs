use std::path::PathBuf;

use clap::Parser;
use pkroam::save::SaveFile;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    sav: PathBuf,
    #[arg(short, long)]
    location: String,
    #[arg(long)]
    slot: Option<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let save_file = SaveFile::new(args.sav)?;

    save_file.verify_sections()?;

    let trainer_info = save_file.get_trainer_info();
    println!("Trainer Info: {trainer_info:?}");

    if args.location == "party" {
        let party_pkmn = save_file.get_party()?;
        for pkmn in party_pkmn {
            println!("{}", pkmn.species);
        }
    } else if args.location.starts_with("box") {
        let box_number = args.location[3..].parse::<u8>()?;
        let boxed_pkmn = save_file.get_box(box_number)?;
        for (slot, pkmn) in boxed_pkmn {
            println!("Slot {slot}: {}", pkmn.species);
        }
    }

    Ok(())
}
