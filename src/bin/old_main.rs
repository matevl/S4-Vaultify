use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use webbrowser;

#[derive(Serialize, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let register_url = "http://127.0.0.1:8080/auth/register";
    let login_url = "http://127.0.0.1:8080/auth/login";

    // 1. Créer un fichier index.html local
    let html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authentification</title>
</head>
<body>
    <h1>Bienvenue sur l'interface d'authentification</h1>
    <p>Veuillez accéder à l'interface via le formulaire d'inscription ou de connexion.</p>
</body>
</html>
"#;

    let file_path = "index.html";
    let mut file = File::create(file_path).expect("Impossible de créer le fichier HTML.");
    file.write_all(html_content.as_bytes())
        .expect("Impossible d'écrire dans le fichier HTML.");

    // 2. Ouvrir le fichier index.html dans le navigateur
    if webbrowser::open(file_path).is_ok() {
        println!("Le fichier HTML a été ouvert dans le navigateur.");
    } else {
        println!("Impossible d'ouvrir le fichier HTML.");
    }

    // 3. Exemple d'inscription
    let register_data = RegisterRequest {
        email: "user@example.com".to_string(),
        password: "password123".to_string(),
        name: "John Doe".to_string(),
    };

    let client = reqwest::Client::new();
    let res = client
        .post(register_url)
        .json(&register_data)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        println!("Utilisateur enregistré avec succès!");
    } else {
        println!("Erreur lors de l'enregistrement.");
    }

    // 4. Exemple de connexion
    let login_data = LoginRequest {
        email: "user@example.com".to_string(),
        password: "password123".to_string(),
    };

    let res = client
        .post(login_url)
        .json(&login_data)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let jwt: String = res.json().await.unwrap();
        println!("Connexion réussie ! JWT : {}", jwt);
    } else {
        println!("Erreur lors de la connexion.");
    }
}
