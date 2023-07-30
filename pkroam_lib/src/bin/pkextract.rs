use clap::Parser;
use pkroam::save::SaveFile;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    sav: PathBuf,
    #[arg(long)]
    box_number: u8,
    #[arg(long)]
    slot: u8,
    #[arg(long)]
    dest: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let mut save_file = SaveFile::new(&args.sav)?;
    save_file.verify_sections()?;

    match save_file.take_pokemon_from_box(args.box_number, args.slot)? {
        Some(pokemon) => {
            println!("{pokemon:?}");
            let pk3_data = pokemon.to_pk3();
            println!("Saving to {}", args.dest.display());
            std::fs::write(args.dest, pk3_data)?;
            save_file.write_to_file(&args.sav)?;
        }
        None => {
            println!("No Pokemon in that location!");
        }
    }

    Ok(())
}
