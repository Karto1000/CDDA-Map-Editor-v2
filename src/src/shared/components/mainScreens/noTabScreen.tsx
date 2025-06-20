import React, {RefObject, useContext} from "react"
import "./noTabScreen.scss"
import {openWindow, WindowLabel} from "../../../windows/lib.js";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {ThemeContext} from "../../../app.js";

type Props = {
    openMapWindowRef: RefObject<WebviewWindow>
    newMapWindowRef: RefObject<WebviewWindow>
}

export function NoTabScreen(props: Props) {
    const {theme} = useContext(ThemeContext)

    function onOpenClicked() {
        props.openMapWindowRef.current = openWindow(WindowLabel.ImportMap, theme)
    }

    function onCreateClicked() {
        props.newMapWindowRef.current = openWindow(WindowLabel.NewMap, theme)
    }

    function onImportClicked() {
        alert("TBD")
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