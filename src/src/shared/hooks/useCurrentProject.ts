import {Project} from "../../tauri/types/editor.js";
import {useEffect, useRef, useState} from "react";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {BackendResponseType, TauriCommand, TauriEvent} from "../../tauri/events/types.js";
import {UseTabsReturn} from "./useTabs.js";
import {listen, UnlistenFn} from "@tauri-apps/api/event";

export function useCurrentProject<T>(openedTab: string): Project<T> {
    const [currentProject, setCurrentProject] = useState<Project<T>>(null)
    const unlistenRef = useRef<UnlistenFn>(null)

    // useEffect(() => {
    //     (async () => {
    //         unlistenRef.current = await listen<Project<T>>(
    //             TauriEvent.CURRENT_PROJECT_CHANGED,
    //             p => setCurrentProject(p.payload)
    //         )
    //     })()
    //
    //     return () => {
    //         if (unlistenRef.current) unlistenRef.current()
    //     }
    // }, [currentProject]);

    useEffect(() => {
        (async () => {
            const response = await tauriBridge.invoke<Project<T>, string>(TauriCommand.GET_CURRENT_PROJECT_DATA, {})

            if (response.type === BackendResponseType.Error) {
                return
            }
            setCurrentProject(response.data)
        })()
    }, [openedTab]);

    return currentProject
}