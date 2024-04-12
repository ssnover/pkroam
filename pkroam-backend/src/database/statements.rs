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

pub const SELECT_ALL_MONS: &str = "SELECT id, original_trainer_id, original_secret_id, personality_value, data_format, data FROM monsters";

pub const SELECT_MON_WITH_ID: &str = "SELECT id, original_trainer_id, original_secret_id, personality_value, data_format, data FROM monsters
    WHERE id = ?";

pub const DELETE_MON_WITH_ID: &str = "DELETE FROM monsters WHERE id = ?";

pub const CREATE_TABLE_BOX_ENTRIES: &str = "CREATE TABLE box_entries (
    box_number INTEGER,
    box_position INTEGER,
    monster_id INTEGER UNIQUE,
    FOREIGN KEY (monster_id)
        REFERENCES monsters (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    UNIQUE (box_number, box_position)
)";

pub const INSERT_BOX_ENTRY: &str =
    "INSERT INTO box_entries (box_number, box_position, monster_id) VALUES (?1, ?2, ?3)";

pub const SELECT_BOX_ENTRY_WITH_MONSTER_ID: &str =
    "SELECT box_number, box_position, monster_id FROM box_entries WHERE monster_id = ?";
