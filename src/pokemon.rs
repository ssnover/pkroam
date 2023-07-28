use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    convert::Infallible,
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

use super::{decode_text, TrainerId};

pub const PK3_SIZE_PARTY: usize = 100;
pub const PK3_SIZE_BOX: usize = 80;
const SUBSTRUCTURE_OFFSET: u64 = 32;

#[derive(Clone, Copy, Debug)]
pub enum Language {
    Japanese,
    English,
    French,
    Italian,
    German,
    Spanish,
}

#[derive(Clone, Debug)]
pub struct Pokemon {
    pub source_data: Vec<u8>,
    pub personality_value: u32,
    pub original_trainer_id: TrainerId,
    pub nickname: String,
    pub origin_language: Language,
    pub original_trainer_name: String,
    pub species: u16,
    pub experience: u32,
    pub moves: [u16; 4],
    pub evs: [u8; 6],
    pub ivs: [u8; 6],
    pub is_egg: bool,
    pub ability: u8,
}

impl Pokemon {
    pub fn from_pk3(pk3: &[u8]) -> std::io::Result<Self> {
        let mut source_data = pk3.to_owned();
        encrypt_decrypt_pk3(&mut source_data[..]);

        let mut cursor = Cursor::new(&source_data[..]);
        let personality_value = cursor.read_u32::<LittleEndian>()?;
        let original_trainer_id = cursor.read_u32::<LittleEndian>()?;
        let public_id = (original_trainer_id & 0xffff) as u16;
        let secret_id = (original_trainer_id >> 16) as u16;
        let mut nickname = [0u8; 10];
        cursor.read_exact(&mut nickname)?;
        let nickname = decode_text(&nickname);
        let language = Language::try_from(cursor.read_u8()?)?;
        let _egg_data = EggData::try_from(cursor.read_u8()?).unwrap();
        let mut original_trainer_name = [0u8; 7];
        cursor.read_exact(&mut original_trainer_name)?;
        let original_trainer_name = decode_text(&original_trainer_name);
        let _markings = cursor.read_u8()?;
        let _checksum = cursor.read_u16::<LittleEndian>()?;
        let _ = cursor.read_u16::<LittleEndian>()?;

        let offset =
            get_offset_for_substructure(personality_value, Component::Growth) + SUBSTRUCTURE_OFFSET;
        cursor.seek(SeekFrom::Start(offset))?;
        let species = cursor.read_u16::<LittleEndian>()?;
        let _held_item_id = cursor.read_u16::<LittleEndian>()?;
        let experience = cursor.read_u32::<LittleEndian>()?;
        let _pp_bonuses = cursor.read_u8()?;
        let _friendship = cursor.read_u8()?;
        let _ = cursor.read_u16::<LittleEndian>()?;

        let offset = get_offset_for_substructure(personality_value, Component::Attacks)
            + SUBSTRUCTURE_OFFSET;
        cursor.seek(SeekFrom::Start(offset))?;
        let mut moves = [0u16; 4];
        (0..4).into_iter().for_each(|idx| {
            moves[idx] = cursor.read_u16::<LittleEndian>().unwrap();
        });
        let _pp = (0..4)
            .into_iter()
            .map(|_| cursor.read_u8().unwrap())
            .collect::<Vec<_>>();

        let offset = get_offset_for_substructure(personality_value, Component::EvsConditions)
            + SUBSTRUCTURE_OFFSET;
        cursor.seek(SeekFrom::Start(offset))?;
        let mut evs = [0u8; 6];
        (0..6)
            .into_iter()
            .for_each(|idx| evs[idx] = cursor.read_u8().unwrap());
        let _contest_stats = (0..6)
            .into_iter()
            .map(|_| cursor.read_u8().unwrap())
            .collect::<Vec<_>>();

        let offset = get_offset_for_substructure(personality_value, Component::Miscellaneous)
            + SUBSTRUCTURE_OFFSET;
        cursor.seek(SeekFrom::Start(offset))?;
        let _pokerus_status = cursor.read_u8()?;
        let _met_location = cursor.read_u8()?;
        let _origin_info = cursor.read_u16::<LittleEndian>()?;
        let ivs_egg_ability_blob = cursor.read_u32::<LittleEndian>()?;
        let mut ivs = [0u8; 6];
        (0..6)
            .into_iter()
            .for_each(|idx| ivs[idx] = ((ivs_egg_ability_blob >> (5 * idx)) & 0b11111) as u8);
        let is_egg = ((ivs_egg_ability_blob >> 30) & 0b1) != 0;
        let ability = ((ivs_egg_ability_blob >> 31) & 0b1) as u8;
        let _ribbons_obedience_data = cursor.read_u32::<LittleEndian>()?;

        let pkmn = Pokemon {
            source_data,
            personality_value,
            original_trainer_id: TrainerId {
                public_id,
                secret_id,
            },
            nickname,
            origin_language: language,
            original_trainer_name,
            species,
            experience,
            moves,
            evs,
            ivs,
            is_egg,
            ability,
        };
        Ok(pkmn)
    }

    pub fn to_pk3(mut self) -> Vec<u8> {
        encrypt_decrypt_pk3(&mut self.source_data);
        self.source_data
    }

    pub fn clear_evs(&mut self) {
        self.evs = [0u8; 6];
        let mut cursor = Cursor::new(&mut self.source_data[..]);
        cursor.set_position(32 + (2 * 12));
        cursor.write_all(&self.evs).unwrap();

        let new_checksum = compute_checksum(&self.source_data[32..80]);

        let mut cursor = Cursor::new(&mut self.source_data[..]);
        cursor.set_position(28);
        cursor.write_u16::<LittleEndian>(new_checksum).unwrap();
    }
}

enum Component {
    Growth,
    Attacks,
    EvsConditions,
    Miscellaneous,
}

fn get_offset_for_substructure(personality_value: u32, component: Component) -> u64 {
    const COMPONENT_SIZE: u64 = 12;
    match (component, personality_value % 24) {
        (Component::Growth, 0..=5) => 0,
        (Component::Growth, 6 | 7 | 12 | 13 | 18 | 19) => COMPONENT_SIZE,
        (Component::Growth, 8 | 10 | 14 | 16 | 20 | 22) => COMPONENT_SIZE * 2,
        (Component::Growth, 9 | 11 | 15 | 17 | 21 | 23) => COMPONENT_SIZE * 3,
        (Component::Attacks, 6..=11) => 0,
        (Component::Attacks, 0 | 1 | 14 | 15 | 20 | 21) => COMPONENT_SIZE,
        (Component::Attacks, 2 | 4 | 12 | 17 | 18 | 23) => COMPONENT_SIZE * 2,
        (Component::Attacks, 3 | 5 | 13 | 16 | 19 | 22) => COMPONENT_SIZE * 3,
        (Component::EvsConditions, 12..=17) => 0,
        (Component::EvsConditions, 2 | 3 | 8 | 9 | 22 | 23) => COMPONENT_SIZE,
        (Component::EvsConditions, 0 | 5 | 6 | 11 | 19 | 21) => COMPONENT_SIZE * 2,
        (Component::EvsConditions, 1 | 4 | 7 | 10 | 18 | 20) => COMPONENT_SIZE * 3,
        (Component::Miscellaneous, 18..=23) => 0,
        (Component::Miscellaneous, 4 | 5 | 10 | 11 | 16 | 17) => COMPONENT_SIZE,
        (Component::Miscellaneous, 1 | 3 | 7 | 9 | 13 | 15) => COMPONENT_SIZE * 2,
        (Component::Miscellaneous, 0 | 2 | 6 | 8 | 12 | 14) => COMPONENT_SIZE * 3,
        _ => unreachable!(),
    }
}

fn encrypt_decrypt_pk3(pk3_data: &mut [u8]) {
    let mut cursor = Cursor::new(&pk3_data);
    let personality_value = cursor.read_u32::<LittleEndian>().unwrap();
    let original_trainer_id = cursor.read_u32::<LittleEndian>().unwrap();
    let decryption_key = personality_value ^ original_trainer_id;
    let mut decryption_key_buf = [0u8; 4];
    LittleEndian::write_u32(&mut decryption_key_buf, decryption_key);

    // First XOR the key over the region
    for idx in (32..80).step_by(4) {
        for byte in 0..4 {
            pk3_data[idx + byte] ^= decryption_key_buf[byte];
        }
    }
}

fn compute_checksum(pk3_unencrypted_data_region: &[u8]) -> u16 {
    assert_eq!(pk3_unencrypted_data_region.len(), 80 - 32);
    let mut cursor = Cursor::new(pk3_unencrypted_data_region);

    let mut checksum = 0u16;

    for _ in 0..((80 - 32) / 2) {
        checksum = checksum.wrapping_add(cursor.read_u16::<LittleEndian>().unwrap());
    }

    checksum
}

impl TryFrom<u8> for Language {
    type Error = std::io::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Language::Japanese),
            2 => Ok(Language::English),
            3 => Ok(Language::French),
            4 => Ok(Language::Italian),
            5 => Ok(Language::German),
            7 => Ok(Language::Spanish),
            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }
}

pub struct EggData {
    _is_bad_egg: bool,
    _has_species: bool,
    _use_egg_name: bool,
}

impl TryFrom<u8> for EggData {
    type Error = Infallible;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(EggData {
            _is_bad_egg: (value & 0b1) != 0,
            _has_species: (value & 0b10) != 0,
            _use_egg_name: (value & 0b100) != 0,
        })
    }
}
