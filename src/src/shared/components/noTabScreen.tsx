import React, {RefObject, useContext, useRef} from "react"
import "./noTabScreen.scss"
import {openWindow, WindowLabel} from "../../windows/lib.js";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {ThemeContext} from "../../app.js";
import {UnlistenFn} from "@tauri-apps/api/event";

type Props = {
    importMapWindowRef: RefObject<WebviewWindow>
    newMapWindowRef: RefObject<WebviewWindow>
}

export function NoTabScreen(props: Props) {
    const {theme} = useContext(ThemeContext)

    const importUnlistenFn = useRef<UnlistenFn>(null)
    const newUnlistenFn = useRef<UnlistenFn>(null)

    function onOpenClicked() {
        alert("TBD")
    }

    async function onCreateClicked() {
        newUnlistenFn.current = (await openWindow(WindowLabel.NewMap, theme, props.newMapWindowRef))[1]
    }

    async function onImportClicked() {
         importUnlistenFn.current = (await openWindow(WindowLabel.ImportMap, theme, props.importMapWindowRef))[1]
    }

    return (
        <main id={"startupMain"}>
            <div id={"centerOptions"}>
                <div>
                    <span className={"interactable-text"} onClick={onOpenClicked}>Open</span> an existing Map
                </div>
                <div>
                    <span className={"interactable-text"} onClick={onCreateClicked}>Create</span> a new Map
                </div>
                <div>
                    <span className={"interactable-text"} onClick={onImportClicked}>Import</span> a Mapgen File
                </div>
            </div>
        </main>
    )
}