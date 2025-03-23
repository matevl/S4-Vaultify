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
            if (data.id) {
                console.log('Connexion réussie:', data);
                // Redirige vers la page d'accueil ou dashboard
                window.location.href = "/dashboard";
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
            if (data.id) {
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

// Initialiser la page sur le formulaire de login
showLoginForm();
