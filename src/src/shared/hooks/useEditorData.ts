import {RefObject, useState} from "react";
import {ProgramData} from "../../tauri/types/editor.js";
import {useTauriEvent} from "./useTauriEvent.js";
import {TauriEvent} from "../../tauri/events/types.js";

export function useEditorData(): ProgramData {
    const [editorData, setEditorData] = useState<ProgramData>()

    useTauriEvent(
        TauriEvent.EDITOR_DATA_CHANGED,
        (editorData) => {
            setEditorData(editorData)
        },
        []
    )

    return editorData
}