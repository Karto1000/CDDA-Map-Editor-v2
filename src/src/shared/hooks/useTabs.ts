import {RefObject, useState} from "react";
import {useTauriEvent} from "./useTauriEvent.js";
import {TauriEvent} from "../../tauri/events/types.js";

export enum TabTypeKind {
    MapEditor = "MapEditor",
    LiveViewer = "LiveViewer"
}

export type Tab = {
    name: string,
    tab_type: TabTypeKind,
}

export type UseTabsReturn = {
    tabs: { [name: string]: Tab },
    openedTab: string,
    shouldDisplayCanvas: () => boolean,
    getCurrentTab: () => Tab | null,
}


export function useTabs(): UseTabsReturn {
    const [tabs, setTabs] = useState<{ [name: string]: Tab }>({})
    const [openedTab, setOpenedTab] = useState<string | null>(null)

    useTauriEvent(
        TauriEvent.OPEN_TAB,
        data => {
            if (!tabs[data.name]) return
            setOpenedTab(() => data.name)
        },
        [tabs]
    )

    useTauriEvent(
        TauriEvent.CLOSE_TAB,
        data => {
            if (!tabs[data.name]) return
            setOpenedTab(() => null)
        },
        [tabs]
    )

    useTauriEvent(
        TauriEvent.CREATE_TAB,
        (data) => {
            setTabs((t) => {
                return {...t, [data.name]: data}
            })
        },
        [tabs]
    )

    useTauriEvent(
        TauriEvent.REMOVE_TAB,
        (data) => {
            if (openedTab === data.name) {
                if (!tabs[data.name]) return
                setOpenedTab(() => null)
            }

            setOpenedTab(() => {
                setTabs(tabs => {
                    const newTabs = {...tabs}
                    delete newTabs[data.name]
                    return newTabs
                })

                return null
            })
        },
        [tabs]
    )

    return {
        tabs,
        openedTab: openedTab,
        shouldDisplayCanvas: () => {
            if (!openedTab) return false
            if (!tabs[openedTab]) return false

            if (tabs[openedTab].tab_type === TabTypeKind.MapEditor) return true
            return tabs[openedTab].tab_type === TabTypeKind.LiveViewer;
        },
        getCurrentTab: () => {
            return tabs[openedTab]
        }
    }
}