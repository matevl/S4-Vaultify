use dioxus::prelude::*;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::ops::Deref;
use regex::Regex;
use web_sys::window;
mod backend;
use backend::*;
use crate::backend::account_manager::account::{load_users_data, local_log_in, UserInput};
use crate::backend::vault_manager::init_config_vaultify;

mod error_manager;
const FAVICON: Asset = asset!("/assets/Vaultify-black-png.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/vault-text-svg.svg");

fn main() {
    init_config_vaultify();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Hero {}
        TextEditor {}
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://github.com/matevl/S4-Vaultify", "En savoir plus sur nous 🔒" }
            }
        }
    }
}

#[component]
pub fn TextEditor() -> Element {
    let input_email = Rc::new(RefCell::new(String::new()));
    let input_mdp = Rc::new(RefCell::new(String::new()));
    let mut text  = "";
    let mut text2  = "";
    let userD = load_users_data(format!("{}{",));
        rsx! {
        div {
            id: "text-editor",
            style: "margin-top: 2rem; padding: 1rem; border: 1px solid #ccc; border-radius: 5px; color: white; background-color: #021433;",
            class:"container",
            h2 { "Se Connecter" }

            textarea {
                style: "width: 94%; height: 20px; padding: 0.5rem; font-size: 1rem;",
                placeholder: "E-Mail",
                value: "{input_email.borrow()}",
                oninput: move |evt| {
                    
                    text = input_email.borrow().deref().as_str();
                },
            }
            
            textarea {
                style: "width: 94%; height: 20px; padding: 0.5rem; font-size: 1rem;",
                placeholder: "mot de passe",
                value: "{input_mdp.borrow()}",
                oninput: move |evt| {
                    
                    text2 = input_mdp.borrow_mut().deref().as_str();
                },
            }

            button {
                style: "margin-top: 1rem; padding: 0.5rem 1rem; font-size: 1rem; cursor: pointer;",
                onclick: move |_| {
                    let email = text ;
                    let mdp = text2;
                    let userI = UserInput::new(email.to_string(), mdp.to_string());
                   
                    
                    match res {
                            Ok(jwt) => {
                            },
                            Err(_>) => {print!("email pas bon");}
                        }
                    
                },
                
            }
        }
    }
}

/// Fonction pour ouvrir une URL dans un nouvel onglet
fn open_webpage(url: &str) {
    if let Some(win) = window() {
        win.open_with_url(url).unwrap();
    }
}
