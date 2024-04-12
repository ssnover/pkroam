use crate::types::{BoxLocation, GameSaveData, MonsterData};
use rusqlite::Connection;
use std::path::Path;

mod internal_types;
mod migrations;
mod statements;

const CURRENT_DATABASE_SCHEMA_VERSION: i32 = 4;

pub struct DbConn {
    conn: Connection,
}

impl DbConn {
    pub fn new(db_path: impl AsRef<Path>) -> anyhow::Result<Self> {
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

    fn initialize_database(&mut self) -> anyhow::Result<()> {
        self.with_transaction(|txn| {
            txn.execute(statements::CREATE_TABLE_SAVES, ())?;
            txn.execute(statements::CREATE_TABLE_ROAM_POKEMON, ())?;
            txn.execute(statements::CREATE_TABLE_BOX_ENTRIES, ())?;

            set_schema_version(txn, CURRENT_DATABASE_SCHEMA_VERSION)?;
            Ok(())
        })
    }

    fn migrate_database(
        &mut self,
        current_version: i32,
        target_version: i32,
    ) -> anyhow::Result<()> {
        self.with_transaction(|txn| {
            for version in current_version..target_version {
                migrations::perform_migration(txn, version)?;
            }
            set_schema_version(txn, target_version)?;
            Ok(())
        })?;

        log::info!("Migrated database from version {current_version} to version {target_version}");
        Ok(())
    }

    fn with_transaction<T, F>(&mut self, op: F) -> anyhow::Result<T>
    where
        T: std::fmt::Debug + Clone,
        F: FnOnce(&rusqlite::Transaction) -> anyhow::Result<T>,
    {
        let txn = self.conn.transaction()?;
        let res = op(&txn)?;
        txn.commit()?;
        Ok(res)
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

    pub fn insert_new_mon(
        &mut self,
        mon: &MonsterData,
        location: BoxLocation,
    ) -> anyhow::Result<u64> {
        let mon = internal_types::Monster::from(mon.clone());
        self.with_transaction(|txn| {
            let _rows_changed = txn.execute(
                statements::INSERT_MON_INTO_MONS,
                (
                    &mon.original_trainer_id,
                    &mon.original_secret_id,
                    &mon.personality_value,
                    &mon.data_format,
                    mon.data.as_slice(),
                ),
            )?;
            let row_id = txn.last_insert_rowid();
            let location = internal_types::BoxEntry::from((location.clone(), row_id as u64));
            let _ = txn.execute(
                statements::INSERT_BOX_ENTRY,
                (
                    location.box_number,
                    location.box_position,
                    location.monster_id,
                ),
            )?;
            Ok(row_id as u64)
        })
    }

    pub fn get_all_mons(&self) -> anyhow::Result<Vec<MonsterData>> {
        let mut stmt = self.conn.prepare(statements::SELECT_ALL_MONS)?;
        let mons = stmt
            .query_map([], internal_types::Monster::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        mons.into_iter().map(|mon| mon.try_into()).collect()
    }

    pub fn withdraw_mon(&mut self, id: u64) -> anyhow::Result<(MonsterData, BoxLocation)> {
        let (monster, entry) = self.with_transaction(|txn| {
            let monster = txn.query_row_and_then(
                statements::SELECT_MON_WITH_ID,
                (id,),
                internal_types::Monster::from_row,
            )?;
            let entry = txn.query_row_and_then(
                statements::SELECT_BOX_ENTRY_WITH_MONSTER_ID,
                (id,),
                internal_types::BoxEntry::from_row,
            )?;
            let _rows_changed = txn.execute(statements::DELETE_MON_WITH_ID, (id,))?;
            Ok((monster, entry))
        })?;

        Ok((monster.try_into()?, entry.try_into()?))
    }
}

fn get_schema_version(conn: &Connection) -> rusqlite::Result<i32> {
    conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
}

fn set_schema_version(txn: &rusqlite::Transaction, schema_version: i32) -> rusqlite::Result<()> {
    txn.pragma_update(None, "user_version", schema_version)
}
