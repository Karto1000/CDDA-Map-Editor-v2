import React, {RefObject, useEffect, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {useCurrentProject} from "../../shared/hooks/useCurrentProject.js";
import {MapEditorData} from "../../tauri/types/editor.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {MapGenValue} from "../../tauri/types/map_data.js";
import {BackendResponseType, CharacterMappingContainer, TauriCommand, TauriEvent} from "../../tauri/events/types.js";
import {useForeignOpenedTab} from "../useForeignOpenedTab.js";
import {SlimTilesheets} from "../../features/sprites/slimTilesheets.js";
import {SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {useInitialData} from "../useInitialData.js";
import {useTauriEvent} from "../../shared/hooks/useTauriEvent.js";
import {Tooltip} from "react-tooltip";
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {TilesheetSprite} from "../../shared/components/tilesheetSprite.js";
import {emit} from "@tauri-apps/api/event";
import {clsx} from "clsx";
import {MultiMenu} from "../../shared/components/imguilike/multimenu.js";

type GlobalPaletteReprResponseData = { [char: string]: CharacterMappingContainer }

export type InitialGlobalPaletteData = {
    slimTilesheets: SlimTilesheets,
    spritesheetConfig: RefObject<SpritesheetConfig>
    selectedCharacter: string,
}

function Main() {
    const [globalPalettes, setGlobalPalettes] = useState<MapGenValue[]>([])
    const openedTab = useForeignOpenedTab()
    const project = useCurrentProject<MapEditorData>(openedTab)
    const [globalPaletteReprs, setGlobalPaletteReprs] = useState<GlobalPaletteReprResponseData>({})
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()
    const [selectedCharacter, setSelectedCharacter] = useState<string>("")
    const initialData = useInitialData<InitialGlobalPaletteData>(data => setSelectedCharacter(data.selectedCharacter))[0]
    const [query, setQuery] = useState<string>("")

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

        setGlobalPaletteReprs(response.data)
    }

    async function onCharacterSelected(character: string) {
        await emit(
            TauriEvent.CHARACTER_SELECTED,
            {
                character,
            }
        )
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

    useTauriEvent(
        TauriEvent.CHARACTER_SELECTED,
        (selectedChar) => {
            setSelectedCharacter(selectedChar.character)
        },
        [selectedCharacter]
    )

    useEffect(() => {
        (async () => {
            await onReload()
        })()
    }, [project]);

    return (
        <GenericWindow title={"Global Select"} hasSearch={true} onSearchQueryChanged={v => setQuery(v)}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}
                     style={{zIndex: 2}}/>

            <MultiMenu tabs={
                [
                    {
                        name: "Terrain",
                        content: <div className={"character-mapping"}>
                            {
                                initialData &&
                                Object.keys(globalPaletteReprs)
                                    .filter(key => {
                                        const characterMapping = globalPaletteReprs[key]

                                        let doesMatch = key.toLowerCase().includes(query.toLowerCase())

                                        if (characterMapping.terrain) {
                                            doesMatch ||= characterMapping.terrain.id.toLowerCase().includes(query.toLowerCase())
                                        }

                                        if (characterMapping.furniture) {
                                            doesMatch ||= characterMapping.furniture.id.toLowerCase().includes(query.toLowerCase())
                                        }

                                        if (characterMapping.field) {
                                            doesMatch ||= characterMapping.field.id.toLowerCase().includes(query.toLowerCase())
                                        }

                                        if (characterMapping.monster) {
                                            doesMatch ||= characterMapping.monster.id.toLowerCase().includes(query.toLowerCase())
                                        }

                                        return doesMatch
                                    })
                                    .map(key => {
                                        const characterMapping = globalPaletteReprs[key]

                                        return <div
                                            className={clsx("character-mapping-container", selectedCharacter === key && "selected")}
                                            onClick={() => onCharacterSelected(key)}
                                            key={key}
                                            data-tooltip-id={"info-tooltip"}
                                            data-tooltip-html={`
                                        Character: "${key}"<br/>
                                        ${
                                                characterMapping.terrain ?
                                                    `Shown Terrain: ${characterMapping.terrain?.id}<br/>`
                                                    :
                                                    ""
                                            }
                                        ${
                                                characterMapping.furniture ?
                                                    `Shown Furniture: ${characterMapping.furniture.id}<br/>`
                                                    :
                                                    ""
                                            }
                                        ${
                                                characterMapping.field ?
                                                    `Shown Field: ${characterMapping.field.id}<br/>`
                                                    :
                                                    ""
                                            }
                                        ${
                                                characterMapping.monster ?
                                                    `Shown Monster: ${characterMapping.monster.id}`
                                                    :
                                                    ""
                                            }`}
                                            onMouseMove={handleMouseMove}
                                        >
                                            {
                                                characterMapping.terrain?.ids.bg &&
                                                <TilesheetSprite
                                                    className={"character-mapping-sprite"}
                                                    tilesheets={initialData.slimTilesheets}
                                                    spritesheetConfig={initialData.spritesheetConfig}
                                                    index={characterMapping.terrain.ids.bg}
                                                    scale={2}
                                                />
                                            }
                                            {
                                                characterMapping.terrain?.ids.fg &&
                                                <TilesheetSprite
                                                    className={"character-mapping-sprite"}
                                                    tilesheets={initialData.slimTilesheets}
                                                    spritesheetConfig={initialData.spritesheetConfig}
                                                    index={characterMapping.terrain.ids.fg}
                                                    scale={2}
                                                />
                                            }
                                            {
                                                characterMapping.furniture?.ids.bg &&
                                                <TilesheetSprite
                                                    className={"character-mapping-sprite"}
                                                    tilesheets={initialData.slimTilesheets}
                                                    spritesheetConfig={initialData.spritesheetConfig}
                                                    index={characterMapping.furniture.ids.bg}
                                                    scale={2}
                                                />
                                            }
                                            {
                                                characterMapping.furniture?.ids.fg &&
                                                <TilesheetSprite
                                                    className={"character-mapping-sprite"}
                                                    tilesheets={initialData.slimTilesheets}
                                                    spritesheetConfig={initialData.spritesheetConfig}
                                                    index={characterMapping.furniture.ids.fg}
                                                    scale={2}
                                                />
                                            }
                                            {
                                                characterMapping.monster?.ids.bg &&
                                                <TilesheetSprite
                                                    className={"character-mapping-sprite"}
                                                    tilesheets={initialData.slimTilesheets}
                                                    spritesheetConfig={initialData.spritesheetConfig}
                                                    index={characterMapping.monster.ids.bg}
                                                    scale={2}
                                                />
                                            }
                                            {
                                                characterMapping.monster?.ids.fg &&
                                                <TilesheetSprite
                                                    className={"character-mapping-sprite"}
                                                    tilesheets={initialData.slimTilesheets}
                                                    spritesheetConfig={initialData.spritesheetConfig}
                                                    index={characterMapping.monster.ids.fg}
                                                    scale={2}
                                                />
                                            }
                                            {
                                                <span className={"char-text"}>{key}</span>
                                            }
                                        </div>
                                    })
                            }
                        </div>
                    }
                ]
            }/>
        </GenericWindow>
    );
}

export default Main;
