import React, {useEffect, useRef, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {BackendResponseType, TauriCommand} from "../../tauri/events/types.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {getCurrentWindow} from "@tauri-apps/api/window";
import {open} from "@tauri-apps/plugin-dialog";
import {ProgramData} from "../../tauri/types/editor.js";
import Icon, {IconName} from "../../shared/components/icon.js";

function Main() {
    const [cddaInstallDirectory, setCDDAInstallDirectory] = useState<string>()
    const [availableTilesets, setAvailableTilesets] = useState<string[]>([])
    const [selectedTilset, setSelectedTileset] = useState<string>("None")
    const [hasPickedCDDADirectory, setHasPickedCDDADirectory] = useState<boolean>(false)
    const [isLoadingData, setIsLoadingData] = useState<boolean>(false)
    const selectRef = useRef<HTMLSelectElement>(null)

    useEffect(() => {
        if (!hasPickedCDDADirectory) return

        (async () => {
            await tauriBridge.invoke(
                TauriCommand.TILESET_PICKED,
                {
                    tileset: selectedTilset
                }
            )
        })()
    }, [hasPickedCDDADirectory, selectedTilset]);

    async function onCloseClicked() {
        // We want to close every single window here since the application cannot function without the cdda data
        await tauriBridge.invoke<null, null>(TauriCommand.CLOSE_APP, {})
    }

    async function onCDDAGameSelectClick() {
        const path = await open({
            multiple: false,
            directory: true,
        });

        if (!path) return;

        setIsLoadingData(true)

        await tauriBridge.invoke(
            TauriCommand.CDDA_INSTALLATION_DIRECTORY_PICKED,
            {
                path
            }
        )
        setCDDAInstallDirectory(path)

        const response = await tauriBridge.invoke<ProgramData, unknown>(
            TauriCommand.GET_EDITOR_DATA,
            {}
        )

        if (response.type === BackendResponseType.Error) return;

        setAvailableTilesets(response.data.available_tilesets)
        setHasPickedCDDADirectory(true)
        setIsLoadingData(false)
    }

    async function onSaveAndCloseClick() {
        if (!hasPickedCDDADirectory) {
            window.alert("You need to pick a CDDA install directory before proceeding")
            return
        }

        await tauriBridge.invoke(TauriCommand.SAVE_EDITOR_DATA, {})

        const tauriWindow = getCurrentWindow();
        await tauriWindow.close()
    }

    return (
        <GenericWindow title={"Welcome to the CDDA Map Editor!"} onCloseClicked={onCloseClicked}>
            <main id={"welcome-main"}>
                <div id={"introduction-container"}>
                    <h1>Welcome to the CDDA Map Editor / Viewer!</h1>
                    <p>This application is still in development and is expected to still contain bugs that the developer
                        hasn't bothered to fix yet.</p>
                    <p>First, please select the root of the CDDA game installation directory.
                        This application requires access to the json data of CDDA to display Nested Mapgen entries among
                        other things.</p>
                    <button
                        onClick={onCDDAGameSelectClick}>{cddaInstallDirectory ?
                        cddaInstallDirectory :
                        isLoadingData ?
                            "Loading..." :
                            "Select your CDDA game Installation directory"}
                    </button>
                    <div>
                        <p>Select a tileset if you want a graphical representation of your map. If you do not select
                            one, the tiles will be displayed using a fallback ascii tileset</p>
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
                        To get started with creating or viewing maps, click on the <span><Icon
                        name={IconName.AddSmall}/></span> Icon next to the application title to create a new Map.
                        Alternatively you can use the dropdown menu in the top left to create or import a map under
                        File &gt; Create or File &gt; Import
                    </p>
                    <p>
                        The previously selected settings can be changed anytime under the File {">"} Settings dropdown
                    </p>
                    {
                        hasPickedCDDADirectory &&
                        <button id={"tab-close-button"} onClick={onSaveAndCloseClick}>Save and close this
                            window</button>
                    }
                </div>
            </main>
        </GenericWindow>
    );
}

export default Main;
