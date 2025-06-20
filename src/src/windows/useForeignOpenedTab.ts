import {useEffect, useRef, useState} from "react";
import {__TAB_CHANGED} from "../tauri/events/types.js";
import {listen, UnlistenFn} from "@tauri-apps/api/event";

export function useForeignOpenedTab(): string {
    const [openedTab, setOpenedTab] = useState<string>(null)
    const tabUnlistenFnRef = useRef<UnlistenFn>(null)

    useEffect(() => {
        (async () => {
            tabUnlistenFnRef.current = await listen<string>(
                __TAB_CHANGED,
                e => {
                    setOpenedTab(e.payload)
                }
            )
        })()

        return () => {
            if (tabUnlistenFnRef.current) tabUnlistenFnRef.current()
        }
    }, [openedTab]);

    return openedTab
}