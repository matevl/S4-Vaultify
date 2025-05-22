use crate::backend::server_manager::global_manager::{get_user_from_cookie, VAULTS_CACHE};
use crate::backend::server_manager::vault_manager::{load_vault, VaultInfo};
use actix_web::{test, web, HttpRequest, HttpResponse, Responder};
use rusqlite::fallible_iterator::FallibleIterator;

pub async fn get_file_tree_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
) -> impl Responder {
    let jwt = match get_user_from_cookie(&req) {
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
