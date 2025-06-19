import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {Theme} from "@tauri-apps/api/window";
import {emitTo, UnlistenFn} from "@tauri-apps/api/event";
import {Webview} from "@tauri-apps/api/webview";

export enum WindowLabel {
    Main = "main",
    Settings = "settings",
    ImportMap = "import-map",
    NewMap = "new-map",
    About = "about",
    Welcome = "welcome",
    MapInfo = "map-info",
}

export type WindowOptions = {
    defaultWidth?: number,
    defaultHeight?: number,
}

export async function openWindow<T = any>(
    label: WindowLabel,
    theme: Theme,
    {
        defaultWidth = 400,
        defaultHeight = 400
    }: WindowOptions = {},
    data?: T
): Promise<[WebviewWindow, UnlistenFn]> {
    const existingWindow = await Webview.getByLabel(label.toString())
    if (existingWindow) return [existingWindow, () => {}]

    const window = new WebviewWindow(label.toString(), {
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

    const unlisten = await window.once("window-ready", async () => {
        await emitTo(label, "initial-data", data)
    })

    return [window, unlisten]
}