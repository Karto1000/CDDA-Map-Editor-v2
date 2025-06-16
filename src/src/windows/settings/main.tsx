import React, {useEffect, useRef, useState} from "react";
import GenericWindow from "../generic-window.js";
import {getCurrentWindow} from "@tauri-apps/api/window";
import "./main.scss"
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {EditorData, getKeybindingText, Keybind} from "../../tauri/types/editor.js";
import {BackendResponseType, TauriCommand} from "../../tauri/events/types.js";
import {clsx} from "clsx";
import {open} from "@tauri-apps/plugin-dialog";
import {DEFAULT_TILESET} from "../../features/editor/index.ts";
import {MultiMenu} from "../../shared/components/imguilike/multimenu.js";

function Main() {
    const [selectedTilset, setSelectedTileset] = useState<string>("None")
    const [cddaDirectoryPath, setCDDADirectoryPath] = useState<string>(null)
    const [editorData, setEditorData] = useState<EditorData>(null)
    const selectRef = useRef<HTMLSelectElement>(null)

    async function getAndSetEditorData() {
        const response = await tauriBridge.invoke<EditorData, unknown>(
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
            <div className={"settings-body"}>
                <MultiMenu tabs={
                    [
                        {
                            name: "General",
                            content: <div className={"general-settings"}>
                                <div className={"form-element"}>
                                    <label className={clsx("file-input", !cddaDirectoryPath && "placeholder")}>
                                        {cddaDirectoryPath ? cddaDirectoryPath : "Select your CDDA Game directory"}
                                        <button onClick={onCDDAInputChange}/>
                                    </label>
                                    <label>Change your CDDA Game directory</label>
                                </div>
                                <div className={"form-element"}>
                                    <button onClick={onThemeChange}>Change Theme</button>
                                    <label>Change your theme to dark or light</label>
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
                                    defaultValue={"None"}
                                >
                                    <option>None</option>
                                    {
                                        editorData?.available_tilesets.map(t => <option key={t}>{t}</option>)
                                    }
                                </select>
                                <label>Select your tileset here</label>
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
