import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {Theme} from "@tauri-apps/api/window";

export enum WindowLabel {
    Main = "main",
    Settings = "settings",
    ImportMap = "import-map",
    NewMap = "new-map",
    About = "about",
    Welcome = "welcome",
}

export type WindowOptions = {
    defaultWidth?: number,
    defaultHeight?: number,
}

export function openWindow(
    label: WindowLabel,
    theme: Theme,
    {
        defaultWidth = 400,
        defaultHeight = 400
    }: WindowOptions = {}
): WebviewWindow {
    return new WebviewWindow(label.toString(), {
        url: `src/windows/${label.toString()}/window.html?theme=${theme.toString()}`,
        width: defaultWidth,
        height: defaultHeight,
        decorations: false,
        center: true,
        alwaysOnTop: true,
        title: label.toString(),
        parent: WindowLabel.Main,
        skipTaskbar: true,
        focus: true
    })

}