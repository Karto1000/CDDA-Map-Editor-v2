import {RefObject, useRef} from "react";
import {Webview} from "@tauri-apps/api/webview";

export type UseWindowsRet = {
    newMapWindowRef: RefObject<Webview>,
    importMapWindowRef: RefObject<Webview>,
    settingsWindowRef: RefObject<Webview>,
    aboutWindowRef: RefObject<Webview>,
    mapInfoWindowRef: RefObject<Webview>,
    palettesWindowRef: RefObject<Webview>,
}

export function useWindows(): UseWindowsRet {
    // Thanks to the legend at https://stackoverflow.com/questions/77775315/how-to-create-mulitwindows-in-tauri-rust-react-typescript-html-css
    const importMapWindowRef = useRef<Webview>(null)
    const settingsWindowRef = useRef<Webview>(null)
    const newMapWindowRef = useRef<Webview>(null)
    const aboutWindowRef = useRef<Webview>(null)
    const mapInfoWindowRef = useRef<Webview>(null)
    const palettesWindowRef = useRef<Webview>(null)

    return {
        newMapWindowRef,
        importMapWindowRef,
        settingsWindowRef,
        aboutWindowRef,
        mapInfoWindowRef,
        palettesWindowRef,
    }
}