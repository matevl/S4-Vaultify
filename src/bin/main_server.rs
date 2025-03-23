use actix_web::{web, App, HttpServer};
use s4_vaultify::backend::account_manager::account_server::{create_user_query, load_vault_matching, load_vault_query, login_user_query};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
        println!("Starting server on port {}", port);
    HttpServer::new(move || {

        App::new()
            .service(create_user_query)
            .service(login_user_query)
    })
        .bind(format!("127.0.0.1:{}", port))?
        .workers(2)
        .run()
        .await
}