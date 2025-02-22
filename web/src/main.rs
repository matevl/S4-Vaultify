use dioxus::prelude::*;
use reqwest::Error;
use serde::Deserialize;
use tokio::runtime;

// Structure pour les données utilisateur venant du back-end
#[derive(Deserialize)]
struct UserData {
    first_name: String,
    last_name: String,
    email: String,
}

#[component]
fn App() -> Element {
    // Crée un signal pour stocker les données utilisateur
    let user_data = use_signal(|| None::<UserData>);

    // On effectue l'appel HTTP lorsque le composant est monté
    use_effect(|| {
        async {
            match fetch_user_data().await {
                Ok(data) => {
                    user_data.set(Some(data));  // On met à jour les données
                }
                Err(_) => {
                    user_data.set(None);  // En cas d'erreur, on remet None
                }
            }
        };
        || {}  // Fonction de nettoyage (vide ici)
    }, ());

    // Affichage des données
    rsx! {
        div {
            style: "background-color: white; height: 100vh; padding: 20px;",
            h1 { "Informations Utilisateur" }
            match user_data.get() {
                Some(data) => {
                    p { "Prénom: " "{data.first_name}" }
                    p { "Nom: " "{data.last_name}" }
                    p { "Email: " "{data.email}" }
                }
                None => {
                    p { "Chargement des données..." }
                }
            }
        }
    }
}

// Fonction pour effectuer l'appel HTTP et récupérer les données utilisateur
async fn fetch_user_data() -> Result<UserData, Error> {
    let response = reqwest::get("http://127.0.0.1:8080/api/user")
        .await?
        .json::<UserData>()
        .await?;

    Ok(response)
}

// Fonction principale pour lancer l'application avec le runtime tokio
fn main() {
    // Lancer le runtime tokio et l'application Dioxus
    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(dioxus::launch(App)); // Lancer l'application
}
