use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct FileMap {
    original_filename: String,
    binary: String,
    metadata: String,
    file_type: String,
}

pub fn update_map(original_filename: String, binary: String, metadata: String, file_type: String) {
    let map_path = Path::new("binary_files").join("map.json");
    let mut map: Vec<FileMap> = if map_path.exists() {
        let content = fs::read_to_string(&map_path).unwrap();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    map.push(FileMap {
        original_filename,
        binary,
        metadata,
        file_type,
    });

    let json = serde_json::to_string_pretty(&map).unwrap();
    fs::write(map_path, json).unwrap();
}