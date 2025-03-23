use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_session::{CookieSession, Session};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct UserJson {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct JWT {
    id: u32,
    email: String,
}

type UsersData = HashMap<String, (String, u32)>;

lazy_static! {
    static ref USERS_DB: Arc<Mutex<UsersData>> = Arc::new(Mutex::new(HashMap::new()));
}

#[actix_web::post("/user/register")]
async fn create_user_query(user: web::Json<UserJson>, session: Session) -> impl Responder {
    let email = user.email.clone();
    let pw = user.password.clone();

    let mut db = USERS_DB.lock().unwrap();
    let new_id = db.values().last().map_or(1, |last_user| last_user.1 + 1);

    let hash_pw = hash(pw, DEFAULT_COST).unwrap();
    db.insert(email.clone(), (hash_pw, new_id));

    session.insert("user_id", new_id).unwrap();

    HttpResponse::Ok().json(JWT { id: new_id, email })
}

#[actix_web::post("/user/login")]
async fn login_user_query(user: web::Json<UserJson>, session: Session) -> impl Responder {
    let email = user.email.clone();
    let pw = user.password.clone();
    let db = USERS_DB.lock().unwrap();

    if let Some((hash_pw, user_id)) = db.get(&email) {
        if verify(&pw, hash_pw).is_ok() {
            session.insert("user_id", *user_id).unwrap();
            return HttpResponse::Ok().json(JWT { id: *user_id, email });
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[actix_web::get("/home")]
async fn home(session: Session) -> impl Responder {
    if let Some(user_id) = session.get::<u32>("user_id").unwrap() {
        HttpResponse::Ok().body(format!("Bienvenue sur la page d'accueil, utilisateur {}", user_id))
    } else {
        HttpResponse::Found()
            .header("LOCATION", "/login")
            .finish()
    }
}

#[actix_web::get("/login")]
async fn login_page() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../../static/login.html"))
}

#[actix_web::get("/register")]
async fn register_page() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../../static/register.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .wrap(Logger::default())
            .service(create_user_query)
            .service(login_user_query)
            .service(home)
            .service(login_page)
            .service(register_page)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
