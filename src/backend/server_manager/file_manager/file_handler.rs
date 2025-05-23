use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::server_manager::account_manager::Perms;
use crate::backend::server_manager::file_manager::file_tree::*;
use crate::backend::server_manager::global_manager::{get_user_from_cookie, VAULTS_CACHE};
use crate::backend::server_manager::vault_manager::{load_vault, VaultInfo, VaultsCache};
use actix_web::{test, web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use rand::distr::Alphanumeric;
use rand::Rng;
use serde::Deserialize;
use std::fs;
use std::io::Write;

const BUFFER_SIZE: usize = 4096;

pub async fn get_file_tree_query(req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let _jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let vault_id = path.into_inner();

    let cache = match VAULTS_CACHE.get(&vault_id) {
        Some(cache) => cache,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let vault_cache = cache.lock().unwrap();

    let public_tree = vault_cache.vault_file_tree.to_public();
    HttpResponse::Ok().json(public_tree)
}

/// Payload for creating a new folder
#[derive(Deserialize)]
pub struct CreateFolderPayload {
    /// Full path where the new folder should be created (e.g. "Documents/Projects")
    path: String,
    /// Name of the new folder to create
    name: String,
}

#[derive(Deserialize)]
pub struct CreateFolderRequest {
    pub vault_info: VaultInfo,
    pub path: String,
    pub name: String,
}

/// Handler to create a new folder at the specified path
pub async fn create_folder_query(
    req: HttpRequest,
    data: web::Json<CreateFolderRequest>,
) -> impl Responder {
    let _jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let vault_info = data.vault_info.clone();

    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(cache) => cache,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };
    let mut vault_cache = cache.lock().unwrap();

    let target_dir = match vault_cache
        .vault_file_tree
        .get_mut_directory_from_path(&data.path)
    {
        Ok(dir) => dir,
        Err(_) => return HttpResponse::NotFound().body("Invalid path"),
    };

    target_dir.add_dir(&data.name);

    match vault_info.save_file_tree(
        vault_cache.vault_key.as_slice(),
        vault_cache.vault_file_tree.clone(),
    ) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Payload for renaming a file or directory
#[derive(Deserialize)]
pub struct RenamePayload {
    /// Path to the parent directory of the item
    path: String,
    /// Current name of the item to rename
    old_name: String,
    /// New name for the item
    new_name: String,
}

#[derive(Deserialize)]
pub struct RenameRequest {
    pub vault_info: VaultInfo,
    pub path: String,
    pub old_name: String,
    pub new_name: String,
}

/// Handler to rename a file or folder
pub async fn rename_item_query(
    req: HttpRequest,
    payload: web::Json<RenameRequest>,
) -> impl Responder {
    // Authenticate user
    let _jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // On clone pour garder la valeur locale utilisable
    let vault_info = payload.vault_info.clone();

    // Load vault (nécessaire pour valider l'accès utilisateur)
    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    // Accès au cache du vault
    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(cache) => cache,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let mut vault_cache = cache.lock().unwrap();

    // Trouve le dossier parent contenant l'élément à renommer
    let target_dir = match vault_cache
        .vault_file_tree
        .get_mut_directory_from_path(&payload.path)
    {
        Ok(dir) => dir,
        Err(_) => return HttpResponse::NotFound().body("Invalid path"),
    };

    // Rename
    match target_dir.rename(&payload.old_name, &payload.new_name) {
        Ok(_) => {}
        Err(err) => return HttpResponse::BadRequest().body(err),
    }

    // Sauvegarde l'arborescence modifiée
    match vault_info.save_file_tree(
        vault_cache.vault_key.as_slice(),
        vault_cache.vault_file_tree.clone(),
    ) {
        Ok(_) => HttpResponse::Ok().json(vault_cache.vault_file_tree.to_public()),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Payload for removing a folder (virtual only)
#[derive(Deserialize)]
pub struct RemoveFolderRequest {
    pub vault_info: VaultInfo,
    pub path: String,
    pub folder_name: String,
}


/// Handler to remove a folder (recursively)
pub async fn remove_folder_query(
    req: HttpRequest,
    payload: web::Json<RemoveFolderRequest>,
) -> impl Responder {
    let vault_info = payload.vault_info.clone();

    let _jwt = match get_user_from_cookie(&req) {
        Some(_) => (),
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let mut vault_cache = cache.lock().unwrap();

    let parent_dir = match vault_cache
        .vault_file_tree
        .get_mut_directory_from_path(&payload.path)
    {
        Ok(d) => d,
        Err(_) => return HttpResponse::NotFound().body("Invalid path"),
    };

    match remove_directory_recursively(parent_dir, &payload.folder_name, &vault_info.get_path()) {
        Ok(_) => {}
        Err(e) => return HttpResponse::InternalServerError().body(e),
    }

    match vault_info.save_file_tree(
        vault_cache.vault_key.as_slice(),
        vault_cache.vault_file_tree.clone(),
    ) {
        Ok(_) => HttpResponse::Ok().json(vault_cache.vault_file_tree.to_public()),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Payload for removing a file
#[derive(Deserialize)]
pub struct RemoveFileRequest {
    pub vault_info: VaultInfo,
    pub path: String,
    pub file_name: String,
}

/// Handler to remove a file

pub async fn remove_file_query(
    req: HttpRequest,
    payload: web::Json<RemoveFileRequest>,
) -> impl Responder {
    let vault_info = payload.vault_info.clone();

    let _jwt = match get_user_from_cookie(&req) {
        Some(_) => (),
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let mut vault_cache = cache.lock().unwrap();

    let parent_dir = match vault_cache
        .vault_file_tree
        .get_mut_directory_from_path(&payload.path)
    {
        Ok(d) => d,
        Err(_) => return HttpResponse::NotFound().body("Invalid path"),
    };

    match remove_file_from_directory(parent_dir, &payload.file_name, &vault_info.get_path()) {
        Ok(_) => {}
        Err(e) => return HttpResponse::InternalServerError().body(e),
    }

    match vault_info.save_file_tree(
        vault_cache.vault_key.as_slice(),
        vault_cache.vault_file_tree.clone(),
    ) {
        Ok(_) => HttpResponse::Ok().json(vault_cache.vault_file_tree.to_public()),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// upload files to the server
pub async fn upload_file_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
    mut payload: web::Payload,
) -> impl Responder {
    let jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let vault_info = vault_info.into_inner();
    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let vault_cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(cache) => cache,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let vault_key = {
        let vault_cache = vault_cache.lock().unwrap();
        if vault_cache.vault_key.is_empty() {
            return HttpResponse::Unauthorized().body("Unauthorized");
        }
        if !vault_cache.perms.contains_key(&jwt.id)
            || vault_cache.perms.get(&jwt.id).unwrap() < &Perms::Write
        {
            return HttpResponse::Unauthorized().body("Unauthorized");
        }
        vault_cache.vault_key.clone()
    };

    let vault_path = vault_info.get_path();
    let file_path = loop {
        let file_name: Vec<u8> = rand::rng().sample_iter(&Alphanumeric).take(16).collect();
        let name = String::from_utf8(file_name).unwrap();
        if !std::path::Path::new(&vault_path).exists() {
            return HttpResponse::NotFound().body("File not found");
        }
        if !std::path::Path::new(&vault_path).exists() {
            return HttpResponse::NotFound().body("File not found");
        }

        let file_path = format!("{}{}", vault_path, name);
        if !std::path::Path::new(&file_path).exists() {
            break file_path;
        }
    };

    let mut file = match fs::File::create(&file_path) {
        Ok(file) => file,
        Err(_) => return HttpResponse::InternalServerError().body("Internal server error"),
    };

    let mut buffer = Vec::with_capacity(BUFFER_SIZE);

    while let Some(chunk) = payload.next().await {
        let data = chunk.unwrap();

        for byte in data {
            buffer.push(byte);
            if buffer.len() == BUFFER_SIZE {
                let encrypted_content = encrypt(buffer.as_slice(), &vault_key);

                match file.write_all(&encrypted_content) {
                    Ok(_) => (),
                    Err(_) => {
                        fs::remove_file(&file_path).unwrap();
                        return HttpResponse::InternalServerError().body("Internal server error");
                    }
                }

                buffer.clear();
            }
        }
    }

    if !buffer.is_empty() {
        let encrypted_content = encrypt(buffer.as_slice(), &vault_key);
        match file.write_all(&encrypted_content) {
            Ok(_) => (),
            Err(_) => {
                fs::remove_file(&file_path).unwrap();
                return HttpResponse::InternalServerError().body("Internal server error");
            }
        }
    }

    HttpResponse::Ok().body("")
}
