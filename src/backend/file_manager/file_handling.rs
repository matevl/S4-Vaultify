use rand::Rng;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{env, fs, io};
use crate::backend::file_manager::metadata_handling::detect_type;

pub fn get_name<P: AsRef<Path>>(file_path: P) ->String{
    file_path.as_ref().file_name().unwrap().to_str().unwrap().to_string()
}
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

pub fn save_binary(contents: &[u8])->String {
    let name:String= binary_namegen();
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

pub fn clear_binary() {
    let path = std::env::current_dir().unwrap().join("binary_files");

    for bin in fs::read_dir(&path).unwrap() {
        let bin = bin.unwrap();
        let bin_path = bin.path();
        fs::remove_file(&bin_path).unwrap();
    }
}

//pub fn refusion<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<u8>> {
//    let buffer = read_bytes(file_path)?;
//    let file_type = detect_type(&buffer);
//    if format!("{:?}", file_type).to_lowercase().contains("heic")
//    {
//        let output_path = env::current_dir()?.join("output.heic");
//        println!(
//            "DEBUG: Detected HEIC image. Writing output to {:?}",
//            output_path
//        );
//        fs::write(&output_path, &buffer)?;
//    } else {
//        println!("DEBUG: File is not a HEIC image; skipping HEIC output generation.");
//    }
//
//    if format!("{:?}", file_type).to_lowercase().contains("heif")
//    {
//        let output_path = env::current_dir()?.join("output.heif");
//        println!(
//            "DEBUG: Detected HEIF image. Writing output to {:?}",
//            output_path
//        );
//        fs::write(&output_path, &buffer)?;
//    } else {
//        println!("DEBUG: File is not a HEIF image; skipping HEIF output generation.");
//    }

//}