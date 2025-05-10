import React from "react";
import GenericWindow from "../generic-window.js";
import {getCurrentWindow} from "@tauri-apps/api/window";

function Main() {
    async function onThemeChange() {
        const window = getCurrentWindow();
        await window.emit("change-theme");
    }

    return (
        <GenericWindow title={"Settings"}>
            <button onClick={onThemeChange}>Change Theme</button>
        </GenericWindow>
    );
}

export default Main;
