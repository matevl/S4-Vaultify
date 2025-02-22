use dioxus::prelude::*;
use regex::Regex;
use std::sync::mpmc::SendTimeoutError::Timeout;

// ... (assets and other code)

#[derive(Clone, PartialEq)]
enum Route {
    Home,
    LoginPage,
    BlankPage,
}

#[component]
fn App() -> Element {
    let current_route = use_signal(|| Route::Home); // Use signal here

    rsx! {
        // ... (document links)

        match *current_route {
            Route::Home => rsx! { Home { current_route: current_route } }, // No .clone() needed
            Route::LoginPage => rsx! { LoginPage { current_route: current_route } }, // No .clone() needed
            Route::BlankPage => rsx! { BlankPage {} },
        }
    }
}

#[component]
pub fn Home(current_route: Signal<Route>) -> Element { // Signal as prop
    rsx! {
        div {
            // ... (hero content)
            button {
                onclick: move |_| current_route.set(Route::LoginPage), // No .clone() needed
                "Se connecter"
            }
            button {
                onclick: move |_| current_route.set(Route::BlankPage), // No .clone() needed
                "Page blanche"
            }
        }
    }
}

#[component]
pub fn LoginPage(current_route: Signal<Route>) -> Element { // Signal as prop
    let input_email = use_signal(|| String::new()); // Use signal
    let validation_message = use_signal(|| String::new()); // Use signal
    let input_mdp = use_signal(|| String::new()); // Use signal
    let notification_message = use_signal(|| String::new()); // Use signal
    let show_notification = use_signal(|| false); // Use signal

    let handle_notification = move |message: String| {
        *notification_message = message; // Direct assignment
        *show_notification = true; // Direct assignment

        gloo_timers::Timeout::new(3000, move || {
            *show_notification = false; // Direct assignment
        }).start();
    };

    rsx! {
        div {
            // ... (text-editor content)

            textarea {
                // ...
                value: "{input_email}", // Access signal value directly
                oninput: move |evt| *input_email = evt.value().clone(), // Direct assignment
            }

            div {
                // ...
                "{validation_message}" // Access signal value directly
            }

            textarea {
                // ...
                value: "{input_mdp}", // Access signal value directly
                oninput: move |evt| *input_mdp = evt.value().clone(), // Direct assignment
            }

            button {
                // ...
                onclick: move |_| {
                    let email = input_email.trim().to_string(); // Access signal value directly
                    if is_valid_email(&email) {
                        *validation_message = "L'email est valide.".to_string(); // Direct assignment
                        handle_notification("Email valide !".to_string());
                    } else {
                        *validation_message = "L'email est pas valide.".to_string(); // Direct assignment
                        handle_notification("Email invalide !".to_string());
                    }
                },
                "Se connecter"
            }

            button {
                // ...
                onclick: move |_| {
                    *input_email = String::new(); // Direct assignment
                    *input_mdp = String::new(); // Direct assignment
                    *validation_message = String::new(); // Direct assignment
                },
                "Effacer"
            }
            button {
                onclick: move |_| current_route.set(Route::Home), // No .clone() needed
                "Retour à l'accueil"
            }

            if *show_notification { // Access signal value directly
                div {
                    // ...
                    "{notification_message}" // Access signal value directly
                }
            }
        }
    }
}

// ... (BlankPage and is_valid_email)
#[component]
pub fn BlankPage() -> Element {
    rsx! {
        div {
            "Page blanche !"
        }
    }
}


fn is_valid_email(email: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}