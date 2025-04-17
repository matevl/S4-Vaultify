use crate::backend::account_manager::account_server::{VaultInfo, JWT, ROOT, SESSION_CACHE};
use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::aes_keys::decrypted_key::decrypt;
use crate::backend::{VAULTS_DATA, VAULT_USERS_DIR};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
        FileType::File(file_map) => FrontFileType::new_file(file_map.original_filename.clone()),
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

pub async fn get_tree_vault(req: HttpRequest, info: web::Json<VaultInfo>) -> impl Responder {
    // 1) extract and decode JWT from the HttpOnly cookie
    let token = match req.cookie("user_token") {
        Some(c) => c.value().to_string(),
        None => return HttpResponse::Unauthorized().body("Not authenticated"),
    };
    let secret = "test"; // your secret
    let jwt = match JWT::decode(&token, secret) {
        Some(j) => j,
        None => return HttpResponse::Unauthorized().body("Invalid token"),
    };
    let vault_info = info.into_inner();

    let vault_name = format!("{}_{}", vault_info.user_id, vault_info.date);

    let mut vault_path = format!("{}/{}{}/", ROOT.to_str().unwrap(), VAULTS_DATA, vault_name);

    let mut map_path = vault_path.clone();
    map_path.push_str("map.json");

    let encrypted = match fs::read(&map_path) {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read map"),
    };

    if let Some(cache) = SESSION_CACHE.lock().unwrap().get_mut(&jwt.session_id) {
        let key = cache.user_key.clone();

        let decrypted = match decrypt(&encrypted, key.as_slice()) {
            Ok(data) => data,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to decrypt map"),
        };

        let tree: FileTree = match serde_json::from_slice(&decrypted) {
            Ok(t) => t,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to parse map"),
        };

        let client_tree = server_map_to_client_map(&tree);

        HttpResponse::Ok().json(client_tree)
    } else {
        HttpResponse::InternalServerError().body("Failed to get vault")
    }
}


fn extract_node(tree: &mut FileTree, path: &[&str]) -> Option<FileTree> { //&[&str] for reference to slice
    if path.is_empty() {
        return None;
    }

    if let FileType::Folder(children) = &mut tree.file_type {
        if path.len() == 1 {
            let name = path[0];
            let index = children.iter().position(|c| c.name == name)?;
            return Some(children.remove(index));
        }

        let next = path[0];
        let child = children.iter_mut().find(|c| c.name == next)?;
        return extract_node(child, &path[1..]);
    }

    None
}

fn insert_node(tree: &mut FileTree, path: &[&str], node: FileTree) -> bool {
    if let FileType::Folder(children) = &mut tree.file_type {
        if path.is_empty() {
            children.push(node);
            return true;
        }

        let next = path[0];
        if let Some(child) = children.iter_mut().find(|c| c.name == next) {
            return insert_node(child, &path[1..], node);
        }
    }

    false
}

pub fn move_in_map(from_path: &str, to_path: &str) {
    let map_path = Path::new("binary_files").join("map.json");
    let content = fs::read_to_string(&map_path).expect("Failed to read map.json");
    let mut root: FileTree = serde_json::from_str(&content).expect("Invalid map.json");

    let from_segments: Vec<&str> = from_path.trim_matches('/').split('/').collect();
    let node = extract_node(&mut root, &from_segments).expect("Invalid source path");

    let to_segments: Vec<&str> = to_path.trim_matches('/').split('/').collect();
    let success = insert_node(&mut root, &to_segments, node);
    assert!(success, "Invalid destination path");

    let new_content = serde_json::to_string_pretty(&root).unwrap();
    fs::write(&map_path, new_content).expect("Failed to write map.json");
}