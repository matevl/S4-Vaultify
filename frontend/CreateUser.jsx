import React, { useState } from 'react';

export default function CreateUser() {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');

    const handleSubmit = async (e) => {
        e.preventDefault();

        const userData = { username, password };

        try {
            const res = await fetch('/create-user', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(userData),
            });

            if (res.ok) {
                const responseData = await res.json();
                alert('Utilisateur créé : ' + responseData.message);
                window.location.href = '/login';
            } else {
                const errorData = await res.json();
                alert('Erreur : ' + errorData.message);
            }
        } catch (error) {
            console.error('Erreur:', error);
            alert('Une erreur est survenue lors de la création de l’utilisateur');
        }
    };

    return (
        <div className="login-page">
            <div className="login-background-animated" />
            <div className="login-box">
                <div className="login-logo">
                    <img src="../static/vault-text-svg.svg" alt="Bannière" className="logo-img" />
                </div>
                <h2 className="login-title">Créer un compte</h2>
                <form onSubmit={handleSubmit} className="login-form">
                    <div className="input-icon-wrapper">
                        <svg xmlns="http://www.w3.org/2000/svg" className="input-icon" fill="none" viewBox="0 0 24 24" stroke="#ffffff">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2.5" d="M5.121 17.804A13.937 13.937 0 0112 15c2.485 0 4.797.755 6.879 2.047M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                        </svg>
                        <input
                            type="text"
                            id="username"
                            value={username}
                            onChange={(e) => setUsername(e.target.value)}
                            required
                            placeholder="Nom d'utilisateur"
                            className="login-input"
                        />
                    </div>
                    <div className="input-icon-wrapper">
                        <svg xmlns="http://www.w3.org/2000/svg" className="input-icon" fill="none" viewBox="0 0 24 24" stroke="#ffffff">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2.5" d="M16 10V7a4 4 0 00-8 0v3M5 10h14v10H5V10z" />
                        </svg>
                        <input
                            type="password"
                            id="password"
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            required
                            placeholder="Mot de passe"
                            className="login-input"
                        />
                    </div>
                    <button type="submit" className="login-button">Créer un compte</button>
                    <p className="login-text">
                        Déjà inscrit ? <a href="/login" className="login-link">Se connecter</a>
                    </p>
                </form>
            </div>
        </div>
    );
}