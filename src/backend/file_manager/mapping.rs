use crate::backend::account_manager::account_server::{VaultInfo, JWT, SESSION_CACHE};
use crate::backend::aes_keys::crypted_key::{encrypt};
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::backend::aes_keys::decrypted_key::decrypt;

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
    original_filename: String,
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

impl FileMap {
    pub fn new(
        original_filename: String,
        binary: String,
        metadata: String,
        file_type: String,
    ) -> Self {
        Self {
            original_filename,
            binary,
            metadata,
            file_type,
        }
    }
}

impl FileTree {
    pub fn new(name: String, file_type: FileType) -> Self {
        Self { name, file_type }
    }
}

impl FrontFileMap {
    pub fn new(original_filename: String) -> Self {
        Self { original_filename }
    }
}

impl FrontFileTree {
    pub fn new(name: String, file_type: FrontFileType) -> Self {
        Self { name, file_type }
    }
}

impl FileType {
    pub fn new_file(
        original_filename: String,
        binary: String,
        metadata: String,
        file_type: String,
    ) -> Self {
        FileType::File(FileMap::new(original_filename, binary, metadata, file_type))
    }

    pub fn new_folder(files: Vec<FileTree>) -> Self {
        FileType::Folder(files)
    }
}

impl FrontFileType {
    pub fn new_file(original_filename: String) -> Self {
        FrontFileType::File(FrontFileMap::new(original_filename))
    }

    pub fn new_folder(files: Vec<FrontFileTree>) -> Self {
        FrontFileType::Folder(files)
    }
}

pub fn init_map(path: &str, key: &[u8]) {
    let folder = FileTree::new("/".to_string(), FileType::new_folder(Vec::new()));
    let content = serde_json::to_string(&folder).unwrap();
    let encrypted_content = encrypt(content.as_bytes(), key);
    fs::write(path, encrypted_content).unwrap();
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
    match &tree.file_type {
        FileType::File(file_map) => {
            FrontFileType::new_file(file_map.original_filename.clone())
        }
        FileType::Folder(children) => {
            let client_children = children
                .iter()
                .map(|child| {
                    FrontFileTree::new(child.name.clone(), server_map_to_client_map(child))
                })
                .collect();
            FrontFileType::new_folder(client_children)
        }
    }
}

pub async fn get_tree_vault(
    path: web::Path<String>,
    data: web::Json<(String, JWT, VaultInfo)>
) -> impl Responder {
    let vault_id = path.into_inner();
    let (vault_name, jwt, _vault_info) = data.into_inner();

    let map_path = Path::new("vaults")
        .join(vault_id)
        .join("map.json");

    if !map_path.exists() {
        return HttpResponse::NotFound().body("Vault not found");
    }

    let encrypted = match fs::read(&map_path) {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read map"),
    };

    let session_cache = SESSION_CACHE.lock().unwrap();
    let key = match session_cache.get(&jwt.session_id)
        .and_then(|cache| cache.vault_key.get(&vault_name)) {
        Some(k) => k,
        None => return HttpResponse::Unauthorized().body("Vault key not found in session"),
    };

    let decrypted = match decrypt(&encrypted, key) {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to decrypt map"),
    };

    let tree: FileTree = match serde_json::from_slice(&decrypted) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to parse map"),
    };

    let client_tree = server_map_to_client_map(&tree);

    HttpResponse::Ok().json(client_tree)
}
