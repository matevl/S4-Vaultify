use crate::backend::server_manager::file_manager::file_tree::*;
use crate::backend::server_manager::global_manager::{get_user_from_cookie, VAULTS_CACHE};
use crate::backend::server_manager::vault_manager::{load_vault, VaultInfo, VaultsCache};
use actix_web::{test, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

pub async fn get_file_tree_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
) -> impl Responder {
    let _ = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let vault_info = vault_info.into_inner();
    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::Unauthorized().finish();
    }

    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(cache) => cache,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let vault_cache = cache.lock().unwrap();

    match vault_info.get_file_tree(vault_cache.vault_key.as_slice()) {
        Ok(tree) => HttpResponse::Ok().json(tree),
        Err(_) => HttpResponse::Unauthorized().finish(),
    }
}

/// Payload for creating a new folder
#[derive(Deserialize)]
pub struct CreateFolderPayload {
    /// Full path where the new folder should be created (e.g. "Documents/Projects")
    path: String,
    /// Name of the new folder to create
    name: String,
}

/// Handler to create a new folder at the specified path
pub async fn create_folder_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
    payload: web::Json<CreateFolderPayload>,
) -> impl Responder {
    // Authenticate user from cookie
    let jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let vault_info = vault_info.into_inner();

    // Load the vault if needed
    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    // Access vault cache
    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(cache) => cache,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };
    let mut vault_cache = cache.lock().unwrap();

    // Navigate to the target directory using path
    let target_dir = match vault_cache
        .vault_file_tree
        .get_mut_directory_from_path(&payload.path)
    {
        Ok(dir) => dir,
        Err(_) => return HttpResponse::NotFound().body("Invalid path"),
    };

    // Add the new folder
    target_dir.add_dir(&payload.name);

    // Return updated public representation
    HttpResponse::Ok().json(target_dir.to_public())
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

/// Handler to rename a file or folder
pub async fn rename_item_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
    payload: web::Json<RenamePayload>,
) -> impl Responder {
    // Authenticate user
    let jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let vault_info = vault_info.into_inner();

    // Load vault
    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    // Access vault cache
    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(cache) => cache,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let mut vault_cache = cache.lock().unwrap();
    // Navigate to the directory containing the item
    let target_dir = match vault_cache
        .vault_file_tree
        .get_mut_directory_from_path(&payload.path)
    {
        Ok(dir) => dir,
        Err(_) => return HttpResponse::NotFound().body("Invalid path"),
    };

    // Rename the item
    match target_dir.rename(&payload.old_name, &payload.new_name) {
        Ok(_) => HttpResponse::Ok().json(target_dir.to_public()),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}

/// Payload for removing a folder (virtual only)
#[derive(Deserialize)]
pub struct RemoveFolderPayload {
    /// Path to the parent directory
    path: String,
    /// Name of the folder to remove
    folder_name: String,
}

/// Handler to remove a folder (recursively)
pub async fn remove_folder_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
    payload: web::Json<RemoveFolderPayload>,
) -> impl Responder {
    let jwt = match get_user_from_cookie(&req) {
        Some(_) => (),
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let vault_info = vault_info.into_inner();
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
        Ok(_) => HttpResponse::Ok().json(parent_dir.to_public()),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

/// Payload for removing a file
#[derive(Deserialize)]
pub struct RemoveFilePayload {
    /// Path to the parent directory
    path: String,
    /// Name of the file to remove
    file_name: String,
}

/// Handler to remove a file
pub async fn remove_file_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
    payload: web::Json<RemoveFilePayload>,
) -> impl Responder {
    let jwt = match get_user_from_cookie(&req) {
        Some(_) => (),
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let vault_info = vault_info.into_inner();
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
        Ok(_) => HttpResponse::Ok().json(parent_dir.to_public()),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}
