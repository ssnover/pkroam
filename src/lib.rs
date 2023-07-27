pub mod pokemon;
pub mod save;

use pokemon::Pokemon;

#[derive(Clone, Copy, Debug)]
pub struct TrainerId {
    pub public_id: u16,
    pub secret_id: u16,
}

fn decode_text(text_data: &[u8]) -> String {
    let mut out_text = String::new();
    for byte in text_data {
        let decoded_char = match *byte {
            0xfa..=0xff => break,
            0xbb => 'A',
            0xbc => 'B',
            0xbd => 'C',
            0xbe => 'D',
            0xbf => 'E',
            0xc0 => 'F',
            0xc1 => 'G',
            0xc2 => 'H',
            0xc3 => 'I',
            0xc4 => 'J',
            0xc5 => 'K',
            0xc6 => 'L',
            0xc7 => 'M',
            0xc8 => 'N',
            0xc9 => 'O',
            0xca => 'P',
            0xcb => 'Q',
            0xcc => 'R',
            0xcd => 'S',
            0xce => 'T',
            0xcf => 'U',
            0xd0 => 'V',
            0xd1 => 'W',
            0xd2 => 'X',
            0xd3 => 'Y',
            0xd4 => 'Z',
            0xd5 => 'a',
            0xd6 => 'b',
            0xd7 => 'c',
            0xd8 => 'd',
            0xd9 => 'e',
            0xda => 'f',
            0xdb => 'g',
            0xdc => 'h',
            0xdd => 'i',
            0xde => 'j',
            0xdf => 'k',
            0xe0 => 'l',
            0xe1 => 'm',
            0xe2 => 'n',
            0xe3 => 'o',
            0xe4 => 'p',
            0xe5 => 'q',
            0xe6 => 'r',
            0xe7 => 's',
            0xe8 => 't',
            0xe9 => 'u',
            0xea => 'v',
            0xeb => 'w',
            0xec => 'x',
            0xed => 'y',
            0xee => 'z',
            _ => '*',
        };
        out_text.push(decoded_char);
    }

    out_text
}
