// src/components/Home.jsx
import React, { useState } from 'react';
import './home.css';

export default function Home() {
    const [newVaultName, setNewVaultName] = useState('');
    const [creating, setCreating] = useState(false);

    const handleCreateVault = async e => {
        e.preventDefault();
        if (!newVaultName.trim()) return;

        setCreating(true);
        try {
            const res = await fetch('/create-vault', {
                method: 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded',
                },
                body: new URLSearchParams({ name: newVaultName }),
            });
            if (!res.ok) {
                const text = await res.text();
                throw new Error(text || `Erreur ${res.status}`);
            }
            // après création, on peut recharger l'iframe des vaults
            document.getElementById('vaults-iframe').contentWindow.location.reload();
            setNewVaultName('');
        } catch (err) {
            console.error('Erreur création vault:', err);
            alert('Impossible de créer le vault : ' + err.message);
        } finally {
            setCreating(false);
        }
    };

    return (
        <div className="dashboard">
            {/* Sidebar */}
            <div className="sidebar">
                <h2 className="sidebar-title">Vaultify</h2>
                <ul className="sidebar-menu">
                    <li>🏠 Accueil</li>
                    <li>🔐 Mes Vaults</li>
                    <li>📜 Historique</li>
                    <li>⚙️ Paramètres</li>
                </ul>
            </div>

            {/* Contenu principal */}
            <div className="main-content">
                {/* Header */}
                <div className="dashboard-header">
                    <h1>Bienvenue</h1>
                </div>

                {/* SECTION: Mes Vaults (embed) */}
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

                {/* SECTION: Création de Vault */}
                <div className="dashboard-section">
                    <h2>Créer un nouveau Vault</h2>
                    <form className="vault-create-form" onSubmit={handleCreateVault}>
                        <input
                            type="text"
                            placeholder="Nom du Vault"
                            value={newVaultName}
                            onChange={e => setNewVaultName(e.target.value)}
                            disabled={creating}
                            required
                        />
                        <button
                            type="submit"
                            className="btn btn-primary"
                            disabled={creating}
                        >
                            {creating ? 'Création…' : 'Créer Vault'}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    );
}