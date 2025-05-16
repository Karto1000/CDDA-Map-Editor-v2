import {MutableRefObject, useRef} from "react";
import {Webview} from "@tauri-apps/api/webview";

export type UseWindowsRet = {
    openMapWindowRef: MutableRefObject<Webview>,
    settingsWindowRef: MutableRefObject<Webview>,
}

export function useWindows(): UseWindowsRet {
    // Thanks to the legend at https://stackoverflow.com/questions/77775315/how-to-create-mulitwindows-in-tauri-rust-react-typescript-html-css
    const openMapWindowRef = useRef<Webview>()
    const settingsWindowRef = useRef<Webview>()

    return {
        openMapWindowRef,
        settingsWindowRef,
    }
}