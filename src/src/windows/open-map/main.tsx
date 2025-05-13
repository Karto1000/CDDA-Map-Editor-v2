import React, {useState} from "react";
import GenericWindow from "../generic-window.js";
import MultiMenu from "../../components/multimenu.js";
import {open} from "@tauri-apps/plugin-dialog";
import "./main.scss"

function MapViewer() {
    const [mapFilePath, setMapFilePath] = useState<string>("")
    const [rows, setRows] = useState<number>(1)
    const [columns, setColumns] = useState<number>(1)

    function onSubmit(e: React.FormEvent<HTMLFormElement>) {
        e.preventDefault()
    }

    async function onFileInputChange(e: React.MouseEvent<HTMLButtonElement>) {
        e.preventDefault()

        const selected = await open(
            {
                multiple: false,
                filters: [
                    {
                        name: "Json",
                        extensions: ["json"]
                    }
                ]
            }
        )

        setMapFilePath(selected)
    }

    return (
        <div className={"map-viewer-body"}>
            <p>
                A map viewer is ideal if you don't want to actually use the map editor to create a map,
                but still want to see what the map looks like without having to open the main game.
                The map will be automatically reloaded once it detects a change to the map file which is
                currently open.
            </p>
            <form onSubmit={onSubmit} className={"map-viewer-form"}>
                <div className={"form-element"}>
                    <label className={"file-input"}>
                        {mapFilePath ? mapFilePath : "Select a Map File Path"}
                        <button onClick={onFileInputChange}/>
                    </label>
                    <label>
                        The path to the map file
                    </label>
                </div>
                <div className={"om-terrain-form-element"}>
                    {
                     Array(rows).keys().map(() => {
                         return (
                             <div className={"om-terrain-row"}>
                                 {
                                     Array(columns).keys().map(() => {
                                         return (
                                             <div className={"om-terrain-slot"}/>
                                         )
                                     })
                                 }
                             </div>
                         )
                     })
                    }
                </div>

                <button type={"submit"}>Open</button>
            </form>
        </div>
    )
}

function Main() {
    return (
        <GenericWindow title={"Open Map"}>
            <MultiMenu tabs={[
                {
                    name: "New Map Editor",
                    content: <></>,
                    isDisabled: true
                },
                {
                    name: "New Map Viewer",
                    content: <MapViewer/>,
                }
            ]}/>
        </GenericWindow>
    );
}

export default Main;
