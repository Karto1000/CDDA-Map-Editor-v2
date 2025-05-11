import React, {useState} from "react";
import GenericWindow from "../generic-window.js";
import MultiMenu from "../../components/multimenu.js";
import {open} from "@tauri-apps/plugin-dialog";

function MapViewer() {
    const [mapFilePath, setMapFilePath] = useState<string>("")
    const [overmapTerrainId, setOvermapTerrainId] = useState<string>("")

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
        <div>
            <p>
                A map viewer is ideal if you don't want to actually use the map editor to create a map,
                but still want to see what the map looks like without having to open the main game.
                The map will be automatically reloaded once it detects a change to the map file which is
                currently open.
            </p>
            <form onSubmit={onSubmit}>
                <div className={"form-element"}>
                    <label className={"file-input"}>
                        {mapFilePath ? mapFilePath : "Select a Map File Path"}
                        <button onClick={onFileInputChange}/>
                    </label>
                    <label>
                        The path to the map file
                    </label>
                </div>
                <div className={"form-element"}>
                    <input
                        type={"text"}
                        name={"overmap-terrain-id"}
                        placeholder={"Overmap terrain Id"}
                        value={overmapTerrainId} onChange={(e) => setOvermapTerrainId(e.target.value)}
                    />
                    <label htmlFor={"overmap-terrain-id"}>The overmap terrain id</label>
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
