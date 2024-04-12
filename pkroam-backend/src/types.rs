/// This module contains data types for concepts used throughout the program.
/// They are intended to be strongly-typed such that they cannot contain invalid
/// state (i.e. a meaningless save id, a too-large vector of data)
use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct GameSaveData {
    pub id: Option<u64>,
    pub game: Game,
    pub trainer_name: String,
    pub trainer_id: u32,
    pub secret_id: u32,
    pub playtime: Playtime,
    pub connected: bool,
    pub save_path: PathBuf,
}

impl GameSaveData {
    pub fn from_path(p: impl AsRef<Path>, game_id: u32) -> anyhow::Result<Self> {
        let save_file = pkroam::save::SaveFile::new(&p)?;
        let trainer_info = save_file.get_trainer_info();
        Ok(Self {
            id: None,
            game: Game::try_from(game_id)?,
            trainer_name: trainer_info.player_name,
            trainer_id: trainer_info.id.public_id.into(),
            secret_id: trainer_info.id.secret_id.into(),
            playtime: Playtime::new(
                trainer_info.time_played.hours.into(),
                trainer_info.time_played.minutes.into(),
                trainer_info.time_played.frames.into(),
            )?,
            connected: true,
            save_path: p.as_ref().to_owned(),
        })
    }
}

impl std::fmt::Display for GameSaveData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = 10;
        write!(
            f,
            "{:width$} [{:05}]: {:10} {}",
            self.trainer_name,
            self.trainer_id,
            self.game,
            self.save_path.display(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct Playtime {
    pub hours: u32,
    pub minutes: u32,
    pub frames: u32,
}

impl Playtime {
    pub fn new(hours: u32, minutes: u32, frames: u32) -> anyhow::Result<Self> {
        if minutes > 59 || frames > 59 {
            Err(anyhow::anyhow!(
                "Invalid playtime fields: minutes {minutes}, frames: {frames}"
            ))
        } else {
            Ok(Playtime {
                hours,
                minutes,
                frames,
            })
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Game {
    Ruby = 0,
    Sapphire = 1,
    Emerald = 2,
    FireRed = 3,
    LeafGreen = 4,
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Game::Ruby => "Ruby",
            Game::Sapphire => "Sapphire",
            Game::Emerald => "Emerald",
            Game::FireRed => "FireRed",
            Game::LeafGreen => "LeafGreen",
        })
    }
}

impl TryFrom<u32> for Game {
    type Error = io::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Game::Ruby),
            1 => Ok(Game::Sapphire),
            2 => Ok(Game::Emerald),
            3 => Ok(Game::FireRed),
            4 => Ok(Game::LeafGreen),
            _ => Err(io::ErrorKind::InvalidData.into()),
        }
    }
}

impl Into<u32> for Game {
    fn into(self) -> u32 {
        match self {
            Game::Ruby => 0,
            Game::Sapphire => 1,
            Game::Emerald => 2,
            Game::FireRed => 3,
            Game::LeafGreen => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MonsterData {
    pub id: Option<u64>,
    pub original_trainer_id: u32,
    pub original_secret_id: u32,
    pub personality_value: u32,
    pub data_format: DataFormat,
    pub data: Vec<u8>,
}

impl MonsterData {
    pub fn from_pk3(pk3_data: &[u8]) -> anyhow::Result<Self> {
        let pkmn = pkroam::pk3::Pokemon::from_pk3(pk3_data)?;
        Ok(MonsterData {
            id: None,
            original_trainer_id: pkmn.original_trainer_id.public_id.into(),
            original_secret_id: pkmn.original_trainer_id.secret_id.into(),
            personality_value: pkmn.personality_value,
            data_format: DataFormat::PK3,
            data: pk3_data.to_vec(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum DataFormat {
    PK3 = 1,
    PK4 = 2,
}

impl TryFrom<u32> for DataFormat {
    type Error = io::Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(DataFormat::PK3),
            2 => Ok(DataFormat::PK4),
            _ => Err(io::ErrorKind::InvalidData.into()),
        }
    }
}

impl Into<u32> for DataFormat {
    fn into(self) -> u32 {
        match self {
            DataFormat::PK3 => 1,
            DataFormat::PK4 => 2,
        }
    }
}
