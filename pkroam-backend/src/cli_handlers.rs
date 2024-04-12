use crate::{
    database::DbConn,
    types::{DataFormat, MonsterData},
};
use prettytable::{format, row, Table};

pub fn handle_deposit(
    db_handle: DbConn,
    save_id: u32,
    box_number: u8,
    box_position: u8,
) -> anyhow::Result<()> {
    let game_save = db_handle.get_save(save_id)?;
    let mut save_file = pkroam::save::SaveFile::new(game_save.save_path.as_path())?;
    if let Some(pokemon) = save_file.take_pokemon_from_box(box_number, box_position)? {
        match save_file.write_to_file(game_save.save_path.as_path()) {
            Ok(()) => {
                let pokemon_id =
                    db_handle.insert_new_mon(&MonsterData::from_pk3(&pokemon.to_pk3())?)?;
                log::info!("Added with ID: {pokemon_id}");
            }
            Err(err) => {
                log::error!("Unable to update save file: {err}");
            }
        }
    } else {
        log::warn!("Couldn't get a Pokemon from that box slot on this save file");
    }

    Ok(())
}

pub fn handle_list_saves(db_handle: DbConn) -> anyhow::Result<()> {
    let saves = db_handle.get_saves()?;
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.add_row(row![
        "ID",
        "GAME",
        "TRAINER NAME",
        "TRAINER ID",
        "PLAYTIME",
        "PATH"
    ]);

    for save in saves.iter().filter(|save| save.connected) {
        table.add_row(row![
            save.id.expect("Saves coming from the database have an id"),
            save.game,
            save.trainer_name,
            save.trainer_id,
            format!("{:02}:{:02}", save.playtime.hours, save.playtime.minutes),
            save.save_path.display(),
        ]);
    }

    table.printstd();
    Ok(())
}

pub fn handle_list_mons(db_handle: DbConn, save_id: Option<u32>) -> anyhow::Result<()> {
    if let Some(save_id) = save_id {
        let game_save = db_handle.get_save(save_id)?;
        let save_file = pkroam::save::SaveFile::new(game_save.save_path.as_path())?;
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.add_row(row!["BOX", "SLOT", "POKEMON"]);

        for (idx, pkmn) in save_file.get_party()?.iter().enumerate() {
            table.add_row(row!["P", idx + 1, pkmn.species]);
        }

        for box_number in 1..14 {
            let box_pkmn = save_file.get_box(box_number).map_err(|err| {
                log::error!("Failed to get Pokemon from box {box_number}: {err}");
                err
            })?;
            for (position, pkmn) in box_pkmn {
                table.add_row(row![box_number, position, pkmn.species]);
            }
        }

        table.printstd();
    } else {
        // Default to check the roam boxes
        let mons = db_handle.get_all_mons()?;
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.add_row(row!["ID", "NATL DEX", "POKEMON"]);

        for mon in mons.iter() {
            if let DataFormat::PK3 = mon.data_format {
                let pkmn = pkroam::pk3::Pokemon::from_pk3(&mon.data)?;
                table.add_row(row![
                    mon.id.expect("Monster data from database must have an id"),
                    pkmn.species.national_dex_number()?,
                    pkmn.species
                ]);
            }
        }

        table.printstd();
    }
    Ok(())
}
