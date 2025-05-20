import React, {useEffect, useState } from 'react';
import './home.css';

export default function Home() {
    const [vaultCreated, setVaultCreated] = useState(false);
    const [showToast, setShowToast] = useState(false);

    useEffect(() => {
        if (localStorage.getItem('loginSuccess') === 'true') {
            setShowToast(true);
            localStorage.removeItem('loginSuccess');
            setTimeout(() => setShowToast(false), 3000);
        }
    }, []);

    const [showModal, setShowModal] = useState(false);
    const [newVaultName, setNewVaultName] = useState('');
    const [creating, setCreating] = useState(false);

    const openModal = () => setShowModal(true);
    const closeModal = () => {
        if (!creating) {
            setShowModal(false);
            setNewVaultName('');
        }
    };

    const handleCreateVault = async e => {
        e.preventDefault();
        if (!newVaultName.trim()) return;

        setCreating(true);
        try {
            const res = await fetch('/create-vault', {
                method: 'POST',
                credentials: 'include',
                headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                body: new URLSearchParams({ name: newVaultName }),
            });
            if (!res.ok) {
                const text = await res.text();
                throw new Error(text || `Erreur ${res.status}`);
            }

            const iframe = document.getElementById('vaults-iframe');
            if (iframe) iframe.contentWindow.location.reload();
            setTimeout(() => {
                setVaultCreated(true);
            }, 10);
            setTimeout(() => {
                setVaultCreated(false);
            }, 3000);

            setTimeout(() => {
                closeModal();
            }, 50);
        } catch (err) {
            console.error('Error during creation:', err);
            alert('Impossible to create vault : ' + err.message);
        } finally {
            setCreating(false);
        }
    };

    return (
        <div className="dashboard">
            <div className="sidebar">
                <h2 className="sidebar-title">Vaultify</h2>
                <ul className="sidebar-menu">
                    <li>🏠 Accueil</li>
                    <li>🔐 Mes Vaults</li>
                    <li>📜 Historique</li>
                    <li>⚙️ Paramètres</li>
                </ul>
            </div>

            <div className="main-content">
                <div className="dashboard-header">
                    <h1>Bienvenue</h1>
                    <button className="btn btn-primary" onClick={openModal}>
                        + Créer un Vault
                    </button>
                </div>

                <div className="dashboard-section">
                    <h2>Mes Vaults</h2>
                    <div className="vaults-embed">
                        <iframe
                            id="vaults-iframe"
                            src="/vaults"
                            title="Mes Vaults"
                            frameBorder="0"
                            className="vaults-iframe"
                        />
                    </div>
                </div>
            </div>

            {showModal && (
                <div className="modal-overlay" onClick={closeModal}>
                    <div
                        className="modal-content"
                        onClick={e => e.stopPropagation()}
                    >
                        <h3>Créer un nouveau Vault</h3>
                        <form onSubmit={handleCreateVault}>
                            <input
                                type="text"
                                placeholder="Vault name"
                                value={newVaultName}
                                onChange={e => setNewVaultName(e.target.value)}
                                disabled={creating}
                                required
                            />
                            <div className="modal-actions">
                                <button
                                    type="button"
                                    className="btn btn-secondary"
                                    onClick={closeModal}
                                    disabled={creating}
                                >
                                    Annuler
                                </button>
                                <button
                                    type="submit"
                                    className="btn btn-primary"
                                    disabled={creating}
                                >
                                    {creating ? 'Creation…' : 'Create'}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            )}
            {showToast && (
                <div className="toast-notification">
                <span className="toast-icon">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none"
                         viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
                         className="icon-size">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                    </svg>
                </span>
                <span>Successful login</span>
                </div>
            )}
            {vaultCreated && (
                <div className="toast-notification">
                <span className="toast-icon">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none"
                viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
                className="icon-size">
                <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                </svg>
                </span>
                    <span>Successful vault creation</span>
                </div>
            )}
        </div>

    );
}