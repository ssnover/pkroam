pub fn perform_migration(
    db_conn: &mut rusqlite::Connection,
    starting_version: i32,
) -> rusqlite::Result<()> {
    match starting_version {
        1 => migrate_from_1_to_2(db_conn),
        2 => migrate_from_2_to_3(db_conn),
        ver => {
            log::error!("Request to migrate invalid database version {ver}");
            Err(rusqlite::Error::InvalidQuery)
        }
    }
}

fn migrate_from_2_to_3(db_conn: &mut rusqlite::Connection) -> rusqlite::Result<()> {
    log::debug!("Beginning migration 2 to 3");
    let txn = db_conn.transaction()?;
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
    txn.commit()
}

fn migrate_from_1_to_2(db_conn: &mut rusqlite::Connection) -> rusqlite::Result<()> {
    log::debug!("Beginning migration 1 to 2");
    let txn = db_conn.transaction()?;
    let _row_changed = txn.execute("ALTER TABLE saves ADD COLUMN connected DEFAULT 1", ())?;
    txn.commit()
}
