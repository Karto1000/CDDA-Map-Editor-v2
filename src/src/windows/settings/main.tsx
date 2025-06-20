import React, {useEffect, useRef, useState} from "react";
import GenericWindow from "../generic-window.js";
import {getCurrentWindow} from "@tauri-apps/api/window";
import "./main.scss"
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {getKeybindingText, ProgramData} from "../../tauri/types/editor.js";
import {BackendResponseType, TauriCommand, TauriEvent} from "../../tauri/events/types.js";
import {clsx} from "clsx";
import {open} from "@tauri-apps/plugin-dialog";
import {MultiMenu} from "../../shared/components/imguilike/multimenu.js";
import {DEFAULT_TILESET} from "../../features/sprites/tilesheets.js";
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {Tooltip} from "react-tooltip";
import {useTauriEvent} from "../../shared/hooks/useTauriEvent.js";

function Main() {
    const [selectedTilset, setSelectedTileset] = useState<string>("None")
    const [cddaDirectoryPath, setCDDADirectoryPath] = useState<string>(null)
    const [editorData, setEditorData] = useState<ProgramData>(null)
    const selectRef = useRef<HTMLSelectElement>(null)
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()

    async function getAndSetEditorData() {
        const response = await tauriBridge.invoke<ProgramData, unknown>(
            TauriCommand.GET_EDITOR_DATA,
            {}
        )

        if (response.type === BackendResponseType.Error) return;

        if (response.data.config.selected_tileset) {
            setSelectedTileset(response.data.config.selected_tileset)
        }

        if (response.data.config.cdda_path) {
            setCDDADirectoryPath(response.data.config.cdda_path)
        }

        setEditorData(response.data)
    }

    useEffect(() => {
        (async () => {
            await getAndSetEditorData()
        })()
    }, []);

    async function onThemeChange() {
        const window = getCurrentWindow();
        await window.emit("change-theme");
    }

    async function onTilesetSelect() {
        let newTileset: string;

        if (selectRef.current.selectedIndex === 0) newTileset = "None";
        else {
            newTileset = editorData.available_tilesets[selectRef.current.selectedIndex - 1]
        }

        await tauriBridge.invoke(
            TauriCommand.TILESET_PICKED,
            {
                tileset: newTileset
            }
        )

        setSelectedTileset(newTileset)
    }

    async function onCDDAInputChange() {
        const path = await open({
            multiple: false,
            directory: true,
        });

        if (!path) return;

        // Reset the tileset to none since we can't guarantee that the previously selected tileset is present in the new
        // directory
        setSelectedTileset(DEFAULT_TILESET)
        await tauriBridge.invoke(
            TauriCommand.TILESET_PICKED,
            {
                tileset: DEFAULT_TILESET
            }
        )

        await tauriBridge.invoke(
            TauriCommand.CDDA_INSTALLATION_DIRECTORY_PICKED,
            {
                path
            }
        )

        setCDDADirectoryPath(path)

        await tauriBridge.invoke(TauriCommand.SAVE_EDITOR_DATA, {})

        await getAndSetEditorData()
    }

    return (
        <GenericWindow title={"Settings"}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>

            <div className={"settings-body"}>
                <MultiMenu tabs={
                    [
                        {
                            name: "General",
                            content: <div className={"general-settings"}>
                                <div className={"form-element"}>
                                    <label
                                        className={clsx("file-input", !cddaDirectoryPath && "placeholder")}
                                        data-tooltip-id={"info-tooltip"}
                                        data-tooltip-content={"The path to the CDDA Game directory where the 'json' directory is located"}
                                        onMouseMove={handleMouseMove}
                                    >
                                        {cddaDirectoryPath ? cddaDirectoryPath : "Select your CDDA Game directory"}
                                        <button onClick={onCDDAInputChange}/>
                                    </label>
                                    <label>CDDA Path</label>
                                </div>
                                <div className={"form-element"}>
                                    <button
                                        onClick={onThemeChange}
                                        data-tooltip-id={"info-tooltip"}
                                        data-tooltip-content={"Change the theme of the application"}
                                        onMouseMove={handleMouseMove}
                                    >Change Theme
                                    </button>
                                    <label>Theme</label>
                                </div>
                            </div>
                        },
                        {
                            name: "Graphics",
                            content: <div className={"form-element"}>
                                <select
                                    value={selectedTilset}
                                    onChange={onTilesetSelect}
                                    ref={selectRef}
                                    data-tooltip-id={"info-tooltip"}
                                    data-tooltip-html={"The currently selected tileset. If you don't select a tileset, <br/> the tiles will be displayed using a fallback ascii tileset."}
                                    onMouseMove={handleMouseMove}
                                    defaultValue={"None"}
                                >
                                    <option>None</option>
                                    {
                                        editorData?.available_tilesets.map(t => <option key={t}>{t}</option>)
                                    }
                                </select>
                                <label>Tileset</label>
                            </div>
                        },
                        {
                            name: "Keybinds",
                            content: <div className={"keybindings-container"}>
                                {
                                    editorData?.config.keybinds.map(kb => (
                                        <div className={"keybinding"}>
                                            <span>{getKeybindingText(kb)}</span>
                                            <span>{kb.action}</span>
                                        </div>
                                    ))
                                }
                            </div>
                        }
                    ]
                }/>
            </div>
        </GenericWindow>
    );
}

export default Main;
