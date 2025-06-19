import React, {useState} from "react";
import GenericWindow from "../generic-window.js";
import {open, save} from "@tauri-apps/plugin-dialog";
import "./main.scss"
import {getCurrentWindow} from "@tauri-apps/api/window";
import {OpenViewerData, OpenViewerDataType} from "../../tauri/types/viewer.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {TauriCommand} from "../../tauri/events/types.js";
import {MultiMenu} from "../../shared/components/imguilike/multimenu.js";
import {clsx} from "clsx";
import {Tooltip} from "react-tooltip";
import {MouseMoveHandler, useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";

function OpenMapViewer({handleMouseMove}: { handleMouseMove: MouseMoveHandler }) {
    const [omFilePaths, setOmFilePaths] = useState<string[]>([])
    const [mapgenFilePaths, setMapgenFilePaths] = useState<string[]>([])
    const [projectName, setProjectName] = useState<string>("")
    const [omSpecialOrTerrainId, setOmSpecialOrTerrainId] = useState<string>("")
    const [creatingType, setCreatingType] = useState<OpenViewerDataType>(OpenViewerDataType.Terrain)

    async function onSubmit(e: React.FormEvent<HTMLFormElement>) {
        e.preventDefault()

        const projectSavePath = await save({
            filters: [
                {
                    name: "Json",
                    extensions: ["json"]
                }
            ]
        })

        if (!projectSavePath) return;

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
            TauriCommand.CREATE_VIEWER,
            {
                projectSavePath,
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
                Import a Mapgen File or a Overmap Special as a Map Viewer.

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
                                        <label
                                            className={clsx("file-input", mapgenFilePaths.length === 0 && "placeholder")}
                                            data-tooltip-id={"info-tooltip"}
                                            data-tooltip-html="The paths to the files where the mapgen entries are stored"
                                            onMouseMove={handleMouseMove}>
                                            {mapgenFilePaths.length > 0 ? mapgenFilePaths : "Select one or more mapgen File Paths"}
                                            <button onClick={onMapFileInputChange}/>
                                        </label>
                                        <label>Mapgen Paths</label>
                                    </div>
                                    <div className={"form-element"}>
                                        <input
                                            onChange={onOmTerrainIdChange}
                                            placeholder={"Enter the overmap id"}
                                            data-tooltip-id={"info-tooltip"}
                                            data-tooltip-html="The overmap id which is defined in the mapgen file as om_terrain."
                                            onMouseMove={handleMouseMove}
                                        />
                                        <label>Overmap Id</label>
                                    </div>
                                    <div className={"form-element"}>
                                        <input
                                            onChange={onProjectNameChange}
                                            placeholder={"Define a name for the project"}
                                            data-tooltip-id={"info-tooltip"}
                                            data-tooltip-html="The name of the project"
                                            onMouseMove={handleMouseMove}
                                        />
                                        <label>Project Name</label>
                                    </div>
                                </div>
                                <button type={"submit"}>Import</button>
                            </form>
                        },
                        {
                            name: "Overmap Special",
                            content: <form onSubmit={onSubmit} className={"map-viewer-form"}>
                                <div className={"map-viewer-form-special"}>
                                    <div className={"form-element"}>
                                        <label
                                            className={clsx("file-input", omFilePaths.length === 0 && "placeholder")}
                                            data-tooltip-id={"info-tooltip"}
                                            data-tooltip-html="The path to one or more overmap special files to search for the overmap id"
                                            onMouseMove={handleMouseMove}
                                        >
                                            {omFilePaths.length > 0 ? omFilePaths : "Select one or more Overmap special File Paths"}
                                            <button onClick={onOmFileInputChange}/>
                                        </label>
                                        <label>Overmap Special Paths</label>
                                    </div>
                                    <div className={"form-element"}>
                                        <label
                                            className={clsx("file-input", mapgenFilePaths.length === 0 && "placeholder")}
                                            data-tooltip-id={"info-tooltip"}
                                            data-tooltip-html="The path to the files where the map data which is referenced in the overmap special files is stored."
                                            onMouseMove={handleMouseMove}
                                        >
                                            {mapgenFilePaths.length > 0 ? mapgenFilePaths : "Select one or more mapgen File Paths"}
                                            <button onClick={onMapFileInputChange}/>
                                        </label>
                                        <label>Mapgen Paths</label>
                                    </div>
                                    <div className={"form-element"}>
                                        <input onChange={onOmTerrainIdChange}
                                               placeholder={"Enter the overmap special id"}
                                               data-tooltip-id={"info-tooltip"}
                                               data-tooltip-html="The overmap special id. The overmap special entry is used to combine multiple mapgen entries into one."
                                               onMouseMove={handleMouseMove}
                                        />
                                        <label>Overmap Special Id</label>
                                    </div>
                                    <div className={"form-element"}>
                                        <input onChange={onProjectNameChange}
                                               placeholder={"The name of the project"}
                                               data-tooltip-id={"info-tooltip"}
                                               data-tooltip-html="The name of the project"
                                               onMouseMove={handleMouseMove}
                                        />
                                        <label>Project Name</label>
                                    </div>
                                </div>
                                <button type={"submit"}>Import</button>
                            </form>
                        }
                    ]
                }/>
        </div>
    )
}

function Main() {
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()

    return (
        <GenericWindow title={"Import Map"}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>

            <MultiMenu tabs={[
                {
                    name: "Map Editor",
                    content: <></>,
                    isDisabled: true
                },
                {
                    name: "Map Viewer",
                    content: <OpenMapViewer handleMouseMove={handleMouseMove}/>,
                }
            ]}/>
        </GenericWindow>
    );
}

export default Main;
