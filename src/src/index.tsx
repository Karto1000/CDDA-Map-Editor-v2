import React from 'react';
import './index.scss';
import App from './app.tsx';
import {createRoot} from "react-dom/client";
import {attachLogger} from "@tauri-apps/plugin-log";

attachLogger(l => console.log(l.message)).then(r => {
})

const root = createRoot(document.getElementById('root'));
root.render(
    <App/>
);

