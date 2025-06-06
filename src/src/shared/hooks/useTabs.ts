import {RefObject, useEffect, useState} from "react";
import {useTauriEvent} from "./useTauriEvent.js";
import {TauriEvent} from "../../tauri/events/types.js";
import {
    AddLocalTabEvent,
    CloseLocalTabEvent,
    LocalEvent,
    LocalEventsMap,
    RemoveLocalTabEvent
} from "../utils/localEvent.js";

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
    openedTab: string,
    shouldDisplayCanvas: () => boolean,
    getCurrentTab: () => Tab | null,
}


export function useTabs(eventBus: RefObject<EventTarget>): UseTabsReturn {
    const [tabs, setTabs] = useState<{ [name: string]: Tab }>({})
    const [openedTab, setOpenedTab] = useState<string | null>(null)

    useEffect(() => {
        const addLocalTabHandler = (
            data: CustomEvent<LocalEventsMap[LocalEvent.ADD_LOCAL_TAB]>
        ) => {
            setTabs((t) => {
                return {...t, [data.detail.name]: data.detail}
            })
        }

        const removeLocalTabHandler = (
            data: CustomEvent<LocalEventsMap[LocalEvent.REMOVE_LOCAL_TAB]>
        ) => {
            setOpenedTab(() => {
                setTabs(tabs => {
                    const newTabs = {...tabs}
                    delete newTabs[data.detail.name]
                    return newTabs
                })

                return null
            })
        }

        const openLocalTabHandler = (
            data: CustomEvent<LocalEventsMap[LocalEvent.OPEN_LOCAL_TAB]>
        ) => {
            if (!tabs[data.detail.name]) return

            setOpenedTab(() => data.detail.name)
        }

        const closeLocalTabHandler = (
            data: CustomEvent<LocalEventsMap[LocalEvent.CLOSE_LOCAL_TAB]>
        ) => {
            if (!tabs[data.detail.name]) return
            setOpenedTab(() => null)
        }

        eventBus.current.addEventListener(
            LocalEvent.ADD_LOCAL_TAB,
            addLocalTabHandler
        )

        eventBus.current.addEventListener(
            LocalEvent.REMOVE_LOCAL_TAB,
            removeLocalTabHandler
        )

        eventBus.current.addEventListener(
            LocalEvent.OPEN_LOCAL_TAB,
            openLocalTabHandler
        )

        eventBus.current.addEventListener(
            LocalEvent.CLOSE_LOCAL_TAB,
            closeLocalTabHandler
        )

        return () => {
            eventBus.current.removeEventListener(
                LocalEvent.ADD_LOCAL_TAB,
                addLocalTabHandler
            )

            eventBus.current.removeEventListener(
                LocalEvent.REMOVE_LOCAL_TAB,
                removeLocalTabHandler
            )

            eventBus.current.removeEventListener(
                LocalEvent.OPEN_LOCAL_TAB,
                openLocalTabHandler
            )

            eventBus.current.removeEventListener(
                LocalEvent.CLOSE_LOCAL_TAB,
                closeLocalTabHandler
            )
        }
    }, [tabs]);

    useTauriEvent(
        TauriEvent.TAB_CREATED,
        (tab) => {
            eventBus.current.dispatchEvent(
                new AddLocalTabEvent(
                    LocalEvent.ADD_LOCAL_TAB,
                    {detail: tab}
                )
            )
        },
        []
    )

    useTauriEvent(
        TauriEvent.TAB_REMOVED,
        (tab) => {
            if (openedTab === tab.name) eventBus.current.dispatchEvent(
                new CloseLocalTabEvent(
                    LocalEvent.CLOSE_LOCAL_TAB,
                    {detail: tab}
                )
            )

            eventBus.current.dispatchEvent(
                new RemoveLocalTabEvent(
                    LocalEvent.REMOVE_LOCAL_TAB,
                    {detail: tab}
                )
            )
        },
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