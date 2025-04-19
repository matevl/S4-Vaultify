import React from 'react';
import ReactDOM from 'react-dom/client';
import Login from './Login.jsx';
import CreateUser from './CreateUser.jsx';
import Index from "./Index.jsx";
import Home from "./Home.jsx";

const root = document.getElementById('root');
const page = root?.dataset.page;

const components = {
    index: <Index />,
    login: <Login />,
    create: <CreateUser />,
    home: <Home />,
};

if (root && page && components[page]) {
    ReactDOM.createRoot(root).render(components[page]);
} else {
    console.error("Composant introuvable pour la page :", page);
}