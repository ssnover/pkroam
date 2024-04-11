use num_traits::{FromPrimitive, ToPrimitive};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use types::{Game, GameSave, GameSaveData};

mod migrations;
mod statements;
pub mod types;

const CURRENT_DATABASE_SCHEMA_VERSION: i32 = 3;

pub struct DbConn {
    conn: Connection,
}

impl DbConn {
    pub fn new(db_path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(db_path)?;
        let schema_version = get_schema_version(&conn)?;
        log::debug!("Schema version at start: {schema_version}");

        let mut conn = Self { conn };
        if schema_version == 0 {
            conn.initialize_database()?;
            log::info!("Initialized a database from scratch");
        } else if schema_version < CURRENT_DATABASE_SCHEMA_VERSION {
            conn.migrate_database(schema_version, CURRENT_DATABASE_SCHEMA_VERSION)?;
        } else if schema_version > CURRENT_DATABASE_SCHEMA_VERSION {
            log::error!("PkRoam database was created by a newer version of the program, please update to the latest version");
            std::process::exit(1);
        }

        Ok(conn)
    }

    fn initialize_database(&self) -> rusqlite::Result<()> {
        self.conn.execute(statements::CREATE_TABLE_SAVES, ())?;
        self.conn
            .execute(statements::CREATE_TABLE_ROAM_POKEMON, ())?;

        set_schema_version(&self.conn, CURRENT_DATABASE_SCHEMA_VERSION)
    }

    fn migrate_database(
        &mut self,
        current_version: i32,
        target_version: i32,
    ) -> rusqlite::Result<()> {
        for version in current_version..target_version {
            migrations::perform_migration(&mut self.conn, version)?;
        }
        set_schema_version(&self.conn, target_version)?;
        log::info!("Migrated database from version {current_version} to version {target_version}");
        Ok(())
    }

    pub fn get_save(&self, save_id: u32) -> rusqlite::Result<GameSave> {
        self.conn
            .query_row_and_then(statements::SELECT_SAVE, (save_id,), |row| {
                Ok(GameSave::new(
                    row.get(0)?,
                    GameSaveData::new(
                        Game::from_u32(row.get(1)?).unwrap(),
                        row.get::<_, String>(2)?.as_str(),
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        PathBuf::from_str(row.get::<_, String>(8)?.as_str()).unwrap(),
                        row.get::<_, i32>(9)? != 0,
                    ),
                ))
            })
    }

    pub fn get_saves(&self) -> rusqlite::Result<Vec<GameSave>> {
        let mut stmt = self.conn.prepare(statements::SELECT_SAVES)?;
        let iter = stmt.query_map([], |row| {
            let trainer_name: String = row.get(2)?;
            let save_path: String = row.get(8)?;
            Ok(GameSave::new(
                row.get(0)?,
                GameSaveData::new(
                    Game::from_u32(row.get(1)?).unwrap(),
                    &trainer_name,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    PathBuf::from_str(&save_path).unwrap(),
                    row.get::<_, i32>(9)? != 0,
                ),
            ))
        })?;
        iter.collect::<rusqlite::Result<Vec<_>>>()
    }

    pub fn add_new_save(&self, save: &GameSaveData) -> rusqlite::Result<()> {
        let _rows_changed = self.conn.execute(
            statements::INSERT_SAVE_INTO_SAVES,
            (
                &save.game.to_u32(),
                &save.trainer_name,
                &save.trainer_id,
                &save.secret_id,
                &save.playtime.hours,
                &save.playtime.minutes,
                &save.playtime.frames,
                &save.save_path.to_string_lossy(),
                1,
            ),
        )?;
        Ok(())
    }

    pub fn set_save_disconnected(&self, save_id: u32) -> rusqlite::Result<()> {
        let _rows_changed = self
            .conn
            .execute(statements::UPDATE_SAVE_CONNECTED, (0, save_id))?;
        Ok(())
    }

    pub fn insert_new_mon(
        &self,
        original_trainer_id: u32,
        secret_trainer_id: u32,
        personality_value: u32,
        data: Vec<u8>,
    ) -> rusqlite::Result<u32> {
        let _rows_changed = self.conn.execute(
            statements::INSERT_MON_INTO_MONS,
            (
                &original_trainer_id,
                &secret_trainer_id,
                &personality_value,
                &1,
                data.as_slice(),
            ),
        )?;
        let row_id = self.conn.last_insert_rowid();
        Ok(row_id as u32)
    }
}

fn get_schema_version(conn: &Connection) -> rusqlite::Result<i32> {
    conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
}

fn set_schema_version(conn: &Connection, schema_version: i32) -> rusqlite::Result<()> {
    conn.pragma_update(None, "user_version", schema_version)
}
