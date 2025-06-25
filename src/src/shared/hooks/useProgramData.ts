import {Dispatch, RefObject, SetStateAction, useState} from "react";
import {ProgramData} from "../../tauri/types/editor.js";
import {useTauriEvent} from "./useTauriEvent.js";
import {TauriEvent} from "../../tauri/events/types.js";

export function useProgramData(): [ProgramData, Dispatch<SetStateAction<ProgramData>>] {
    const [programData, setProgramData] = useState<ProgramData>()

    useTauriEvent(
        TauriEvent.EDITOR_DATA_CHANGED,
        (editorData) => {
            setProgramData(editorData)
        },
        []
    )

    return [programData, setProgramData]
}