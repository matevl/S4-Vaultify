import React, { useState, useEffect } from 'react';
import './login.css';
export default function Login() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [loginError, setLoginError] = useState(false);
    const [accountCreated, setAccountCreated] = useState(false);

    useEffect(() => {
        if (localStorage.getItem('accountCreated') === 'true') {
            setAccountCreated(true);
            localStorage.removeItem('accountCreated');
            setTimeout(() => setAccountCreated(false), 3000);
        }
    }, []);

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

            if (!res.ok) {
                if (res.status === 401) {
                    setLoginError(true);
                    setTimeout(() => setLoginError(false), 3000);
                } else {
                    const text = await res.text();
                    alert('Erreur : ' + text);
                }
                return;
            }

            localStorage.setItem('loginSuccess', 'true');
            setTimeout(() => {
                window.location.href = '/home';
            }, 50);
        } catch (err) {
            console.error('Erreur réseau :', err);
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
                <h2 className="login-title">Sign in</h2>
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
                            placeholder="Password"
                            className="login-input"
                        />
                    </div>
                    <button type="submit" className="login-button">Login</button>
                    <p className="login-text">
                        Not registered yet ? <a href="/create-user" className="login-link">Sign up</a>
                    </p>
                </form>
            </div>

            {loginError && (
                <div className="toast-notification error">
          <span className="toast-icon">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none"
                 viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
                 className="icon-size">
              <path strokeLinecap="round" strokeLinejoin="round"
                    d="M6 18 18 6M6 6l12 12" />
            </svg>
          </span>
                    <span>Credentials not found</span>
                </div>
            )}

            {accountCreated && (
                <div className="toast-notification success">
                <span className="toast-icon">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none"
                viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
                className="icon-size">
                <path strokeLinecap="round" strokeLinejoin="round"
              d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                </svg>
                </span>
                    <span>Account created</span>
                </div>
            )}
        </div>
    );
}