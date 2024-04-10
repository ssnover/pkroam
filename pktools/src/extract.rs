use clap::Args;
use pkroam::save::SaveFile;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct Opts {
    #[arg(short, long)]
    pub sav: PathBuf,
    #[arg(long)]
    pub box_number: u8,
    #[arg(long)]
    pub slot: u8,
    #[arg(long)]
    pub dest: PathBuf,
}

pub fn run(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let mut save_file = SaveFile::new(&opts.sav)?;
    save_file.verify_sections()?;

    match save_file.take_pokemon_from_box(opts.box_number, opts.slot)? {
        Some(pokemon) => {
            let pk3_data = pokemon.to_pk3();
            println!("Saving to {}", opts.dest.display());
            std::fs::write(opts.dest, pk3_data)?;
            save_file.write_to_file(&opts.sav)?;
        }
        None => {
            println!("No Pokemon in that location!");
        }
    }

    Ok(())
}
