import React from 'react';
import './index.css';

export default function Index() {
    return (
        <div className="index-page">
            <div className="animated-bg"></div>
            <div className="hero-container">
                <div className="hero-content">
                    <img src="/static/vault-text-svg.svg" alt="Vaultify logo" className="hero-logo" />
                    <h1 className="vaultify-title">Bienvenue sur Vaultify</h1>
                    <p className="hero-subtitle">Un espace sécurisé pour vos fichiers confidentiels.</p>
                    <div className="hero-buttons">
                        <a href="/login" className="btn btn-primary">Se connecter</a>
                        <a href="/create-user" className="btn btn-secondary">Créer un compte</a>
                    </div>
                </div>
                <div className="about-container">
                    <a href="/about" className="about-link">À propos</a>
                </div>
            </div>
        </div>
    );
}