pub fn perform_migration(
    txn: &rusqlite::Transaction,
    starting_version: i32,
) -> rusqlite::Result<()> {
    match starting_version {
        1 => migrate_from_1_to_2(txn),
        2 => migrate_from_2_to_3(txn),
        3 => migrate_from_3_to_4(txn),
        ver => {
            log::error!("Request to migrate invalid database version {ver}");
            Err(rusqlite::Error::InvalidQuery)
        }
    }
}

fn migrate_from_3_to_4(txn: &rusqlite::Transaction) -> rusqlite::Result<()> {
    log::debug!("Beginning migration 3 to 4");
    let _ = txn.execute(
        "CREATE TABLE box_entries (
            box_number INTEGER,
            box_position INTEGER,
            monster_id INTEGER UNIQUE,
            FOREIGN KEY (monster_id)
                REFERENCES monsters (id)
                ON UPDATE CASCADE
                ON DELETE CASCADE,
            UNIQUE (box_number, box_position)
        )",
        (),
    )?;
    Ok(())
}

fn migrate_from_2_to_3(txn: &rusqlite::Transaction) -> rusqlite::Result<()> {
    log::debug!("Beginning migration 2 to 3");
    let _row_changed = txn.execute(
        "CREATE TABLE monsters (
        id INTEGER PRIMARY KEY,
        original_trainer_id INTEGER,
        original_secret_id INTEGER,
        personality_value INTEGER,
        data_format INTEGER,
        data BLOB
    )",
        (),
    )?;
    Ok(())
}

fn migrate_from_1_to_2(txn: &rusqlite::Transaction) -> rusqlite::Result<()> {
    log::debug!("Beginning migration 1 to 2");
    let _row_changed = txn.execute("ALTER TABLE saves ADD COLUMN connected DEFAULT 1", ())?;
    Ok(())
}
