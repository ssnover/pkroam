use clap::Args;
use pkroam::save::SaveFile;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct Opts {
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

pub fn run(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let mut save_file = SaveFile::new(&opts.sav)?;
    save_file.verify_sections()?;

    let pk3_data = std::fs::read(opts.pk3)?;
    if save_file.put_pokemon_in_box(
        opts.box_number,
        opts.slot,
        &pk3_data[..],
        opts.force.unwrap_or(false),
    )? {
        save_file.write_to_file(&opts.sav)?;
        println!("Wrote Pokemon into save file");
    } else {
        eprintln!("That box position is occupied!");
    }

    Ok(())
}
