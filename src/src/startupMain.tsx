import React from "react"
import "./startupMain.scss"

export function StartupMain() {
    function onOpenClicked() {
        alert("TBD")
    }

    function onCreateClicked() {
        alert("TBD")
    }

    function onImportClicked() {
        alert("TBD")
    }

    return (
        <main id={"startupMain"}>
            <div id={"centerOptions"}>
                <div>
                    <span className={"interactable-text"} onClick={onOpenClicked}>Open</span> an existing Project
                </div>
                <div>
                    <span className={"interactable-text"} onClick={onCreateClicked}>Create</span> a new Project
                </div>
                <div>
                    <span className={"interactable-text"} onClick={onImportClicked}>Import</span> a Mapgen File
                </div>
            </div>
        </main>
    )
}