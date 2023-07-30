use std::path::PathBuf;

pub struct GameSave {
    id: u64,
    pub trainer_name: String,
    pub trainer_id: u32,
    pub secret_id: u32,
    pub connected: bool,
    pub save_path: PathBuf,
}

impl GameSave {
    pub fn new(
        id: u64,
        trainer_name: &str,
        trainer_id: u32,
        secret_id: u32,
        save_path: PathBuf,
    ) -> Self {
        Self {
            id,
            trainer_name: trainer_name.to_owned(),
            trainer_id,
            secret_id,
            connected: save_path.exists(),
            save_path,
        }
    }
}
