use std::{
    io::{self, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::{decode_text, TrainerId};
use crate::{
    pk3::{self as pokemon, species::Species},
    Pokemon,
};

pub struct SaveFile {
    source: PathBuf,
    full_contents: Vec<u8>,
    latest_save_offset: u64,
    section_rotation: u8,
    game_code: Option<GameCode>,
    trainer_info: Option<TrainerInfo>,
}

const GAME_SAVE_DATA_LENGTH: usize = 131072;
const SAVE_INDEX_OFFSET: u64 = 0x0FFC;
const SAVE_A_OFFSET: u64 = 0x0000;
const SAVE_B_OFFSET: u64 = 0xE000;
const SECTION_SIZE: u64 = 0x1000;
const SECTION_DATA_SIZE: usize = 3968;
const SECTION_CHECKSUM_OFFSET: u64 = 0x0ff6;
const NUMBER_OF_SECTIONS: u8 = 14;

#[derive(Clone, Copy)]
pub enum GameCode {
    RubySapphire,
    FireRedLeafGreen,
    Emerald,
}

impl GameCode {
    fn team_size_offset(&self) -> u64 {
        match self {
            GameCode::RubySapphire | GameCode::Emerald => 0x0234,
            GameCode::FireRedLeafGreen => 0x0034,
        }
    }

    fn pokedex_owned(&self) -> u64 {
        0x0028
    }

    fn pokedex_seen_a(&self) -> u64 {
        0x005c
    }

    fn pokedex_seen_b(&self) -> u64 {
        match self {
            GameCode::RubySapphire => 0x0938,
            GameCode::Emerald => 0x0988,
            GameCode::FireRedLeafGreen => 0x05f8,
        }
    }

    fn pokedex_seen_c(&self) -> u64 {
        match self {
            GameCode::RubySapphire => 0x0c0c,
            GameCode::Emerald => 0x0ca4,
            GameCode::FireRedLeafGreen => 0x0b98,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PlayerGender {
    Male,
    Female,
}

#[derive(Clone, Copy, Debug)]
pub struct TimePlayed {
    pub hours: u16,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
}

#[derive(Clone, Debug)]
pub struct TrainerInfo {
    pub player_name: String,
    pub player_gender: PlayerGender,
    pub id: TrainerId,
    pub time_played: TimePlayed,
}

impl SaveFile {
    pub fn new(p: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        if p.as_ref().is_file() {
            let file = std::fs::File::open(&p)?;
            let mut reader = std::io::BufReader::new(file);
            let mut full_contents = Vec::new();
            let read_len = reader.read_to_end(&mut full_contents)?;
            if read_len >= GAME_SAVE_DATA_LENGTH {
                let latest_save_offset = determine_latest_game_save_offset(&full_contents)?;
                let section_rotation =
                    determine_section_rotation(latest_save_offset, &full_contents)?;
                let mut save = SaveFile {
                    source: p.as_ref().to_path_buf(),
                    full_contents,
                    latest_save_offset,
                    section_rotation,
                    game_code: None,
                    trainer_info: None,
                };
                let (trainer_info, game_code) = save.parse_trainer_info()?;
                save.trainer_info = Some(trainer_info);
                save.game_code = Some(game_code);

                Ok(save)
            } else {
                log::error!("Invalid file length for a game save. Found: {read_len}, Expected: {GAME_SAVE_DATA_LENGTH}");
                Err(std::io::ErrorKind::InvalidInput.into())
            }
        } else {
            log::error!("No file at path: {}", p.as_ref().display());
            Err(std::io::ErrorKind::InvalidInput.into())
        }
    }

    fn get_offset_for_section(&self, section_id: u8) -> u64 {
        let new_section_id = (section_id + self.section_rotation) % NUMBER_OF_SECTIONS;
        self.latest_save_offset + (SECTION_SIZE * new_section_id as u64)
    }

    pub fn get_game_code(&self) -> GameCode {
        self.game_code.unwrap()
    }

    pub fn get_trainer_info(&self) -> TrainerInfo {
        self.trainer_info.clone().unwrap()
    }

    pub fn get_party(&self) -> io::Result<Vec<Pokemon>> {
        let section_offset = self.get_offset_for_section(1);
        let mut cursor = Cursor::new(&self.full_contents[..]);
        let team_size_offset = self.game_code.unwrap().team_size_offset();
        cursor.seek(SeekFrom::Start(section_offset + team_size_offset))?;
        let team_size = cursor.read_u32::<LittleEndian>()?;

        let mut pk3_buffer = [0u8; pokemon::PK3_SIZE_PARTY];
        (0..team_size)
            .map(|_| {
                cursor.read_exact(&mut pk3_buffer)?;
                Pokemon::from_pk3(&pk3_buffer)
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn get_box(&self, box_number: u8) -> io::Result<Vec<(u8, Pokemon)>> {
        let box_pokemon = (1..=30)
            .map(|slot| self.get_pokemon_from_box(box_number, slot))
            .collect::<io::Result<Vec<_>>>()?;
        Ok(box_pokemon
            .into_iter()
            .enumerate()
            .filter_map(|(idx, pkmn)| pkmn.map(|pkmn| (1 + idx as u8, pkmn)))
            .collect())
    }

    pub fn verify_sections(&self) -> io::Result<()> {
        for section_id in 0..NUMBER_OF_SECTIONS {
            let section_offset = self.get_offset_for_section(section_id) as usize;
            let section_data =
                &self.full_contents[section_offset..section_offset + SECTION_SIZE as usize];
            let checksum = compute_section_checksum(&section_data[..SECTION_DATA_SIZE])?;

            let mut cursor = Cursor::new(section_data);
            cursor.seek(SeekFrom::Start(SECTION_CHECKSUM_OFFSET))?;
            let actual_checksum = cursor.read_u16::<LittleEndian>()?;

            if checksum != actual_checksum {
                log::error!("Computed checksum 0x{checksum:x} for section {section_id}, but checksum was 0x{actual_checksum:x}");
                return Err(std::io::ErrorKind::InvalidData.into());
            }
        }

        Ok(())
    }

    fn recompute_checksums(&mut self) -> io::Result<()> {
        for section_id in 0..NUMBER_OF_SECTIONS {
            let section_offset = self.get_offset_for_section(section_id) as usize;
            let section_data =
                &mut self.full_contents[section_offset..section_offset + SECTION_SIZE as usize];
            let checksum = compute_section_checksum(&section_data[..SECTION_DATA_SIZE])?;

            let mut cursor = Cursor::new(section_data);
            cursor.seek(SeekFrom::Start(SECTION_CHECKSUM_OFFSET))?;
            cursor.write_u16::<LittleEndian>(checksum)?;
        }

        Ok(())
    }

    pub fn get_pokemon_from_box(
        &self,
        box_number: u8,
        slot_number: u8,
    ) -> io::Result<Option<Pokemon>> {
        // Some Pokemon data falls cleanly into a single memory section, some Pokemon data is
        // partitioned over multiple sections (with metadata in between and maybe wrapped
        // around thanks to the section rotation)
        log::trace!("Getting pokemon from box {box_number}-{slot_number}");

        let (section_id, relative_offset) =
            compute_section_id_and_offset_for_box_slot(box_number, slot_number).unwrap();
        let section_offset = self.get_offset_for_section(section_id) as usize;
        if relative_offset + pokemon::PK3_SIZE_BOX > SECTION_DATA_SIZE {
            log::debug!("Retrieving straddling PK3 at box {box_number} position {slot_number}");
            let start_section_id = section_id;
            let mut pk3_data = [0u8; pokemon::PK3_SIZE_BOX];

            // First read from the first section up until the end of the section data
            log::debug!("Straddling sections, first section id {start_section_id}");
            let section_offset = self.get_offset_for_section(start_section_id) as usize;
            let bytes_from_first_section = SECTION_DATA_SIZE - relative_offset;
            pk3_data[..bytes_from_first_section].copy_from_slice(
                &self.full_contents
                    [section_offset + relative_offset..section_offset + SECTION_DATA_SIZE],
            );

            // Next we grab the trailing part and copy that as well
            let bytes_from_next_section = pokemon::PK3_SIZE_BOX - bytes_from_first_section;
            let next_section_id = (start_section_id + 1) % NUMBER_OF_SECTIONS;
            log::debug!("Straddling sections, second section id {next_section_id}");
            let section_offset = self.get_offset_for_section(next_section_id) as usize;

            log::debug!("Copied {bytes_from_first_section} bytes, remaining {bytes_from_next_section} at offset {section_offset:x}");
            pk3_data[bytes_from_first_section..].copy_from_slice(
                &self.full_contents[section_offset..section_offset + bytes_from_next_section],
            );

            // Now we can check if there's even valid data here and attempt to parse
            if pk3_data.iter().any(|byte| *byte != 0x00) {
                log::trace!("Parsing PK3 from non-contiguous data");
                Ok(Some(Pokemon::from_pk3(&pk3_data[..])?))
            } else {
                Ok(None)
            }
        } else {
            log::debug!("Getting contiguous PK3 data from box {box_number} position {slot_number}");
            let pk3_offset = section_offset + relative_offset;
            let pk3_data = &self.full_contents[pk3_offset..pk3_offset + pokemon::PK3_SIZE_BOX];
            if pk3_data.iter().any(|byte| *byte != 0x00) {
                log::trace!("Parsing PK3 from contiguous data");
                Ok(Some(Pokemon::from_pk3(pk3_data)?))
            } else {
                Ok(None)
            }
        }
    }

    pub fn take_pokemon_from_box(
        &mut self,
        box_number: u8,
        slot_number: u8,
    ) -> io::Result<Option<Pokemon>> {
        log::trace!("Taking pokemon from box {box_number}-{slot_number}");
        let pkmn = self.get_pokemon_from_box(box_number, slot_number)?;
        self.clear_box_position(box_number, slot_number)?;
        self.recompute_checksums()?;
        Ok(pkmn)
    }

    fn clear_box_position(&mut self, box_number: u8, slot_number: u8) -> io::Result<()> {
        log::trace!("Clearing box position {box_number}-{slot_number}");
        let cleared_pk3 = [0u8; pokemon::PK3_SIZE_BOX];
        let _ = self.put_pokemon_in_box(box_number, slot_number, &cleared_pk3, true)?;
        Ok(())
    }

    pub fn put_pokemon_in_box(
        &mut self,
        box_number: u8,
        slot_number: u8,
        pk3_data: &[u8],
        force: bool,
    ) -> io::Result<bool> {
        if pk3_data.len() != pokemon::PK3_SIZE_BOX {
            log::error!(
                "Expected {}, got {} bytes for pk3 data format",
                pokemon::PK3_SIZE_BOX,
                pk3_data.len()
            );
            return Err(io::ErrorKind::InvalidInput.into());
        }

        if let Ok(pk3) = Pokemon::from_pk3(pk3_data) {
            self.mark_pokemon_owned_in_dex(pk3.species)?;
        }

        let (section_id, relative_offset) =
            compute_section_id_and_offset_for_box_slot(box_number, slot_number).unwrap();
        let section_offset = self.get_offset_for_section(section_id) as usize;

        if relative_offset + pokemon::PK3_SIZE_BOX > SECTION_DATA_SIZE {
            log::debug!(
                "This PK3 straddles a section {section_id} at section offset {section_offset}"
            );
            let bytes_from_first_section = SECTION_DATA_SIZE - relative_offset;
            let bytes_from_next_section = pokemon::PK3_SIZE_BOX - bytes_from_first_section;

            let pokemon_present = self.full_contents
                [section_offset + relative_offset..section_offset + SECTION_DATA_SIZE]
                .iter()
                .any(|byte| *byte != 0x00)
                || self.full_contents[section_offset..section_offset + bytes_from_next_section]
                    .iter()
                    .any(|byte| *byte != 0x00);
            if pokemon_present && !force {
                return Ok(false);
            }

            // First clear the first section up until the end of the section data
            self.full_contents
                [section_offset + relative_offset..section_offset + SECTION_DATA_SIZE]
                .copy_from_slice(&pk3_data[..bytes_from_first_section]);

            // Next we grab the trailing part and clear that as well
            let section_offset = self.get_offset_for_section(section_id + 1) as usize;
            self.full_contents[section_offset..section_offset + bytes_from_next_section]
                .copy_from_slice(&pk3_data[bytes_from_first_section..]);
            Ok(true)
        } else {
            let pk3_offset = section_offset + relative_offset;
            let existing_pk3_data =
                &mut self.full_contents[pk3_offset..pk3_offset + pokemon::PK3_SIZE_BOX];
            let pokemon_present = existing_pk3_data.iter().any(|byte| *byte != 0x00);

            if pokemon_present && !force {
                return Ok(false);
            }

            existing_pk3_data.copy_from_slice(pk3_data);
            Ok(true)
        }
    }

    fn parse_trainer_info(&self) -> io::Result<(TrainerInfo, GameCode)> {
        let section_offset = self.get_offset_for_section(0) as usize;
        let section_data =
            &self.full_contents[section_offset..section_offset + SECTION_SIZE as usize];
        let mut cursor = Cursor::new(section_data);

        let mut player_name = [0u8; 7];
        cursor.read_exact(&mut player_name)?;
        let _ = cursor.read_u8()?;
        let player_gender = determine_player_gender(cursor.read_u8()?)?;
        let _ = cursor.read_u8()?;
        let trainer_id = cursor.read_u32::<LittleEndian>()?;
        let trainer_id = TrainerId {
            public_id: (trainer_id & 0xffff) as u16,
            secret_id: (trainer_id >> 16) as u16,
        };
        let playtime = TimePlayed {
            hours: cursor.read_u16::<LittleEndian>()?,
            minutes: cursor.read_u8()?,
            seconds: cursor.read_u8()?,
            frames: cursor.read_u8()?,
        };

        cursor.seek(SeekFrom::Start(0xAC))?;
        let game_code = determine_game_code(cursor.read_u32::<LittleEndian>()?);

        Ok((
            TrainerInfo {
                player_name: decode_text(&player_name),
                player_gender,
                id: trainer_id,
                time_played: playtime,
            },
            game_code,
        ))
    }

    fn mark_pokemon_owned_in_dex(&mut self, species: Species) -> io::Result<()> {
        let bit_position = species.national_dex_number()? - 1;
        let byte_number = bit_position >> 3;
        let bit_position = bit_position & 0b111;

        let section_offset = self.get_offset_for_section(0);
        let pokedex_owned_offset = section_offset + self.game_code.unwrap().pokedex_owned();
        let pokedex_seen_a_offset = section_offset + self.game_code.unwrap().pokedex_seen_a();
        let section_offset = self.get_offset_for_section(1);
        let pokedex_seen_b_offset = section_offset + self.game_code.unwrap().pokedex_seen_b();
        let section_offset = self.get_offset_for_section(4);
        let pokedex_seen_c_offset = section_offset + self.game_code.unwrap().pokedex_seen_c();

        let pokedex_offsets = [
            pokedex_owned_offset,
            pokedex_seen_a_offset,
            pokedex_seen_b_offset,
            pokedex_seen_c_offset,
        ];

        let mut cursor = std::io::Cursor::new(&mut self.full_contents[..]);

        for offset in pokedex_offsets {
            cursor.set_position(offset + byte_number as u64);
            let mut current_byte = cursor.read_u8()?;
            current_byte |= 1 << bit_position;
            cursor.set_position(offset + byte_number as u64);
            cursor.write_u8(current_byte)?;
        }

        Ok(())
    }

    pub fn write_to_file(&mut self, filepath: impl AsRef<Path>) -> io::Result<()> {
        self.recompute_checksums()?;
        std::fs::write(filepath, &self.full_contents)
    }

    pub fn write_in_place(&mut self) -> io::Result<()> {
        let source_file = self.source.clone();
        self.write_to_file(source_file)
    }
}

fn determine_latest_game_save_offset(save_data: &[u8]) -> std::io::Result<u64> {
    let mut cursor = Cursor::new(save_data);
    cursor.seek(SeekFrom::Start(SAVE_A_OFFSET + SAVE_INDEX_OFFSET))?;
    let save_index_a = cursor.read_u32::<LittleEndian>()?;

    cursor.seek(SeekFrom::Start(SAVE_B_OFFSET + SAVE_INDEX_OFFSET))?;
    let save_index_b = cursor.read_u32::<LittleEndian>()?;

    log::debug!("Save Index A: {save_index_a}");
    log::debug!("Save Index B: {save_index_b}");

    let offset = if save_index_a == 0xffffffff {
        SAVE_B_OFFSET
    } else if save_index_b == 0xffffffff || save_index_a > save_index_b {
        SAVE_A_OFFSET
    } else {
        SAVE_B_OFFSET
    };

    Ok(offset)
}

fn determine_section_rotation(save_offset: u64, save_data: &[u8]) -> io::Result<u8> {
    let mut cursor = Cursor::new(save_data);
    cursor.seek(SeekFrom::Start(save_offset + 0x0ff4))?;
    let section_id = cursor.read_u16::<LittleEndian>()? as u8;
    let section_rotation = (NUMBER_OF_SECTIONS - section_id) % NUMBER_OF_SECTIONS;
    log::debug!("Current section rotation is {section_rotation}");
    Ok(section_rotation)
}

fn compute_section_checksum(data: &[u8]) -> io::Result<u16> {
    assert_eq!(data.len(), SECTION_DATA_SIZE);

    let mut checksum = 0u32;
    let mut cursor = Cursor::new(data);
    for _ in 0..(SECTION_DATA_SIZE / 4) {
        let next_dword = cursor.read_u32::<LittleEndian>()?;
        checksum = checksum.wrapping_add(next_dword);
    }

    let checksum_lower = (checksum & 0xffff) as u16;
    let checksum_upper = (checksum >> 16) as u16;
    Ok(checksum_upper.wrapping_add(checksum_lower))
}

fn determine_player_gender(data: u8) -> io::Result<PlayerGender> {
    if data == 0x00 {
        Ok(PlayerGender::Male)
    } else if data == 0x01 {
        Ok(PlayerGender::Female)
    } else {
        eprintln!("Invalid player gender: 0x{data:x}");
        return Err(std::io::ErrorKind::InvalidData.into());
    }
}

fn determine_game_code(data: u32) -> GameCode {
    if data == 0x00 {
        GameCode::RubySapphire
    } else if data == 0x01 {
        GameCode::FireRedLeafGreen
    } else {
        // For an Emerald save file, this is actually a security key field
        GameCode::Emerald
    }
}

fn compute_section_id_and_offset_for_box_slot(
    box_number: u8,
    box_entry: u8,
) -> Option<(u8, usize)> {
    let box_number = box_number as usize;
    let box_entry = box_entry as usize;
    if !(1..=16).contains(&box_number) || !(1..=30).contains(&box_entry) {
        eprintln!("Invalid box entry: {box_entry} in box number: {box_number}");
        return None;
    }

    let absolute_entry = ((box_number - 1) * 30) + (box_entry - 1);
    // Including the 4 bytes at the start of section 5 to make the math easier
    let absolute_offset = (absolute_entry * pokemon::PK3_SIZE_BOX) + 4;
    let section_id = 5 + (absolute_offset / SECTION_DATA_SIZE);
    let section_offset = absolute_offset % SECTION_DATA_SIZE;

    Some((section_id as u8, section_offset))
}
