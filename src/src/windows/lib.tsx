import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {Theme} from "../hooks/useTheme.js";

export enum WindowLabel {
    Main = "main",
    Settings = "settings",
    OpenMap = "open-map"
}

export function openWindow(label: WindowLabel, theme: Theme): WebviewWindow {
    return new WebviewWindow(label.toString(), {
        url: `src/windows/${label.toString()}/window.html?theme=${theme.toString()}`,
        width: 400,
        height: 400,
        decorations: false,
        center: true,
        alwaysOnTop: true,
        title: label.toString()
    })

}