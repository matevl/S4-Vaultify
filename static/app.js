document.addEventListener('DOMContentLoaded', function () {
    const registerForm = document.getElementById('registerForm');

    registerForm.addEventListener('submit', async function (e) {
        e.preventDefault();

        const email = document.getElementById('email').value;
        const password = document.getElementById('password').value;

        const data = { email, password };

        try {
            const response = await fetch('/user/register', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(data),
            });

            if (response.ok) {
                const result = await response.json();
                console.log('Inscription réussie:', result);
                window.location.href = '/home';
            } else {
                const errorText = await response.text();
                console.error('Erreur lors de l\'inscription:', errorText);
            }
        } catch (error) {
            console.error('Erreur réseau:', error);
        }
    });
});
