import {MutableRefObject, useEffect, useRef, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {EditorDataSendCommand} from "../lib/editor_data/send/index.ts";
import {listen, UnlistenFn} from "@tauri-apps/api/event";
import {EditorDataRecvEvent} from "../lib/editor_data/recv/index.ts";
import {makeCancelable} from "../lib/index.ts";

export enum TabType {
    Welcome = "Welcome",
    MapEditor = "MapEditor",
    LiveViewer = "LiveViewer"
}

export type Tab = {
    name: string,
    tab_type: TabType
}

export type UseTabsReturn = {
    tabs: Tab[],
    openedTab: number,
    addTab: (tab: Tab) => Promise<void>,
    removeTab: (index: number) => void,
    setOpenedTab: (index: number) => void,
}


export function useTabs(): UseTabsReturn {
    const [tabs, setTabs] = useState<Tab[]>([])
    const [openTab, setOpenTab] = useState<number | null>(null)

    useEffect(() => {
        const unlistenOpened = makeCancelable(listen<Tab>(EditorDataRecvEvent.TabCreated, e => {
            setTabs(tabs => [...tabs, e.payload])
        }))

        let unlistenClosed = makeCancelable(listen<number>(EditorDataRecvEvent.TabClosed, e => {
            setTabs(tabs => {
                const newTabs = [...tabs]
                newTabs.splice(e.payload, 1)

                return newTabs
            })
        }))

        return () => {
            unlistenOpened.cancel()
            unlistenClosed.cancel()
        }
    }, []);

    async function addTab(tab: Tab) {
        await invoke(EditorDataSendCommand.CreateTab, {name: tab.name, tabType: tab.tab_type})
    }

    async function removeTab(index: number) {
        if (index === openTab) setOpenTab(null)
        await invoke(EditorDataSendCommand.CloseTab, {index})
    }

    function setOpenedTab(index: number) {
        setOpenTab(index)
    }

    return {
        tabs,
        addTab,
        removeTab,
        openedTab: openTab,
        setOpenedTab,
    }
}