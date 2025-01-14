import React from 'react';
import './index.scss';
import App from './app.tsx';
import {createRoot} from "react-dom/client";

const root = createRoot(document.getElementById('root'));
root.render(
    <App/>
);

