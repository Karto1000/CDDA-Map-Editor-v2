import React from "react";
import ReactDOM from "react-dom/client";
import Main from "./main.js";

document.getElementById("window-root").style.height = "100%";
ReactDOM.createRoot(document.getElementById("window-root") as HTMLElement).render(
    <React.StrictMode>
        <Main/>
    </React.StrictMode>,
);