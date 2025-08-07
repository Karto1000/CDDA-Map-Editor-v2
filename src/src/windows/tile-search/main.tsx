import React, {RefObject, useEffect, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {CDDADataField, Terrains} from "../../tauri/types/cdda_data.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {BackendResponseType, ForeBackIds, TauriCommand} from "../../tauri/events/types.js";
import {logToastError} from "../../shared/utils/log.js";
import {clsx} from "clsx";
import {TilesheetSprite} from "../../shared/components/tilesheetSprite.js";
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {useInitialData} from "../useInitialData.js";
import {SlimTilesheets} from "../../features/sprites/slimTilesheets.js";
import {SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {Tooltip} from "react-tooltip";

export type InitialTileSearchData = {
    slimTilesheets: SlimTilesheets,
    spritesheetConfig: RefObject<SpritesheetConfig>
}

function Main() {
    const [terrain, setTerrain] = useState<Terrains>({})
    const [terrainIndices, setTerrainIndices] = useState<{ [id: string]: ForeBackIds }>()
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()
    const initialData = useInitialData<InitialTileSearchData>()[0]
    const [query, setQuery] = useState<string>("")

    useEffect(() => {
        (async () => {
            const response = await tauriBridge.invoke<Terrains, string>(
                TauriCommand.GET_CDDA_DATA_FIELD,
                {fieldName: CDDADataField.TERRAIN}
            )

            if (response.type === BackendResponseType.Error) {
                await logToastError(response.error)
                return
            }

            setTerrain(response.data)

            const reprResponse = await tauriBridge.invoke<{ [id: string]: ForeBackIds }, string>(
                TauriCommand.GET_REPRESENTATIONS_FOR_TERRAIN_IDS,
                {ids: Object.keys(response.data)}
            )

            if (reprResponse.type === BackendResponseType.Error) {
                await logToastError(reprResponse.error)
                return
            }

            setTerrainIndices(reprResponse.data)
        })()
    }, [])

    return (
        <GenericWindow title={"Tile Search"} hasSearch={true} onSearchQueryChanged={v => setQuery(v)}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}
                     style={{zIndex: 2}}/>

            <div className={"character-mapping"}>
                {
                    initialData &&
                    Object.keys(terrain)
                        .filter(id => id.toLowerCase().includes(query.toLowerCase()))
                        .map(
                            id => (
                                <div
                                    className={clsx("character-mapping-container")}
                                    key={id}
                                    data-tooltip-id={"info-tooltip"}
                                    data-tooltip-html={`Terrain: ${id}`}
                                    onMouseMove={handleMouseMove}>
                                    <TilesheetSprite
                                        className={"character-mapping-sprite"}
                                        tilesheets={initialData.slimTilesheets}
                                        spritesheetConfig={initialData.spritesheetConfig}
                                        index={terrainIndices[id].bg}
                                        scale={2}
                                    />
                                    <TilesheetSprite
                                        className={"character-mapping-sprite"}
                                        tilesheets={initialData.slimTilesheets}
                                        spritesheetConfig={initialData.spritesheetConfig}
                                        index={terrainIndices[id].fg}
                                        scale={2}
                                    />
                                </div>
                            )
                        )
                }
            </div>
        </GenericWindow>
    )
}

export default Main;
