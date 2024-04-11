pub const CREATE_TABLE_SAVES: &str = "CREATE TABLE saves (
    id INTEGER PRIMARY KEY,
    game INTEGER,
    trainer_name TEXT NOT NULL,
    trainer_id INTEGER,
    secret_id INTEGER,
    playtime_hours INTEGER,
    playtime_minutes INTEGER,
    playtime_frames INTEGER,
    save_path TEXT NOT NULL,
    connected INTEGER
)";

pub const SELECT_SAVES: &str =
    "SELECT id, game, trainer_name, trainer_id, secret_id, playtime_hours, playtime_minutes, playtime_frames, save_path, connected FROM saves";

pub const SELECT_SAVE: &str =
    "SELECT id, game, trainer_name, trainer_id, secret_id, playtime_hours, playtime_minutes, playtime_frames, save_path, connected FROM saves
    WHERE id = (?1)";

pub const INSERT_SAVE_INTO_SAVES: &str = "INSERT INTO saves (
    game, trainer_name, trainer_id, secret_id, 
    playtime_hours, playtime_minutes, playtime_frames, save_path, connected) 
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";

pub const UPDATE_SAVE_CONNECTED: &str = "UPDATE saves SET connected = ? WHERE id = ?";

pub const CREATE_TABLE_ROAM_POKEMON: &str = "CREATE TABLE monsters (
    id INTEGER PRIMARY KEY,
    original_trainer_id INTEGER,
    original_secret_id INTEGER,
    personality_value INTEGER,
    data_format INTEGER,
    data BLOB
)";

pub const INSERT_MON_INTO_MONS: &str = "INSERT INTO monsters (
    original_trainer_id, original_secret_id, personality_value, data_format, data)
    VALUES (?1, ?2, ?3, ?4, ?5)";
