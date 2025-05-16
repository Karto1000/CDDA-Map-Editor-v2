import React, {Dispatch, SetStateAction} from "react"
import "./noTabScreen.scss"

type Props = {
    setIsCreatingMapWindowOpen: Dispatch<SetStateAction<boolean>>
}

export function NoTabScreen(props: Props) {
    function onOpenClicked() {
        alert("TBD")
    }

    function onCreateClicked() {
        props.setIsCreatingMapWindowOpen(true)
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