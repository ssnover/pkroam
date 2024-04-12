use std::path::PathBuf;

/// This module contains data types which closely match the schema of the database
/// and conversions to the constrained data types used by the rest of the program.

#[derive(Clone, Debug)]
pub struct Save {
    pub id: u64,
    pub game: u32,
    pub trainer_name: String,
    pub trainer_id: u64,
    pub secret_id: u64,
    pub playtime_hours: u64,
    pub playtime_minutes: u64,
    pub playtime_frames: u64,
    pub save_path: String,
    pub connected: u64,
}

impl Save {
    pub fn from_row(row: &rusqlite::Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Save {
            id: row.get(0)?,
            game: row.get(1)?,
            trainer_name: row.get(2)?,
            trainer_id: row.get(3)?,
            secret_id: row.get(4)?,
            playtime_hours: row.get(5)?,
            playtime_minutes: row.get(6)?,
            playtime_frames: row.get(7)?,
            save_path: row.get(8)?,
            connected: row.get(9)?,
        })
    }
}

impl TryInto<crate::types::GameSaveData> for Save {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<crate::types::GameSaveData, Self::Error> {
        Ok(crate::types::GameSaveData {
            id: Some(self.id),
            game: crate::types::Game::try_from(self.game)?,
            trainer_name: self.trainer_name,
            trainer_id: self.trainer_id.try_into()?,
            secret_id: self.secret_id.try_into()?,
            playtime: crate::types::Playtime::new(
                self.playtime_hours.try_into()?,
                self.playtime_minutes.try_into()?,
                self.playtime_frames.try_into()?,
            )?,
            save_path: PathBuf::from(self.save_path),
            connected: self.connected != 0,
        })
    }
}

impl From<crate::types::GameSaveData> for Save {
    fn from(value: crate::types::GameSaveData) -> Self {
        Self {
            id: value.id.unwrap_or(0),
            game: value.game.into(),
            trainer_name: value.trainer_name,
            trainer_id: value.trainer_id.into(),
            secret_id: value.secret_id.into(),
            playtime_hours: value.playtime.hours.into(),
            playtime_minutes: value.playtime.minutes.into(),
            playtime_frames: value.playtime.frames.into(),
            save_path: value.save_path.to_string_lossy().to_string(),
            connected: value.connected.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Monster {
    pub id: u64,
    pub original_trainer_id: u64,
    pub original_secret_id: u64,
    pub personality_value: u64,
    pub data_format: u32,
    pub data: Vec<u8>,
}

impl Monster {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Monster {
            id: row.get(0)?,
            original_trainer_id: row.get(1)?,
            original_secret_id: row.get(2)?,
            personality_value: row.get(3)?,
            data_format: row.get(4)?,
            data: row.get(5)?,
        })
    }
}

impl TryInto<crate::types::MonsterData> for Monster {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<crate::types::MonsterData, Self::Error> {
        Ok(crate::types::MonsterData {
            id: Some(self.id),
            original_trainer_id: self.original_trainer_id.try_into()?,
            original_secret_id: self.original_secret_id.try_into()?,
            personality_value: self.personality_value.try_into()?,
            data_format: crate::types::DataFormat::try_from(self.data_format)?,
            data: self.data,
        })
    }
}

impl From<crate::types::MonsterData> for Monster {
    fn from(value: crate::types::MonsterData) -> Self {
        Self {
            id: value.id.unwrap_or(0),
            original_trainer_id: value.original_trainer_id.into(),
            original_secret_id: value.original_secret_id.into(),
            personality_value: value.personality_value.into(),
            data_format: value.data_format.into(),
            data: value.data,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoxEntry {
    pub box_number: u32,
    pub box_position: u32,
    pub monster_id: u64,
}

impl BoxEntry {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            box_number: row.get(0)?,
            box_position: row.get(1)?,
            monster_id: row.get(2)?,
        })
    }
}

impl TryInto<crate::types::BoxLocation> for BoxEntry {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<crate::types::BoxLocation, Self::Error> {
        crate::types::BoxLocation::new(self.box_number, self.box_position, Some(self.monster_id))
    }
}

impl From<(crate::types::BoxLocation, u64)> for BoxEntry {
    fn from((location, monster_id): (crate::types::BoxLocation, u64)) -> Self {
        Self {
            box_number: location.box_number(),
            box_position: location.box_position(),
            monster_id: monster_id,
        }
    }
}
