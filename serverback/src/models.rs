use actix_web::{HttpResponse, Responder, web};
use s4_vaultify::backend::account_manager;
use s4_vaultify::backend::account_manager::account::UserData;
use serde::Serialize;

// Fonction asynchrone qui renvoie un JSON avec les informations de l'utilisateur
pub async fn get_me(user_data: UserData) -> impl Responder {
    web::Json(user_data) // Retourne un objet JSON comme réponse
}
