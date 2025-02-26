use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use s4_vaultify::backend::account_manager::account::{add_user_to_data, UserData, UserInput};

lazy_static!{
    let USERDATA : UserData = Mutex::new(HashMap::new());
}
#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct CreateUserResponse {
    id : u32,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct WuserData {
    id: u32,
    user_input: UserInput,
    usrdata: UserData,
    
}

#[actix_web::post("/user/register")]
async fn create_user( user_data: web::Json<UserInput>, db: web::Data<UserDb>) -> impl Responder {
    let mut db = db.lock().unwrap();
    let new_id = db.keys().max().unwrap_or(&0) + 1;
    let name = user_data.email.clone();
    let res = add_user_to_data(&user_data,vec);
    db.insert(new_id,WuserData{
        id : new_id,
        user_input: user_data.clone(),
        usrdata: res.unwrap(),
    });
    HttpResponse::Created().json(CreateUserResponse{
        id :new_id, 
        name
    })
}

#[actix_web::post("/user/log")]
async fn log_user( user_data: web::Json<User>, db: web::Data<UserDb>) -> impl Responder {
    let mut db = db.lock().unwrap();
    let new_id = db.keys().max().unwrap_or(&0) + 1;
    let name = user_data.name.clone();

    db.insert(new_id,);
    HttpResponse::Created().json(CreateUserResponse{
        id :new_id,
        name
    })
}
    

type UserDb = Arc<Mutex<HashMap<u32, WuserData>>>;
#[actix_web::get("/greet/{name}")]
async fn greet(user_id : web::Path<u32>) -> impl Responder {
    format!("Hello {user_id}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    let user_db: UserDb = Arc::new(Mutex::new(HashMap::<u32, WuserData>::new()));
    println!("Starting server on port {}", port);
    HttpServer::new(move || {
        let app_data = web::Data::new(user_db.clone());
            App::new()
                
                .app_data(app_data)
                .service(create_user)
                .service(greet)
        })

        .bind(format!("127.0.0.1:{}", port))?
        .workers(2)
        .run()
        .await
}
