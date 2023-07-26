use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::io::{Cursor, Seek, SeekFrom};

pub const PK3_SIZE: usize = 100;

pub struct Pokemon {
    pub source_data: Vec<u8>,
    pub species: u16,
}

impl Pokemon {
    pub fn from_pk3(pk3: &[u8]) -> std::io::Result<Self> {
        let mut source_data = pk3.to_owned();
        decrypt_pk3(&mut source_data[..]);

        let mut cursor = Cursor::new(&source_data[..]);

        cursor.seek(SeekFrom::Start(32))?;
        let species = cursor.read_u16::<LittleEndian>()?;

        let pkmn = Pokemon {
            source_data,
            species,
        };
        Ok(pkmn)
    }
}

fn decrypt_pk3(pk3_data: &mut [u8]) {
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

    // Now rearrange the elements
    let mut rearranged_data = Vec::with_capacity(48);
    // Growth
    match personality_value % 24 {
        0..=5 => rearranged_data.extend_from_slice(&pk3_data[32..44]),
        6 | 7 | 12 | 13 | 18 | 19 => rearranged_data.extend_from_slice(&pk3_data[44..56]),
        8 | 10 | 14 | 16 | 20 | 22 => rearranged_data.extend_from_slice(&pk3_data[56..68]),
        9 | 11 | 15 | 17 | 21 | 23 => rearranged_data.extend_from_slice(&pk3_data[68..80]),
        24u32..=u32::MAX => unreachable!(),
    };

    // Attacks
    match personality_value % 24 {
        6..=11 => rearranged_data.extend_from_slice(&pk3_data[32..44]),
        0 | 1 | 14 | 15 | 20 | 21 => rearranged_data.extend_from_slice(&pk3_data[44..56]),
        2 | 4 | 12 | 17 | 18 | 23 => rearranged_data.extend_from_slice(&pk3_data[56..68]),
        3 | 5 | 13 | 16 | 19 | 22 => rearranged_data.extend_from_slice(&pk3_data[68..80]),
        _ => unimplemented!(),
    };

    // EVs and Condition
    match personality_value % 24 {
        12..=17 => rearranged_data.extend_from_slice(&pk3_data[32..44]),
        2 | 3 | 8 | 9 | 22 | 23 => rearranged_data.extend_from_slice(&pk3_data[44..56]),
        0 | 5 | 6 | 11 | 19 | 21 => rearranged_data.extend_from_slice(&pk3_data[56..68]),
        1 | 4 | 7 | 10 | 18 | 20 => rearranged_data.extend_from_slice(&pk3_data[68..80]),
        _ => unimplemented!(),
    };

    // Miscellaneous
    match personality_value % 24 {
        18..=23 => rearranged_data.extend_from_slice(&pk3_data[32..44]),
        4 | 5 | 10 | 11 | 16 | 17 => rearranged_data.extend_from_slice(&pk3_data[44..56]),
        1 | 3 | 7 | 9 | 13 | 15 => rearranged_data.extend_from_slice(&pk3_data[56..68]),
        0 | 2 | 6 | 8 | 12 | 14 => rearranged_data.extend_from_slice(&pk3_data[68..80]),
        _ => unimplemented!(),
    };

    pk3_data[32..80].clone_from_slice(&rearranged_data[..(80 - 32)])
}
