import {useState} from "react";

export enum TabType {
    Welcome,
    MapEditor,
    LiveViewer
}

export type Tab = {
    name: string,
    icon: null,
    type: TabType
}

export type UseTabsReturn = {
    tabs: Tab[],
    openedTab: number,
    addTab: (tab: Tab) => void,
    removeTab: (index: number) => void,
    setOpenedTab: (index: number) => void
}

export function useTabs(): UseTabsReturn {
    const [tabs, setTabs] = useState<Tab[]>([])
    const [openTab, setOpenTab] = useState<number | null>(null)

    function addTab(tab: Tab) {
        setTabs([...tabs, tab])
    }

    function removeTab(index: number) {
        const newTabs = [...tabs]
        newTabs.splice(index, 1)
        setTabs(newTabs)
    }

    function setOpenedTab(index: number) {
        setOpenTab(index)
    }

    return {tabs, addTab, removeTab, openedTab: openTab, setOpenedTab}
}