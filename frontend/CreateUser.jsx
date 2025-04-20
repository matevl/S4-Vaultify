import React, { useState } from 'react';

export default function CreateUser() {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [accountCreated, setAccountCreated] = useState(false);

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
                localStorage.setItem('accountCreated', 'true'); // ← on enregistre
                window.location.href = '/login'; // redirection immédiate
            }
             else {
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
                <h2 className="login-title">Create an account</h2>
                <form onSubmit={handleSubmit} className="login-form">
                    <div className="input-icon-wrapper">
                            <svg xmlns="http://www.w3.org/2000/svg" className="input-icon" fill="none" viewBox="0 0 24 24" stroke="#ffffff">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2.5" d="M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75" />
                            </svg>
                        <input
                            type="text"
                            id="username"
                            value={username}
                            onChange={(e) => setUsername(e.target.value)}
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
                            placeholder="Password"
                            className="login-input"
                        />
                    </div>
                    <button type="submit" className="login-button">Create an account</button>
                    <p className="login-text">
                        Already registered ?<a href="/login" className="login-link"> Sign in</a>
                    </p>
                </form>
            </div>
        </div>
    );
}