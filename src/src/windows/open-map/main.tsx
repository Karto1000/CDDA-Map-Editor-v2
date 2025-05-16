import React, {useState} from "react";
import GenericWindow from "../generic-window.js";
import {open} from "@tauri-apps/plugin-dialog";
import "./main.scss"
import {getCurrentWindow} from "@tauri-apps/api/window";
import Icon, {IconName} from "../../shared/components/icon.js";
import MultiMenu from "../../shared/components/multimenu.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {TauriCommand} from "../../tauri/events/types.js";
import {OmTerrainType, OpenViewerData} from "../../tauri/types/viewer.js";

function MapViewer() {
    const [mapFilePath, setMapFilePath] = useState<string>("")
    const [projectName, setProjectName] = useState<string>("")
    const [omIds, setOmIds] = useState<string[][]>([[""]])
    const [deleteShownForColumn, setDeleteShownForColumn] = useState<number>(null)
    const [deleteShownForRow, setDeleteShownForRow] = useState<number>(null)

    async function onSubmit(e: React.FormEvent<HTMLFormElement>) {
        e.preventDefault()

        const isSingle = omIds.length === 1 && omIds[0].length === 1

        let data: OpenViewerData;
        if (isSingle) {
            data = {
                filePath: mapFilePath,
                projectName: projectName || omIds[0][0],
                omTerrain: {
                    type: OmTerrainType.Single,
                    omTerrainId: omIds[0][0]
                }
            }
        } else {
            data = {
                filePath: mapFilePath,
                projectName,
                omTerrain: {
                    type: OmTerrainType.Nested,
                    omTerrainIds: omIds
                }
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
                <div className={"form-element"}>
                    <input onChange={onProjectNameChange} placeholder={"Define a name for the project"}/>
                    <label>
                        The name of the project, a name must be selected if you are opening a nested map. If you
                        are opening a single map, the om_terrain is used as the name if no name is selected.
                    </label>
                </div>
                <div className={"grid-vertical-center"} onPaste={e => {
                    const pastedText = e.clipboardData.getData("text/plain")
                    const parsed = JSON.parse(pastedText)

                    if (!parsed[0] || !parsed[0][0]) return

                    setOmIds(parsed)
                }}>
                    <div className={"om-terrain-form-element-grid"}>
                        {
                            deleteShownForRow !== null &&
                            <button className={"delete-row-button"}
                                    style={{top: deleteShownForRow * 32}}
                                    type={"button"}
                                    onClick={() => {
                                        const newIds = omIds.filter((_, i) => i !== deleteShownForRow)
                                        setOmIds(newIds)

                                        setDeleteShownForRow(null)
                                        setDeleteShownForColumn(null)
                                    }}>
                                <Icon name={IconName.DeleteSmall}/>
                            </button>
                        }
                        {
                            deleteShownForColumn !== null &&
                            <button className={"delete-col-button"}
                                    style={{left: deleteShownForColumn * 128}}
                                    type={"button"}
                                    onClick={() => {
                                        const newIds = omIds.map(r => {
                                            const newRow = [...r]
                                            newRow.splice(deleteShownForColumn, 1)
                                            return newRow
                                        })

                                        setOmIds(newIds)

                                        setDeleteShownForColumn(null)
                                        setDeleteShownForRow(null)
                                    }}>
                                <Icon name={IconName.DeleteSmall}/>
                            </button>
                        }

                        <div className={"om-terrain-form-element"}>
                            {
                                omIds.map((rowIds, row) => {
                                    return (
                                        <div className={"om-terrain-row"} key={row}
                                             onMouseEnter={() => setDeleteShownForRow(row)}
                                        >
                                            {
                                                rowIds.map((id, col) => {
                                                    return (
                                                        <input className={"om-terrain-slot"}
                                                               key={`${row}-${col}`}
                                                               onMouseEnter={() => setDeleteShownForColumn(col)}
                                                               value={id}
                                                               onChange={e => {
                                                                   const newIds = [...omIds]
                                                                   newIds[row][col] = e.target.value
                                                                   setOmIds(newIds)
                                                               }}
                                                        />
                                                    )
                                                })
                                            }
                                        </div>
                                    )
                                })
                            }
                        </div>
                        <button className={"add-row-button"} onClick={() => {
                            const colLen = omIds[0].length || 1
                            const newIds = [...omIds, new Array(colLen).fill("")]

                            setOmIds(newIds)
                        }}><Icon
                            name={IconName.AddSmall}/></button>
                        <button className={"add-col-button"} onClick={() => {
                            const newIds = omIds.map(r => [...r, ""])
                            setOmIds(newIds)
                        }}><Icon
                            name={IconName.AddSmall}/></button>
                    </div>
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
