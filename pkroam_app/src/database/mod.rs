use crate::types::GameSave;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const CURRENT_DATABASE_SCHEMA_VERSION: i32 = 1;

pub struct DbConn {
    conn: Connection,
}

impl DbConn {
    pub fn new(db_path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(db_path)?;
        let schema_version = get_schema_version(&conn)?;
        log::debug!("Schema version at start: {schema_version}");

        let conn = Self { conn };
        if schema_version == 0 {
            conn.initialize_database()?;
            log::info!("Initialized a database from scratch");
        } else if schema_version < CURRENT_DATABASE_SCHEMA_VERSION {
            conn.migrate_database()?;
        } else if schema_version > CURRENT_DATABASE_SCHEMA_VERSION {
            log::error!("PkRoam database was created by a newer version of the program, please update to the latest version");
            std::process::exit(1);
        }

        Ok(conn)
    }

    fn initialize_database(&self) -> rusqlite::Result<()> {
        self.conn.execute(
            "CREATE TABLE saves (
                id INTEGER PRIMARY KEY,
                trainer_name TEXT NOT NULL,
                trainer_id INTEGER,
                secret_id INTEGER,
                save_path TEXT NOT NULL
            )",
            (),
        )?;

        set_schema_version(&self.conn, CURRENT_DATABASE_SCHEMA_VERSION)
    }

    fn migrate_database(&self) -> rusqlite::Result<()> {
        todo!();
    }

    pub fn get_saves(&self) -> rusqlite::Result<Vec<GameSave>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, trainer_name, trainer_id, secret_id, save_path FROM saves")?;
        let iter = stmt.query_map([], |row| {
            let trainer_name: String = row.get(1)?;
            let save_path: String = row.get(4)?;
            Ok(GameSave::new(
                row.get(0)?,
                &trainer_name,
                row.get(2)?,
                row.get(3)?,
                PathBuf::from_str(&save_path).unwrap(),
            ))
        })?;
        iter.collect::<rusqlite::Result<Vec<_>>>()
    }

    pub fn add_new_save(&self, save: &GameSave) -> rusqlite::Result<()> {
        let _rows_changed = self.conn.execute("INSERT INTO saves (trainer_name, trainer_id, secret_id, save_path) VALUES (?1, ?2, ?3, ?4)",
            (&save.trainer_name, &save.trainer_id, &save.secret_id, &save.save_path.to_string_lossy()))?;
        Ok(())
    }
}

fn get_schema_version(conn: &Connection) -> rusqlite::Result<i32> {
    conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
}

fn set_schema_version(conn: &Connection, schema_version: i32) -> rusqlite::Result<()> {
    conn.pragma_update(None, "user_version", schema_version)
}
