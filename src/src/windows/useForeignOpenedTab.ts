import {useEffect, useRef, useState} from "react";
import {__TAB_CHANGED, TauriEvent} from "../tauri/events/types.js";
import {listen, UnlistenFn} from "@tauri-apps/api/event";
import {useTauriEvent} from "../shared/hooks/useTauriEvent.js";

export function useForeignOpenedTab(): string {
    const [openedTab, setOpenedTab] = useState<string>(null)

    useTauriEvent(
        TauriEvent.OPEN_TAB,
        e => {
            setOpenedTab(e.name)
        },
        [openedTab]
    )

    return openedTab
}