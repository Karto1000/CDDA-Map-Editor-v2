import React, {RefObject, useEffect, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {useCurrentProject} from "../../shared/hooks/useCurrentProject.js";
import {MapEditorData} from "../../tauri/types/editor.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {MapGenValue} from "../../tauri/types/map_data.js";
import {BackendResponseType, CharacterMapping, TauriCommand, TauriEvent} from "../../tauri/events/types.js";
import {useForeignOpenedTab} from "../useForeignOpenedTab.js";
import {TilesheetSprite} from "../../shared/components/tilesheetSprite.js";
import {SlimTilesheets} from "../../features/sprites/slimTilesheets.js";
import {SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {useInitialData} from "../useInitialData.js";
import {useTauriEvent} from "../../shared/hooks/useTauriEvent.js";

type GlobalPaletteReprResponseData = { [char: string]: CharacterMapping }

export type InitialGlobalPaletteData = {
    slimTilesheets: SlimTilesheets,
    spritesheetConfig: RefObject<SpritesheetConfig>
}

function Main() {
    const [globalPalettes, setGlobalPalettes] = useState<MapGenValue[]>([])
    const openedTab = useForeignOpenedTab()
    const project = useCurrentProject<MapEditorData>(openedTab)
    const [globalPaletteReprs, setGlobalPaletteReprs] = useState<GlobalPaletteReprResponseData>({})
    const initialData = useInitialData<InitialGlobalPaletteData>()[0]

    async function onReload() {
        {
            const response = await tauriBridge.invoke<MapGenValue[], string>(
                TauriCommand.GET_GLOBAL_PALETTES,
                {}
            )

            if (response.type === BackendResponseType.Error) {
                return
            }

            setGlobalPalettes(response.data)
        }

        const response = await tauriBridge.invoke<GlobalPaletteReprResponseData, string>(
            TauriCommand.GET_GLOBAL_PALETTE_REPRESENTATIONS,
            {}
        )

        if (response.type === BackendResponseType.Error) {
            return
        }

        console.log(response.data)
        setGlobalPaletteReprs(response.data)
    }

    useTauriEvent(
        TauriEvent.UPDATE_CDDA_DATA,
        () => {
            (async () => {
                await onReload()
            })()
        },
        []
    )

    useEffect(() => {
        (async () => {
            await onReload()
        })()
    }, [project]);

    return (
        <GenericWindow title={"Global Select"}>
            <div className={"character-mapping"}>
                {
                    initialData &&
                    Object.keys(globalPaletteReprs).map(key => {
                        const characterMapping = globalPaletteReprs[key]

                        return <div className={"character-mapping-container"} key={key}>
                            {
                                characterMapping.terrain?.bg &&
                                <TilesheetSprite
                                    className={"character-mapping-sprite"}
                                    tilesheets={initialData.slimTilesheets}
                                    spritesheetConfig={initialData.spritesheetConfig}
                                    index={characterMapping.terrain.bg}
                                    scale={2}
                                />
                            }
                            {
                                characterMapping.terrain?.fg &&
                                <TilesheetSprite
                                    className={"character-mapping-sprite"}
                                    tilesheets={initialData.slimTilesheets}
                                    spritesheetConfig={initialData.spritesheetConfig}
                                    index={characterMapping.terrain.fg}
                                    scale={2}
                                />
                            }
                            {
                                characterMapping.furniture?.bg &&
                                <TilesheetSprite
                                    className={"character-mapping-sprite"}
                                    tilesheets={initialData.slimTilesheets}
                                    spritesheetConfig={initialData.spritesheetConfig}
                                    index={characterMapping.furniture.bg}
                                    scale={2}
                                />
                            }
                            {
                                characterMapping.furniture?.fg &&
                                <TilesheetSprite
                                    className={"character-mapping-sprite"}
                                    tilesheets={initialData.slimTilesheets}
                                    spritesheetConfig={initialData.spritesheetConfig}
                                    index={characterMapping.furniture.fg}
                                    scale={2}
                                />
                            }
                            {
                                characterMapping.monster?.bg &&
                                <TilesheetSprite
                                    className={"character-mapping-sprite"}
                                    tilesheets={initialData.slimTilesheets}
                                    spritesheetConfig={initialData.spritesheetConfig}
                                    index={characterMapping.monster.bg}
                                    scale={2}
                                />
                            }
                            {
                                characterMapping.monster?.fg &&
                                <TilesheetSprite
                                    className={"character-mapping-sprite"}
                                    tilesheets={initialData.slimTilesheets}
                                    spritesheetConfig={initialData.spritesheetConfig}
                                    index={characterMapping.monster.fg}
                                    scale={2}
                                />
                            }
                        </div>
                    })
                }
            </div>
        </GenericWindow>
    );
}

export default Main;
