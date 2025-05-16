import {MutableRefObject, useState} from "react";
import {EditorData} from "../../tauri/types/editor.js";
import {useTauriEvent} from "./useTauriEvent.js";
import {TauriCommand, TauriEvent} from "../../tauri/events/types.js";
import {TabTypeKind} from "./useTabs.js";
import {LocalEvent} from "../utils/localEvent.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";

export function useEditorData(eventBus: MutableRefObject<EventTarget>): EditorData {
    const [editorData, setEditorData] = useState<EditorData>()

    useTauriEvent(
        TauriEvent.EDITOR_DATA_CHANGED,
        async (editorData) => {
            console.log("Received editor data changed event: ", editorData, "")
            setEditorData(editorData)

            if (!editorData.config.cdda_path) {
                eventBus.current.dispatchEvent(
                    new CustomEvent(
                        LocalEvent.ADD_LOCAL_TAB,
                        {
                            detail: {
                                name: "Welcome to the CDDA Map Editor",
                                tab_type: TabTypeKind.Welcome,
                            }
                        }
                    )
                )

                eventBus.current.dispatchEvent(
                    new CustomEvent(
                        LocalEvent.OPEN_LOCAL_TAB,
                        {
                            detail: "Welcome to the CDDA Map Editor"
                        }
                    )
                )
            }
        },
        [eventBus]
    )

    return editorData
}