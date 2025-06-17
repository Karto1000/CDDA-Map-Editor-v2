import {Project} from "../../tauri/types/editor.js";
import {RefObject, useEffect, useRef, useState} from "react";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {BackendResponseType, TauriCommand} from "../../tauri/events/types.js";
import {UseTabsReturn} from "./useTabs.js";

export function useCurrentProject<T>(tabs: UseTabsReturn): Project<T> {
    const [currentProject, setCurrentProject] = useState<Project<T>>(null)

    useEffect(() => {
        (async() => {
           const response = await tauriBridge.invoke<Project<T>, string>(TauriCommand.GET_CURRENT_PROJECT_DATA, {})

            if (response.type === BackendResponseType.Error) {
                return
            }

            setCurrentProject(response.data)
        })()
    }, [tabs.openedTab]);

    return currentProject
}