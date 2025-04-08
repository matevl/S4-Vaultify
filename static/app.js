// Fonction pour lire un cookie
function getCookie(name) {
    let nameEQ = name + "=";
    let ca = document.cookie.split(';');
    for (let i = 0; i < ca.length; i++) {
        let c = ca[i];
        while (c.charAt(0) === ' ') c = c.substring(1, c.length);
        if (c.indexOf(nameEQ) === 0) return c.substring(nameEQ.length, c.length);
    }
    return null;
}

// Fonction pour définir un cookie avec des paramètres de sécurité
function setCookie(name, value, days) {
    let d = new Date();
    d.setTime(d.getTime() + (days * 24 * 60 * 60 * 1000));
    let expires = "expires=" + d.toUTCString();
    // Ajout de l'attribut Secure (nécessite HTTPS) et HttpOnly pour plus de sécurité
    document.cookie = name + "=" + value + ";" + expires + ";path=/;Secure;HttpOnly";
}

// Fonction pour supprimer un cookie
function deleteCookie(name) {
    document.cookie = name + "=; Max-Age=-99999999; path=/;";
}

// Fonction pour afficher le formulaire de login et cacher celui d'enregistrement
function showLoginForm() {
    document.getElementById('login-form').style.display = 'block';
    document.getElementById('register-form').style.display = 'none';
}

// Fonction pour afficher le formulaire d'enregistrement et cacher celui de login
function showRegisterForm() {
    document.getElementById('login-form').style.display = 'none';
    document.getElementById('register-form').style.display = 'block';
}

// Écouter la soumission du formulaire de login
document.getElementById('login').addEventListener('submit', function(event) {
    event.preventDefault();

    let email = document.getElementById('email').value;
    let password = document.getElementById('password').value;

    // Vérifie les informations de connexion
    fetch('/user/login', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            email: email,
            password: password
        })
    })
        .then(response => response.json())
        .then(data => {
            if (data.success) {
                console.log('Connexion réussie:', data);
                // Stocke le token d'authentification dans un cookie sécurisé
                setCookie('user_token', data.token, 7); // Le cookie expire après 7 jours
                // Redirige vers la page d'accueil après la connexion réussie
                window.location.href = "/home"; // La redirection vers /home
            } else {
                console.error('Échec de la connexion:', data.message);
                alert('Erreur de connexion, veuillez vérifier vos informations.');
            }
        })
        .catch(error => {
            console.error('Erreur:', error);
        });
});

// Écouter la soumission du formulaire d'enregistrement
document.getElementById('register').addEventListener('submit', function(event) {
    event.preventDefault();

    let email = document.getElementById('reg-email').value;
    let password = document.getElementById('reg-password').value;
    let confirmPassword = document.getElementById('confirm-password').value;

    // Vérifier si les mots de passe correspondent
    if (password !== confirmPassword) {
        alert("Les mots de passe ne correspondent pas.");
        return;
    }

    // Envoie les données d'enregistrement
    fetch('/user/register', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            email: email,
            password: password
        })
    })
        .then(response => response.json())
        .then(data => {
            if (data.success) {
                console.log('Utilisateur créé:', data);
                alert('Inscription réussie ! Vous pouvez maintenant vous connecter.');
                // Affiche le formulaire de connexion
                showLoginForm();
            } else {
                console.error('Échec de l\'inscription:', data.message);
                alert('Erreur lors de l\'inscription, veuillez réessayer.');
            }
        })
        .catch(error => {
            console.error('Erreur:', error);
        });
});

// Logique spécifique pour la page /home
window.addEventListener('load', function() {
    const userToken = getCookie('user_token');

    if (userToken) {
        // Si le token existe, faire une requête pour récupérer les informations de l'utilisateur
        fetch('/user/profile', {
            method: 'GET',
            headers: {
                'Authorization': 'Bearer ' + userToken // Ajouter le token dans l'en-tête
            }
        })
            .then(response => response.json())
            .then(data => {
                if (data.success) {
                    // Personnaliser l'affichage en fonction des informations utilisateur
                    document.getElementById('welcome-message').innerText = `Bienvenue, ${data.username}!`;
                    document.getElementById('user-email').innerText = data.email; // Affiche l'email de l'utilisateur
                } else {
                    // Si la récupération échoue
                    document.getElementById('welcome-message').innerText = 'Cookie introuvable, veuillez vous connecter.';
                    document.getElementById('user-email').innerText = 'Aucune information disponible';
                }
            })
            .catch(error => {
                console.error('Erreur lors de la récupération des données utilisateur:', error);
                document.getElementById('welcome-message').innerText = 'Erreur lors de la récupération des données utilisateur.';
                document.getElementById('user-email').innerText = 'Aucune information disponible';
            });
    } else {
        // Si le token n'existe pas, afficher un message d'erreur
        document.getElementById('welcome-message').innerText = 'Cookie introuvable, veuillez vous connecter.';
        document.getElementById('user-email').innerText = 'Aucune information disponible';
    }
});

// Initialiser la page sur le formulaire de login
// Logique spécifique pour la page /home
window.addEventListener('load', function() {
    const userToken = getCookie('user_token');

    if (userToken) {
        // Si le token existe, faire une requête pour récupérer les informations de l'utilisateur
        fetch('/user/profile', {
            method: 'GET',
            headers: {
                'Authorization': 'Bearer ' + userToken // Ajouter le token dans l'en-tête
            }
        })
            .then(response => response.json())
            .then(data => {
                if (data.success) {
                    // Personnaliser l'affichage en fonction des informations utilisateur
                    document.getElementById('welcome-message').innerText = `Bienvenue, ${data.username}!`;
                    document.getElementById('user-email').innerText = data.email; // Affiche l'email de l'utilisateur
                } else {
                    // Si la récupération échoue
                    document.getElementById('welcome-message').innerText = 'Erreur de récupération des informations utilisateur';
                    document.getElementById('user-email').innerText = 'Aucune information disponible';
                }
            })
            .catch(error => {
                console.error('Erreur lors de la récupération des données utilisateur:', error);
                document.getElementById('welcome-message').innerText = 'Erreur lors de la récupération des données utilisateur';
                document.getElementById('user-email').innerText = 'Aucune information disponible';
            });
    } else {
        // Si le token n'existe pas, afficher un message d'erreur
        document.getElementById('welcome-message').innerText = 'Cookie introuvable, veuillez vous connecter.';
        document.getElementById('user-email').innerText = 'Aucune information disponible';
    }
});


