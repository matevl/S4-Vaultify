mod models;


use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use models::*;

use s4_vaultify::backend::*;
use s4_vaultify::backend::account_manager::account::{load_users_data, local_log_in, UserInput};
use s4_vaultify::backend::vault_manager::init_config_vaultify;
#[actix_web::main]
async fn main(jwt = ) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())  // Log les requêtes pour le débogage
            .route("/me", web::get().to(get_me))  // Route pour obtenir les infos du compte personnel
            .route("/me/Vaults", web::get().to(show_vaults))
    })
        .bind("127.0.0.1:8081")?  // L'adresse du serveur
        .run()
        .await
}