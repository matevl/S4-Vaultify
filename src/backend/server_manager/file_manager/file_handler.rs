use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::server_manager::account_manager::Perms;
use crate::backend::server_manager::file_manager::file_tree::*;
use crate::backend::server_manager::global_manager::{get_user_from_cookie, VAULTS_CACHE};
use crate::backend::server_manager::vault_manager::{load_vault, VaultInfo, VaultsCache};
use actix_multipart::Multipart;
use actix_web::{test, web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use rand::distr::Alphanumeric;
use rand::Rng;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::Path;

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
pub async fn upload_file_query(req: HttpRequest, mut payload: Multipart) -> impl Responder {
    use futures_util::TryStreamExt;
    use serde_json;

    let mut vault_info_opt: Option<VaultInfo> = None;
    let mut upload_path = String::new();
    let mut upload_file_name = String::new();
    let mut file_bytes = Vec::new();

    // Lecture du contenu du multipart/form-data
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap_or("");
        if name == "vault_info" {
            let mut buf = Vec::new();
            while let Some(chunk) = field.next().await {
                buf.extend_from_slice(&chunk.unwrap());
            }
            vault_info_opt = serde_json::from_slice::<VaultInfo>(&buf).ok();
        } else if name == "path" {
            let mut buf = Vec::new();
            while let Some(chunk) = field.next().await {
                buf.extend_from_slice(&chunk.unwrap());
            }
            upload_path = String::from_utf8(buf).unwrap_or_default();
        } else if name == "file" {
            if let Some(filename) = content_disposition.get_filename() {
                upload_file_name = filename.to_string();
            }
            while let Some(chunk) = field.next().await {
                file_bytes.extend_from_slice(&chunk.unwrap());
            }
        }
    }

    let vault_info = match vault_info_opt {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("Missing vault_info"),
    };

    // Authentification
    let jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

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

    // Génération du chemin du fichier cible
    let mut full_path = vault_info.get_path();
    if !upload_path.is_empty() {
        full_path = format!(
            "{}/{}",
            full_path.trim_end_matches('/'),
            upload_path.trim_start_matches('/')
        );
    }
    let file_path = format!("{}/{}", full_path.trim_end_matches('/'), upload_file_name);

    // Création récursive des dossiers si nécessaire
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to create parent dir: {e}"));
        }
    }

    // Ecriture du fichier chiffré
    let mut file = match fs::File::create(&file_path) {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError().body("Internal server error"),
    };

    let encrypted_content = encrypt(&file_bytes, &vault_key);
    if let Err(_) = file.write_all(&encrypted_content) {
        let _ = fs::remove_file(&file_path);
        return HttpResponse::InternalServerError().body("Internal server error");
    }

    // Ajout dans le file tree virtuel
    {
        let mut vault_cache = vault_cache.lock().unwrap();
        // Gère le cas racine (si jamais le path est vide)
        let path_in_tree = if upload_path.trim().is_empty() {
            ""
        } else {
            upload_path.trim_matches('/')
        };
        let parent_dir = match vault_cache
            .vault_file_tree
            .get_mut_directory_from_path(path_in_tree)
        {
            Ok(dir) => dir,
            Err(_) => return HttpResponse::NotFound().body("Invalid path in tree"),
        };
        parent_dir.add_file(
            &upload_file_name,
            upload_file_name.clone(),
            "File".to_string(),
        );
        if let Err(_) = vault_info.save_file_tree(
            vault_cache.vault_key.as_slice(),
            vault_cache.vault_file_tree.clone(),
        ) {
            return HttpResponse::InternalServerError().body("Failed to save file tree");
        }
    }

    HttpResponse::Ok().body("")
}
