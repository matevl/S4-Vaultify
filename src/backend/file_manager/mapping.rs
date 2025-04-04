use crate::backend::account_manager::account_server::{VaultInfo, JWT};
use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::file_manager::mapping::FileType::Folder;
use actix_web::{web, HttpResponse, Responder};
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
    pub fn new(binary: String) -> Self {
        Self { binary }
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
    pub fn new_file(binary: String) -> Self {
        FrontFileType::File(FrontFileMap::new(binary))
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
    todo!()
}

pub async fn get_tree_vault(data: web::Json<(JWT, VaultInfo)>) -> impl Responder {
    HttpResponse::Ok().json("")
}

fn convert_to_front(tree: &FileTree) -> FrontFileTree {
    match &tree.file_type {
        FileType::File(file_map) => {
            FrontFileTree::new(
                tree.name.clone(),
                FrontFileType::File(FrontFileMap::new(file_map.binary.clone())),
            )
        }
        FileType::Folder(children) => {
            let converted_children = children.iter().map(convert_to_front).collect();
            FrontFileTree::new(tree.name.clone(), FrontFileType::Folder(converted_children))
        }
    }
}

fn add_file_at_path(
    tree: &mut FileTree,
    path: &[&str],
    new_file: FileTree,
) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path cannot be empty".into());
    }

    let mut current = tree;
    for (i, part) in path.iter().enumerate() {
        match &mut current.file_type {
            FileType::Folder(children) => {
                if i == path.len() - 1 {
                    // Dernier niveau — on ajoute
                    children.push(new_file);
                    return Ok(());
                } else {
                    // On descend dans l’arborescence
                    if let Some(next) = children.iter_mut().find(|c| c.name == *part) {
                        current = next;
                    } else {
                        return Err(format!("Folder '{}' not found", part));
                    }
                }
            }
            FileType::File(_) => return Err("Cannot navigate into a file".into()),
        }
    }

    Err("Failed to add file".into())
}

fn delete_file_at_path(tree: &mut FileTree, path: &[&str]) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path cannot be empty".into());
    }

    let mut current = tree;
    for (i, part) in path.iter().enumerate() {
        match &mut current.file_type {
            FileType::Folder(children) => {
                if i == path.len() - 1 {
                    let index = children.iter().position(|c| c.name == *part);
                    if let Some(idx) = index {
                        children.remove(idx);
                        return Ok(());
                    } else {
                        return Err(format!("File or folder '{}' not found", part));
                    }
                } else {
                    if let Some(next) = children.iter_mut().find(|c| c.name == *part) {
                        current = next;
                    } else {
                        return Err(format!("Folder '{}' not found", part));
                    }
                }
            }
            FileType::File(_) => return Err("Cannot navigate into a file".into()),
        }
    }

    Err("Failed to delete file".into())
}

fn get_file_at_path_mut<'a>(tree: &'a mut FileTree, path: &[&str]) -> Option<&'a mut FileTree> {
    let mut current = tree;
    for part in path {
        match &mut current.file_type {
            FileType::Folder(children) => {
                if let Some(next) = children.iter_mut().find(|c| c.name == *part) {
                    current = next;
                } else {
                    return None; // Fichier/Dossier introuvable
                }
            }
            FileType::File(_) => return None, // Impossible de naviguer dans un fichier
        }
    }
    Some(current)
}