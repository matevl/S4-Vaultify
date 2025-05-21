import React, { useState, useEffect } from 'react';
import './index.css';

export default function Index() {
    const [showLogoutToast, setShowLogoutToast] = useState(false);

    useEffect(() => {
        if (localStorage.getItem('logoutSuccess') === 'true') {
            setShowLogoutToast(true);
            localStorage.removeItem('logoutSuccess');
            setTimeout(() => setShowLogoutToast(false), 3000);
        }
    }, []);

    return (
        <div className="index-page">
            <div className="animated-bg"></div>
            <div className="hero-container">
                <div className="hero-content">
                    <img src="/static/vault-text-svg.svg" alt="Vaultify logo" className="hero-logo" />
                    <h1 className="vaultify-title">Welcome to Vaultify</h1>
                    <p className="hero-subtitle">A secure space for your confidential files.</p>
                    <div className="hero-buttons">
                        <a href="/login" className="btn btn-primary">Sign in</a>
                        <a href="/create-user" className="btn btn-secondary">Sign up</a>
                    </div>
                </div>
                <div className="about-container">
                    <a href="/about" className="about-link">About us</a>
                    <a href="https://github.com/matevl/S4-Vaultify" className="about-link">Github</a>
                </div>

            </div>
            {showLogoutToast && (
                <div className="toast-notification">
                    <span className="toast-icon">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none"
                             viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
                             className="icon-size">
                            <path strokeLinecap="round" strokeLinejoin="round"
                                  d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                        </svg>
                    </span>
                    <span>Successful logout</span>
                </div>
            )}
        </div>
    );
}