use clap::Args;
use pkroam::save::SaveFile;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct Opts {
    #[arg(short, long)]
    sav: PathBuf,
    #[arg(short, long)]
    location: String,
    #[arg(long)]
    slot: Option<u8>,
}

pub fn run(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let save_file = SaveFile::new(opts.sav)?;
    save_file.verify_sections()?;

    let trainer_info = save_file.get_trainer_info();
    println!("Trainer Info: {trainer_info:?}");

    if opts.location == "party" {
        let party_pkmn = save_file.get_party()?;
        for pkmn in party_pkmn {
            println!("{pkmn:?}");
        }
    } else if opts.location.starts_with("box") {
        let box_number = opts.location[3..].parse::<u8>()?;
        let boxed_pkmn = save_file.get_box(box_number)?;
        for (slot, pkmn) in boxed_pkmn {
            println!("Slot {slot}: {pkmn:?}");
        }
    }

    Ok(())
}
