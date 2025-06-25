import React, {useEffect, useRef, useState} from "react";
import GenericWindow from "../generic-window.js";
import {getCurrentWindow} from "@tauri-apps/api/window";
import "./main.scss"
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {getKeybindingText, ProgramData} from "../../tauri/types/editor.js";
import {BackendResponseType, TauriCommand, TauriEvent} from "../../tauri/events/types.js";
import {clsx} from "clsx";
import {ask, open} from "@tauri-apps/plugin-dialog";
import {MultiMenu} from "../../shared/components/imguilike/multimenu.js";
import {DEFAULT_TILESET} from "../../features/sprites/tilesheets.js";
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {Tooltip} from "react-tooltip";
import Icon, {IconName} from "../../shared/components/icon.tsx";
import {useProgramData} from "../../shared/hooks/useProgramData.js";
import {emit} from "@tauri-apps/api/event";

function Main() {
    const [selectedTilset, setSelectedTileset] = useState<string>("None")
    const [cddaDirectoryPath, setCDDADirectoryPath] = useState<string>(null)
    const selectRef = useRef<HTMLSelectElement>(null)
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()
    const [programData, setProgramData] = useProgramData()

    async function getAndSetProgramData() {
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

        setProgramData(response.data)
    }

    useEffect(() => {
        (async () => {
            await getAndSetProgramData()
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
            newTileset = programData.available_tilesets[selectRef.current.selectedIndex - 1]
        }

        await tauriBridge.invoke(
            TauriCommand.TILESET_PICKED,
            {
                tileset: newTileset
            }
        )

        setSelectedTileset(newTileset)
    }

    async function pickCDDADirectory(path: string) {
        await tauriBridge.invoke(
            TauriCommand.CDDA_INSTALLATION_DIRECTORY_PICKED,
            {
                path
            }
        )

        setCDDADirectoryPath(path)

        await tauriBridge.invoke(TauriCommand.SAVE_EDITOR_DATA, {})

        await getAndSetProgramData()
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

        await pickCDDADirectory(path)
    }

    async function onReloadClick() {
        await pickCDDADirectory(cddaDirectoryPath)
    }

    async function openDataDirectory() {
        await tauriBridge.invoke(
            TauriCommand.SHOW_PROGRAM_DARA_DIRECTORY,
            {}
        )
    }

    async function onRestoreDefaultsClick() {
        const answer = await ask(
            'Are you sure you want to restore the default settings?', {
                title: 'Map Editor',
                kind: 'warning',
            }
        );

        if (!answer) return;

        await emit(TauriEvent.CLOSE_ALL_TABS)

        await tauriBridge.invoke(
            TauriCommand.RESTORE_DEFAULT_CONFIG,
            {}
        )

        const window = getCurrentWindow();
        await window.close()
    }

    return (
        <GenericWindow title={"Settings"}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>

            <div className={"settings-body"}>
                <div className={"form-element"}
                     data-tooltip-id={"info-tooltip"}
                     data-tooltip-html={"Open the data directory of the editor"}
                     onMouseMove={handleMouseMove}>
                    <button onClick={openDataDirectory}>Open data directory</button>
                    <label>Open data directory</label>
                </div>
                <div className={"form-element"}
                     data-tooltip-id={"info-tooltip"}
                     data-tooltip-html={"Restores the default settings of the application. " +
                         "<b>This will NOT delete projects you have already created</b> "}
                     onMouseMove={handleMouseMove}>
                    <button onClick={onRestoreDefaultsClick}>Restore Defaults</button>
                    <label>Restore defaults </label>
                </div>

                <MultiMenu tabs={
                    [
                        {
                            name: "General",
                            content: <div className={"general-settings"}>
                                <div className={"form-element"}>
                                    {
                                        cddaDirectoryPath &&
                                        <button className={"reload-button"}
                                                data-tooltip-id={"info-tooltip"}
                                                data-tooltip-content={"Reload the game data"}
                                                onMouseMove={handleMouseMove}
                                                onClick={onReloadClick}
                                        >
                                            <Icon name={IconName.ReloadMedium}/>
                                        </button>
                                    }
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
                            content:
                                <div className={"graphics-settings"}>
                                    <div className={"form-element"}>
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
                                                programData?.available_tilesets.map(t => <option key={t}>{t}</option>)
                                            }
                                        </select>
                                        <label>Tileset</label>
                                    </div>
                                </div>
                        },
                        {
                            name: "Keybinds",
                            content:
                                <div className={"keybinds-settings"}>
                                    <div className={"keybindings-container"}>
                                        {
                                            programData?.config.keybinds.map(kb => (
                                                <div className={"keybinding"}>
                                                    <span>{getKeybindingText(kb)}</span>
                                                    <span>{kb.action}</span>
                                                </div>
                                            ))
                                        }
                                    </div>
                                </div>
                        }
                    ]
                }/>
            </div>
        </GenericWindow>
    );
}

export default Main;
