import React, {useState} from "react";
import GenericWindow from "../generic-window.js";
import {open} from "@tauri-apps/plugin-dialog";
import "./main.scss"
import {getCurrentWindow} from "@tauri-apps/api/window";
import MultiMenu from "../../shared/components/multimenu.js";
import {OpenViewerData, OpenViewerDataType} from "../../tauri/types/viewer.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {TauriCommand} from "../../tauri/events/types.js";

function MapViewer() {
    const [omFilePaths, setOmFilePaths] = useState<string[]>([])
    const [mapgenFilePaths, setMapgenFilePaths] = useState<string[]>([])
    const [projectName, setProjectName] = useState<string>("")
    const [omSpecialOrTerrainId, setOmSpecialOrTerrainId] = useState<string>("")
    const [creatingType, setCreatingType] = useState<OpenViewerDataType>(OpenViewerDataType.Terrain)

    async function onSubmit(e: React.FormEvent<HTMLFormElement>) {
        e.preventDefault()

        let data: OpenViewerData;
        if (creatingType === OpenViewerDataType.Terrain) {
            data = {
                type: OpenViewerDataType.Terrain,
                mapgenFilePaths: mapgenFilePaths,
                projectName: projectName,
                omId: omSpecialOrTerrainId,
            }
        } else if (creatingType === OpenViewerDataType.Special) {
            data = {
                type: OpenViewerDataType.Special,
                mapgenFilePaths: mapgenFilePaths,
                omFilePaths: omFilePaths,
                projectName: projectName,
                omId: omSpecialOrTerrainId,
            }
        }

        await tauriBridge.invoke(
            TauriCommand.OPEN_VIEWER,
            {
                data
            }
        )

        const window = getCurrentWindow()
        await window.close()
    }

    function onProjectNameChange(e: React.ChangeEvent<HTMLInputElement>) {
        setProjectName(e.target.value)
    }

    function onOmTerrainIdChange(e: React.ChangeEvent<HTMLInputElement>) {
        setOmSpecialOrTerrainId(e.target.value)
    }

    async function onOmFileInputChange(e: React.MouseEvent<HTMLButtonElement>) {
        e.preventDefault()

        const selected = await open(
            {
                multiple: true,
                filters: [
                    {
                        name: "Json",
                        extensions: ["json"]
                    }
                ]
            }
        )

        if (!selected) return;

        setOmFilePaths(selected)
    }

    async function onMapFileInputChange(e: React.MouseEvent<HTMLButtonElement>) {
        e.preventDefault()

        const selected = await open(
            {
                multiple: true,
                filters: [
                    {
                        name: "Json",
                        extensions: ["json"]
                    }
                ]
            }
        )

        if (!selected) return;

        setMapgenFilePaths(selected)
    }

    return (
        <div className={"map-viewer-body"}>
            <p>
                A map viewer is ideal if you don't want to actually use the map editor to create a map,
                but still want to see what the map looks like without having to open the main game.
                The map will be automatically reloaded once it detects a change to the map file which is
                currently open.
            </p>
            <MultiMenu
                onTabSelected={t => {
                    setCreatingType(t.name === "Overmap Terrain" ? OpenViewerDataType.Terrain : OpenViewerDataType.Special)
                }}
                tabs={
                    [
                        {
                            name: "Overmap Terrain",
                            content: <form onSubmit={onSubmit} className={"map-viewer-form"}>
                                <div className={"map-viewer-form-terrain"}>
                                    <div className={"form-element"}>
                                        <label className={"file-input"}>
                                            {mapgenFilePaths.length > 0 ? mapgenFilePaths : "Select a mapgen File Path"}
                                            <button onClick={onMapFileInputChange}/>
                                        </label>
                                        <label>
                                            The path to the files where the mapgen entries are stored
                                        </label>
                                    </div>
                                    <div className={"form-element"}>
                                        <input onChange={onOmTerrainIdChange} placeholder={"Enter the overmap id"}/>
                                        <label>
                                            The overmap id which is defined in the mapgen file as "om_terrain".
                                        </label>
                                    </div>
                                    <div className={"form-element"}>
                                        <input onChange={onProjectNameChange}
                                               placeholder={"Define a name for the project"}/>
                                        <label>
                                            The name of the project
                                        </label>
                                    </div>
                                </div>
                                <button type={"submit"}>Open</button>
                            </form>
                        },
                        {
                            name: "Overmap Special",
                            content: <form onSubmit={onSubmit} className={"map-viewer-form"}>
                                <div className={"map-viewer-form-special"}>
                                    <div className={"form-element"}>
                                        <label className={"file-input"}>
                                            {omFilePaths.length > 0 ? omFilePaths : "Select a Overmap special File Path"}
                                            <button onClick={onOmFileInputChange}/>
                                        </label>
                                        <label>
                                            The path to one or more overmap special files to search for the overmap id
                                        </label>
                                    </div>
                                    <div className={"form-element"}>
                                        <label className={"file-input"}>
                                            {mapgenFilePaths.length > 0 ? mapgenFilePaths : "Select a mapgen File Path"}
                                            <button onClick={onMapFileInputChange}/>
                                        </label>
                                        <label>
                                            The path to the files where the map data which is referenced in the overmap
                                            special
                                            files is stored.
                                        </label>
                                    </div>
                                    <div className={"form-element"}>
                                        <input onChange={onOmTerrainIdChange}
                                               placeholder={"Enter the overmap special id"}/>
                                        <label>
                                            The overmap special id. The overmap special entry is used to combine
                                            multiple mapgen
                                            entries into one.
                                        </label>
                                    </div>
                                    <div className={"form-element"}>
                                        <input onChange={onProjectNameChange}
                                               placeholder={"Define a name for the project"}/>
                                        <label>
                                            The name of the project
                                        </label>
                                    </div>
                                </div>
                                <button type={"submit"}>Open</button>
                            </form>
                        }
                    ]
                }/>
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
