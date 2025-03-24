const registerForm = document.getElementById('registerForm');
const loginForm = document.getElementById('loginForm');
const messageDiv = document.getElementById('message');

const registerUrl = 'http://127.0.0.1:8080/auth/register';
const loginUrl = 'http://127.0.0.1:8080/auth/login';

// Gérer l'inscription
registerForm.addEventListener('submit', async (e) => {
    e.preventDefault();

    const name = document.getElementById('name').value;
    const email = document.getElementById('registerEmail').value;
    const password = document.getElementById('registerPassword').value;

    try {
        const res = await fetch(registerUrl, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ name, email, password }),
        });

        if (res.ok) {
            messageDiv.innerText = 'Inscription réussie !';
        } else {
            messageDiv.innerText = 'Erreur lors de l\'inscription.';
        }
    } catch (err) {
        console.error(err);
        messageDiv.innerText = 'Erreur réseau.';
    }
});

// Gérer la connexion
loginForm.addEventListener('submit', async (e) => {
    e.preventDefault();

    const email = document.getElementById('loginEmail').value;
    const password = document.getElementById('loginPassword').value;

    try {
        const res = await fetch(loginUrl, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ email, password }),
        });

        if (res.ok) {
            const data = await res.json();
            messageDiv.innerText = `Connexion réussie ! Token : ${data.jwt}`;
        } else {
            messageDiv.innerText = 'Erreur lors de la connexion.';
        }
    } catch (err) {
        console.error(err);
        messageDiv.innerText = 'Erreur réseau.';
    }
});
