use rand::Rng;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{env, fs, io};

/**
 * Returns the file name as a string from a given file path.
 */
pub fn get_name<P: AsRef<Path>>(file_path: P) -> String {
    file_path
        .as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

/**
 * Generates a random binary file name with `.v` extension.
 */
pub fn binary_namegen() -> String {
    let mut rng = rand::rng();
    let id: u32 = rng.gen();
    format!("bin_{}.v", id)
}

/**
 * Reads all bytes from a file and returns them as a vector.
 */
pub fn read_bytes<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

/**
 * Saves byte contents into a new file in the `binary_files` directory.
 * Returns the generated file name.
 */
pub fn save_binary(contents: &[u8]) -> String {
    let name: String = binary_namegen();
    let mut file = File::create(
        std::env::current_dir()
            .unwrap()
            .join("binary_files")
            .join(format!("{}", name)),
    )
    .unwrap();
    file.write_all(contents).unwrap();
    name
}

/**
 * Deletes all `.v` binary files from the `binary_files` directory.
 * This is used to clean up temporary files generated during execution.
 */
pub fn clear_binary() {
    let path = std::env::current_dir().unwrap().join("binary_files");

    for bin in fs::read_dir(&path).unwrap() {
        let bin = bin.unwrap();
        let bin_path = bin.path();
        fs::remove_file(&bin_path).unwrap();
    }
}
