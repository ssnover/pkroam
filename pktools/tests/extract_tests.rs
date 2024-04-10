use pktools::extract;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

const EMERALD_SAV: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", "emerald.sav");
const EMERALD_MODIFIED_SAV: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", "emerald2.sav");
const WURMPLE_PK3: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", "wurmple.pk3");

#[test]
fn test_extract_pk3() {
    let input_save = create_temp_save(EMERALD_SAV);
    let expected_sav = std::fs::read(EMERALD_MODIFIED_SAV).unwrap();
    let wurmple_pk3 = std::fs::read(WURMPLE_PK3).unwrap();
    let wurmple_output = tempfile::NamedTempFile::new().unwrap();
    extract::run(extract::Opts {
        sav: PathBuf::from(input_save.path()),
        box_number: 1,
        slot: 1,
        dest: PathBuf::from(wurmple_output.path()),
    })
    .unwrap();

    let generated_pk3 = std::fs::read(wurmple_output.path()).unwrap();
    assert_eq!(generated_pk3, wurmple_pk3);

    let generated_sav = std::fs::read(input_save.path()).unwrap();
    assert_eq!(generated_sav.len(), expected_sav.len());
    for (idx, (generated, expected)) in generated_sav.into_iter().zip(expected_sav).enumerate() {
        assert_eq!(
            generated, expected,
            "SAV files did not match, starting at idx {}",
            idx
        );
    }
}

fn create_temp_save(save_path: impl AsRef<Path>) -> NamedTempFile {
    let mut save_file = std::fs::File::open(save_path).unwrap();
    let mut save_data = Vec::new();
    save_file.read_to_end(&mut save_data).unwrap();

    let mut temp_save_file = NamedTempFile::new().unwrap();
    temp_save_file.write_all(&save_data[..]).unwrap();
    temp_save_file.flush().unwrap();
    temp_save_file
}
