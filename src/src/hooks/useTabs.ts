import {MutableRefObject, useEffect, useRef, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {listen, UnlistenFn} from "@tauri-apps/api/event";
import {invokeTauri, makeCancelable} from "../lib/index.ts";
import {EditorDataRecvEvent, EditorDataSendCommand} from "../lib/editor_data.ts";

export enum TabTypeKind {
    Welcome = "Welcome",
    MapEditor = "MapEditor",
    LiveViewer = "LiveViewer"
}

export type Tab = {
    name: string,
    tab_type: TabTypeKind,
}

export type UseTabsReturn = {
    tabs: { [name: string]: Tab },
    addLocalTab: (tab: Tab) => void,
    removeLocalTab: (name: string) => void,
    openedTab: string,
    setOpenedTab: (name: string) => void,
}


export function useTabs(): UseTabsReturn {
    const [tabs, setTabs] = useState<{ [name: string]: Tab }>({})
    const [openTab, setOpenTab] = useState<string | null>(null)

    useEffect(() => {
        const unlistenOpened = listen<Tab>(EditorDataRecvEvent.TabCreated, e => {
            setTabs(tabs => {
                const newTabs = {...tabs}
                newTabs[e.payload.name] = e.payload
                return newTabs
            })
        })

        let unlistenClosed = listen<number>(EditorDataRecvEvent.TabClosed, e => {
            setTabs(tabs => {
                const newTabs = {...tabs}
                delete newTabs[e.payload]
                return newTabs
            })
        })

        return () => {
            unlistenOpened.then(f => f())
            unlistenClosed.then(f => f())
        }
    }, []);

    function addLocalTab(tab: Tab) {
        const newTabs = {...tabs}
        newTabs[tab.name] = tab
        setTabs(newTabs)
    }

    function removeLocalTab(name: string) {
        const newTabs = {...tabs}
        delete newTabs[name]
        setTabs(newTabs)
    }

    function setOpenedTab(name: string) {
        setOpenTab(name)
    }

    return {
        tabs,
        addLocalTab,
        removeLocalTab,
        openedTab: openTab,
        setOpenedTab,
    }
}