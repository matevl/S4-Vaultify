import React from 'react';
import './index.css';

export default function Index() {
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
        </div>
    );
}