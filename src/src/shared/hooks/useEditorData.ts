import {RefObject, useState} from "react";
import {EditorData} from "../../tauri/types/editor.js";
import {useTauriEvent} from "./useTauriEvent.js";
import {TauriEvent} from "../../tauri/events/types.js";

export function useEditorData(): EditorData {
    const [editorData, setEditorData] = useState<EditorData>()

    useTauriEvent(
        TauriEvent.EDITOR_DATA_CHANGED,
        (editorData) => {
            console.log(editorData)
            setEditorData(editorData)
        },
        []
    )

    return editorData
}