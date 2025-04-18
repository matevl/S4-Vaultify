import React, { useState } from 'react';

export default function Login() {
    const [email, setEmail]     = useState('');
    const [password, setPassword] = useState('');

    const handleSubmit = async e => {
        e.preventDefault();
        const loginData = { username: email, password };

        try {
            const res = await fetch('/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify(loginData)
            });

            if (res.ok) {
                window.location.href = '/home';
            } else {
                const err = await res.json();
                alert('Erreur : ' + err.message);
            }
        } catch (err) {
            console.error(err);
            alert('Erreur serveur');
        }
    };

    return (
        <div className="login-page">
            <div className="login-background-animated" />
            <div className="login-box">
                <div className="login-logo">
                    <img src="../static/vault-text-svg.svg" alt="Bannière" className="logo-img" />
                </div>
                <h2 className="login-title">Se connecter</h2>
                <form onSubmit={handleSubmit} className="login-form">
                    <div className="input-icon-wrapper">
                        <svg xmlns="http://www.w3.org/2000/svg" className="input-icon" fill="none" viewBox="0 0 24 24" stroke="#ffffff">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2.5" d="M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75" />
                        </svg>
                        <input
                            type="email"
                            id="email"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            required
                            placeholder="Email"
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
                    <button type="submit" className="login-button">Se connecter</button>
                    <p className="login-text">
                        Pas encore inscrit ? <a href="/create-user" className="login-link">S'inscrire</a>
                    </p>
                </form>
            </div>
        </div>
    );
}