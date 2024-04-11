use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct GameSave {
    pub id: u64,
    pub data: GameSaveData,
}

impl GameSave {
    pub fn new(id: u64, data: GameSaveData) -> Self {
        Self { id, data }
    }
}

#[derive(Clone, Debug)]
pub struct GameSaveData {
    pub game: Game,
    pub trainer_name: String,
    pub trainer_id: u32,
    pub secret_id: u32,
    pub playtime: Playtime,
    pub connected: bool,
    pub save_path: PathBuf,
}

impl GameSaveData {
    pub fn new(
        game: Game,
        trainer_name: &str,
        trainer_id: u32,
        secret_id: u32,
        playtime_hours: u32,
        playtime_minutes: u32,
        playtime_frames: u32,
        save_path: PathBuf,
        connected: bool,
    ) -> Self {
        Self {
            game,
            trainer_name: trainer_name.to_owned(),
            trainer_id,
            secret_id,
            playtime: Playtime::new(playtime_hours, playtime_minutes, playtime_frames),
            connected,
            save_path,
        }
    }

    pub fn from_path(p: impl AsRef<Path>, game_id: u32) -> std::io::Result<Self> {
        let save_file = pkroam::save::SaveFile::new(&p)?;
        let trainer_info = save_file.get_trainer_info();
        Ok(Self {
            game: Game::from_u32(game_id).unwrap(),
            trainer_name: trainer_info.player_name,
            trainer_id: trainer_info.id.public_id.into(),
            secret_id: trainer_info.id.secret_id.into(),
            playtime: Playtime::new(
                trainer_info.time_played.hours.into(),
                trainer_info.time_played.minutes.into(),
                trainer_info.time_played.frames.into(),
            ),
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
    pub fn new(hours: u32, minutes: u32, frames: u32) -> Self {
        Playtime {
            hours,
            minutes,
            frames,
        }
    }
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
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
