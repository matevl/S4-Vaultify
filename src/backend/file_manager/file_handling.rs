use rand::Rng;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{fs, io};

pub fn binary_namegen() -> String {
    let mut rng = rand::rng();
    let id: u32 = rng.gen();
    format!("bin_{}.v", id)
}

pub fn read_bytes<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn save_binary(contents: &[u8]) {
    let mut file = File::create(
        std::env::current_dir()
            .unwrap()
            .join("binary_files")
            .join(format!("{}", binary_namegen())),
    )
    .unwrap();
    file.write_all(contents).unwrap();
}

pub fn clear_binary() {
    let path = std::env::current_dir().unwrap().join("binary_files");

    for bin in fs::read_dir(&path).unwrap() {
        let bin = bin.unwrap();
        let bin_path = bin.path();
        fs::remove_file(&bin_path).unwrap();
    }
}
