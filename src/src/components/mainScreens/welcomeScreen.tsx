import React, {Dispatch, SetStateAction, useContext, useEffect, useRef, useState} from "react";
import "./welcomeScreen.scss"
import Icon, {IconName} from "../components/icon.tsx";
import {open} from "@tauri-apps/plugin-dialog";
import {invoke} from "@tauri-apps/api/core";
import {EditorDataContext, TabContext} from "../app.tsx";
import {EditorData, EditorDataSendCommand} from "../lib/editor_data.ts";

export function WelcomeScreen() {
    const [cddaInstallDirectory, setCDDAInstallDirectory] = useState<string>()
    const [availableTilesets, setAvailableTilesets] = useState<string[]>([])
    const [selectedTilset, setSelectedTileset] = useState<string>("None")
    const [hasPickedCDDADirectory, setHasPickedCDDADirectory] = useState<boolean>(false)
    const selectRef = useRef<HTMLSelectElement>()
    const tabs = useContext(TabContext)

    useEffect(() => {
        if (!hasPickedCDDADirectory) return

        (async () => {
            // TODO: handle
            await invoke<unknown>(EditorDataSendCommand.TilesetPicked, {tileset: selectedTilset})
        })()
    }, [hasPickedCDDADirectory, selectedTilset]);

    async function onCDDAGameSelectClick() {
        const path = await open({
            multiple: false,
            directory: true,
        });

        if (!path) return;

        // TODO: Handle
        await invoke<unknown>(EditorDataSendCommand.CDDAInstallationDirectoryPicked, {path})
        setCDDAInstallDirectory(path)

        const data = await invoke<EditorData>(EditorDataSendCommand.GetEditorData, {})
        setAvailableTilesets(data.available_tilesets)
        setHasPickedCDDADirectory(true)
    }

    async function onSaveAndCloseClick() {
        if (!hasPickedCDDADirectory) {
            window.alert("You need to pick a CDDA install directory before proceeding")
            return
        }

        await invoke(EditorDataSendCommand.SaveEditorData, {})

        tabs.setOpenedTab(null)
        tabs.removeTab(0)
    }

    return (
        <main id={"welcome-main"}>
            <div id={"introduction-container"}>
                <h1>Welcome to the CDDA Map Editor!</h1>
                <p>This application is still in development and is expected to still contain bugs that the developer
                    hasn't
                    bothered to fix yet.</p>
                <p>First, please select the CDDA game installation directory</p>
                <button
                    onClick={onCDDAGameSelectClick}>{cddaInstallDirectory ? cddaInstallDirectory : "Select your CDDA game Installation directory"}</button>
                <div>
                    <p>Select a tileset if you want a graphical representation of your map. If you do not select
                        one, the tiles will be displayed as the characters they are mapped to</p>
                    <select value={selectedTilset}
                            onChange={() => {
                                if (selectRef.current.selectedIndex === 0) setSelectedTileset("None")
                                else setSelectedTileset(availableTilesets[selectRef.current.selectedIndex - 1])
                            }}
                            ref={selectRef} defaultValue={"None"}>
                        <option>None</option>
                        {
                            availableTilesets.map(t => <option key={t}>{t}</option>)
                        }
                    </select>
                </div>
                <p>
                    To get started with creating maps, click on the <span><Icon name={IconName.AddSmall}/></span> Icon
                    next to the "Welcome to the CDDA Map Editor" Tab to create a new Map</p>
                <p>
                    The previously selected settings can be changed anytime under the File {">"} Settings dropdown
                </p>
                {
                    hasPickedCDDADirectory &&
                    <button id={"tab-close-button"} onClick={onSaveAndCloseClick}>Save and close this tab</button>
                }
            </div>
        </main>
    )
}