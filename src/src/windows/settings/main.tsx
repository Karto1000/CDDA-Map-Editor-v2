import React, {useEffect, useRef, useState} from "react";
import GenericWindow from "../generic-window.js";
import {getCurrentWindow} from "@tauri-apps/api/window";
import {Accordion} from "../../shared/components/imguilike/accordion.js";
import "./main.scss"
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {EditorData} from "../../tauri/types/editor.js";
import {BackendResponseType, TauriCommand} from "../../tauri/events/types.js";

function Main() {
    const [availableTilesets, setAvailableTilesets] = useState<string[]>([])
    const [selectedTilset, setSelectedTileset] = useState<string>("None")
    const selectRef = useRef<HTMLSelectElement>(null)

    useEffect(() => {
        (async () => {
            const response = await tauriBridge.invoke<
                EditorData,
                unknown,
                TauriCommand.GET_EDITOR_DATA
            >(
                TauriCommand.GET_EDITOR_DATA,
                {}
            )

            if (response.type === BackendResponseType.Error) return;

            setAvailableTilesets(response.data.available_tilesets)

            if (response.data.config.selected_tileset) {
                setSelectedTileset(response.data.config.selected_tileset)
            }
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
            newTileset = availableTilesets[selectRef.current.selectedIndex - 1]
        }

        await tauriBridge.invoke(
            TauriCommand.TILESET_PICKED,
            {
                tileset: newTileset
            }
        )

        setSelectedTileset(newTileset)
    }

    return (
        <GenericWindow title={"Settings"}>
            <div className={"settings-body"}>
                <Accordion title={"General"}>
                    <button onClick={onThemeChange}>Change Theme</button>
                </Accordion>
                <Accordion title={"Graphics"}>
                    <div className={"form-element"}>
                        <select
                            value={selectedTilset}
                            onChange={onTilesetSelect}
                            ref={selectRef}
                            defaultValue={"None"}
                        >
                            <option>None</option>
                            {
                                availableTilesets.map(t => <option key={t}>{t}</option>)
                            }
                        </select>
                        <label>Select your tileset here</label>
                    </div>
                </Accordion>
            </div>
        </GenericWindow>
    );
}

export default Main;
