use std::path::PathBuf;

use num_derive::{FromPrimitive, ToPrimitive};

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
    pub connected: bool,
    pub save_path: PathBuf,
}

impl GameSaveData {
    pub fn new(
        game: Game,
        trainer_name: &str,
        trainer_id: u32,
        secret_id: u32,
        save_path: PathBuf,
    ) -> Self {
        Self {
            game,
            trainer_name: trainer_name.to_owned(),
            trainer_id,
            secret_id,
            connected: save_path.exists(),
            save_path,
        }
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

#[derive(FromPrimitive, ToPrimitive, Clone, Debug)]
pub enum Game {
    Ruby = 0,
    Sapphire = 1,
    Emerald = 2,
    FireRed = 3,
    LeafGreen = 4,
}

impl Game {
    pub fn variants() -> Vec<Game> {
        vec![
            Game::Ruby,
            Game::Sapphire,
            Game::Emerald,
            Game::FireRed,
            Game::LeafGreen,
        ]
    }
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
