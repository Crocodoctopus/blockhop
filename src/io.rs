use bincode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

lazy_static! {
    static ref ROOT: PathBuf = { PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()) };
}

pub fn get_root() -> PathBuf {
    ROOT.clone()
}

pub fn save_to_file<T: Serialize>(t: T, filename: &Path) {
    let mut file = File::create(ROOT.join(filename)).expect("Could not open file");
    let serialized_data: Vec<u8> = serialize(&t).unwrap();
    file.write(&serialized_data).unwrap();
}

pub fn load_from_file<T: DeserializeOwned>(filename: &Path) -> T {
    let mut file = File::open(ROOT.join(filename)).expect("Could not open file");
    let mut serialized_data = Vec::<u8>::new();
    file.read_to_end(&mut serialized_data).unwrap();
    let data = deserialize(&serialized_data).unwrap();
    return data;
}
