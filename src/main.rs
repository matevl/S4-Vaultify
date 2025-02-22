use dioxus::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use regex::Regex; // Ajoute cette dépendance dans Cargo.toml
use web_sys::window;
use wasm_bindgen_futures::spawn_local;
use web_sys::Clipboard;
use std::any::type_name;
use gloo_timers::*;

const FAVICON: Asset = asset!("/assets/Vaultify-black-png.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/vault-text-svg.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Hero {}
        TextEditor {} // Ajout du composant TextEditor
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://github.com/matevl/S4-Vaultify", "en savoir plus sur nous 🔒" }
            }
        }
    }
}

#[component]
pub fn TextEditor() -> Element {
    let input_email = Rc::new(RefCell::new(String::new())); // Utilisation de RefCell pour rendre mutable
    let validation_message = Rc::new(RefCell::new(String::new())); // Message de validation de l'email

    let input_text_clone = Rc::clone(&input_email);
    let validation_message_clone = Rc::clone(&validation_message);

    let input_text_clone2 = Rc::clone(&input_email);
    let validation_message_clone2 = Rc::clone(&validation_message);

    let input_text_clone3 = Rc::clone(&input_email);
    let validation_message_clone3 = Rc::clone(&validation_message);

    let input_text_clone4 = Rc::clone(&input_email);
    let validation_message_clone4 = Rc::clone(&validation_message);

    let input_text_clone5 = Rc::clone(&input_email);
    let validation_message_clone5 = Rc::clone(&validation_message);

    let show_blank_page = Rc::new(RefCell::new(false));

    let toggle_blank_page = {
        let show_blank_page = Rc::clone(&show_blank_page);
        move || {
            let mut show_blank_page = show_blank_page.borrow_mut();
            *show_blank_page = true;
        }
    };
    let notification_message = Rc::new(RefCell::new(String::new()));
    let show_notification = Rc::new(RefCell::new(false));
    let notification_message_clone = Rc::clone(&notification_message);
    let show_notification_clone = Rc::clone(&show_notification);
    let handle_notification = move |message: String| {
        {
            let mut msg = notification_message_clone.borrow_mut();
            *msg = message;
        }

        {
            let mut show = show_notification_clone.borrow_mut();
            *show = true;
        }

        
    };


    let input_mdp = Rc::new(RefCell::new(String::new())); // Utilisation de RefCell pour rendre mutable
    let validation_mdp = Rc::new(RefCell::new(String::new()));

       rsx! {
        div {
            id: "text-editor",
            style: "margin-top: 2rem; padding: 1rem; border: 1px solid #ccc; border-radius: 5px; color: white\
            ; background-color: #021433;",
            class:"container",
            h2 { "Se Connecter" }

            // Zone de texte
            textarea {
                style: "width: 94%; height: 20px; padding: 0.5rem; font-size: 1rem;",
                placeholder: "E-Mail",
                value: "{input_email.borrow()}",
                oninput: move |evt| {
                    let value = evt.value().clone(); // Obtenir la valeur de l'événement
                    let mut text = input_text_clone5.borrow_mut();
                    *text = value; // Met à jour le texte avec la nouvelle valeur
                },
            }

            // Bouton pour valider l'email


            // Afficher le message de validation
            div {
                style: "margin-top: 1rem; color: red;",
                "{validation_message.borrow()}"
            }



            textarea {
                style: "width: 94%; height: 20px; padding: 0.5rem; font-size: 1rem;",
                placeholder: "Mot-de-passe",
                value: "{input_email.borrow()}",
                oninput: move |evt| {
                    let value = evt.value().clone(); // Obtenir la valeur de l'événement
                    let mut text = input_mdp.borrow_mut();
                    *text = value; // Met à jour le texte avec la nouvelle valeur
                },
            }

            button {

                style: "margin-top: 1rem; padding: 0.5rem 1rem; font-size: 1rem; cursor: pointer;",
                onclick: move |_| {
                    let email = input_text_clone.borrow().trim().to_string();
                    if is_valid_email(&email) {
                        validation_message_clone.borrow_mut().clear(); // Effacer tout message précédent
                        validation_message_clone.borrow_mut().push_str("L'email est valide.");
                    } else {
                        validation_message_clone.borrow_mut().clear(); // Effacer tout message précédent
                        validation_message_clone.borrow_mut().push_str("L'email est pas valide.");
                    }
                },
                "Se connecter"
               }


            // Bouton pour effacer le texte
            button {
                style: "margin-top: 1rem; padding: 0.5rem 1rem; font-size: 1rem; cursor: pointer;",
                onclick: move |_| {
                    input_text_clone3.borrow_mut().clear();
                },
                "Effacer"
            }
               
           button {
                onclick: toggle_blank_page,
                "Afficher Page Blanche"
               }
        

        }
    }
}

// Fonction pour vérifier la validité d'un email
fn is_valid_email(email: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}

// Fonction pour sauvegarder du texte dans un fichier (si nécessaire)
fn save_text_to_file(text: &str) {
    // Logique pour sauvegarder du texte dans un fichier
    println!("Enregistrer le texte : {}", text);
}

// Fonction pour copier du texte dans le presse-papiers

