use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use actix_web::{web, HttpResponse, Responder};
use crate::backend::account_manager::account_server::{VaultInfo, JWT};

#[derive(Serialize, Deserialize, Debug)]
struct FileMap {
    original_filename: String,
    binary: String,
    metadata: String,
    file_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum FileType {
    File(FileMap),
    Folder(Vec<FileTree>),
}

#[derive(Serialize, Deserialize, Debug)]
struct FileTree {
    name: String,
    file_type: FileType,
}

#[derive(Serialize, Deserialize, Debug)]
struct FrontFileMap {
    binary: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum FrontFileType {
    File(FrontFileMap),
    Folder(Vec<FrontFileTree>),
}

#[derive(Serialize, Deserialize, Debug)]
struct FrontFileTree {
    name: String,
    file_type: FrontFileType,
}

/**
 * Updates the file mapping JSON by appending a new entry.
 * Stores original filename, binary name, metadata path, and file type.
 */
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
    fs::write(map_path, json).unwrap(); //le rendre en dictionnaire.
}

pub fn server_map_to_client_map(tree: &FileTree) -> FrontFileType {
    todo!()
}

pub async fn get_tree_vault(data: web::Json<(JWT, VaultInfo)>) -> impl Responder {
    HttpResponse::Ok().json("")
}