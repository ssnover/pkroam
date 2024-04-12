use crate::types::{GameSaveData, MonsterData};
use rusqlite::Connection;
use std::path::Path;

mod internal_types;
mod migrations;
mod statements;

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

    pub fn get_save(&self, save_id: u32) -> anyhow::Result<GameSaveData> {
        self.conn
            .query_row_and_then(
                statements::SELECT_SAVE,
                (save_id,),
                internal_types::Save::from_row,
            )?
            .try_into()
    }

    pub fn get_saves(&self) -> anyhow::Result<Vec<GameSaveData>> {
        let mut stmt = self.conn.prepare(statements::SELECT_SAVES)?;
        let saves = stmt
            .query_map([], internal_types::Save::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        saves.into_iter().map(|save| save.try_into()).collect()
    }

    pub fn add_new_save(&self, save: &GameSaveData) -> anyhow::Result<()> {
        let save = internal_types::Save::from(save.clone());
        let _rows_changed = self.conn.execute(
            statements::INSERT_SAVE_INTO_SAVES,
            (
                &save.game,
                &save.trainer_name,
                &save.trainer_id,
                &save.secret_id,
                &save.playtime_hours,
                &save.playtime_minutes,
                &save.playtime_frames,
                &save.save_path,
                &save.connected,
            ),
        )?;
        Ok(())
    }

    pub fn set_save_disconnected(&self, save_id: u32) -> anyhow::Result<()> {
        let _rows_changed = self
            .conn
            .execute(statements::UPDATE_SAVE_CONNECTED, (0, save_id))?;
        Ok(())
    }

    pub fn insert_new_mon(&self, mon: &MonsterData) -> anyhow::Result<u32> {
        let mon = internal_types::Monster::from(mon.clone());
        let _rows_changed = self.conn.execute(
            statements::INSERT_MON_INTO_MONS,
            (
                &mon.original_trainer_id,
                &mon.original_secret_id,
                &mon.personality_value,
                &mon.data_format,
                mon.data.as_slice(),
            ),
        )?;
        let row_id = self.conn.last_insert_rowid();
        Ok(row_id as u32)
    }

    pub fn get_all_mons(&self) -> anyhow::Result<Vec<MonsterData>> {
        let mut stmt = self.conn.prepare(statements::SELECT_ALL_MONS)?;
        let mons = stmt
            .query_map([], internal_types::Monster::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        mons.into_iter().map(|mon| mon.try_into()).collect()
    }
}

fn get_schema_version(conn: &Connection) -> rusqlite::Result<i32> {
    conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
}

fn set_schema_version(conn: &Connection, schema_version: i32) -> rusqlite::Result<()> {
    conn.pragma_update(None, "user_version", schema_version)
}
