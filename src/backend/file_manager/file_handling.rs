use rand::Rng;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn binary_namegen() -> String {
    let mut rng = rand::thread_rng();
    let id: u32 = rng.gen();
    format!("bin_{}.v", id)
}

pub fn open_file_binary(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    contents
}

pub fn save_binary(contents: &[u8]) {
    let file_name = binary_namegen();
    let file_path = std::env::current_dir()
        .unwrap()
        .join("binary_files")
        .join(file_name);
    let mut file = File::create(file_path).unwrap();

    file.write_all(contents).unwrap();
}
