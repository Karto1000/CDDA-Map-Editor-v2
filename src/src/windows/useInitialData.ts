import {Dispatch, SetStateAction, useEffect, useRef, useState} from "react";
import {emitTo, once, UnlistenFn} from "@tauri-apps/api/event";
import {WindowLabel} from "./lib.js";
import {getCurrentWindow} from "@tauri-apps/api/window";

export const INITIAL_DATA = "initial-data"
export const WINDOW_READY = "window-ready"

export function useInitialData<T>(): [T, Dispatch<SetStateAction<T>>] {
    const [data, setData] = useState<T>(null)
    const unlistenFn = useRef<UnlistenFn>(null)

    useEffect(() => {
        (async () => {
            unlistenFn.current = await once<T>(INITIAL_DATA, p => {
                setData(p.payload)
            })

            const currentWindow = getCurrentWindow()
            await emitTo(currentWindow.label, WINDOW_READY)
        })()

        return () => {
            if (unlistenFn.current) unlistenFn.current()
        }
    }, []);

    return [data, setData]
}