import React, {RefObject, useEffect, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {CDDADataField, FurnitureData, TerrainData, TileLayer} from "../../tauri/types/cdda_data.js";
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
import {MultiMenu} from "../../shared/components/imguilike/multimenu.js";

export type InitialTileSearchData = {
    slimTilesheets: SlimTilesheets,
    spritesheetConfig: RefObject<SpritesheetConfig>
}

enum TerrainIdRepresentationKind {
    Defined = "Defined",
    Fallback = "Fallback",
}

type IdRepresentation = {
    type: TerrainIdRepresentationKind.Defined,
    ids: ForeBackIds
} | {
    type: TerrainIdRepresentationKind.Fallback,
    ids: number
}

function Main() {
    const [terrain, setTerrain] = useState<TerrainData>({})
    const [furniture, setFurniture] = useState<FurnitureData>({})

    const [reprIndices, setReprIndices] = useState<{ [id: string]: IdRepresentation }>()
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()
    const initialData = useInitialData<InitialTileSearchData>()[0]
    const [query, setQuery] = useState<string>("")

    useEffect(() => {
        async function getTerrainData(): Promise<TerrainData> {
            const response = await tauriBridge.invoke<TerrainData, string>(
                TauriCommand.GET_CDDA_DATA_FIELD,
                {fieldName: CDDADataField.TERRAIN}
            )

            if (response.type === BackendResponseType.Error) {
                await logToastError(response.error)
                return
            }

            setTerrain(response.data)
            return response.data
        }

        async function getFurnitureData(): Promise<FurnitureData> {
            const response = await tauriBridge.invoke<FurnitureData, string>(
                TauriCommand.GET_CDDA_DATA_FIELD,
                {fieldName: CDDADataField.FURNITURE}
            )

            if (response.type === BackendResponseType.Error) {
                await logToastError(response.error)
                return
            }

            setFurniture(response.data)
            return response.data
        }

        (async () => {
            const terrainData = await getTerrainData()
            const furnitureData = await getFurnitureData()

            let newReprIndices: { [id: string]: IdRepresentation } = {};

            const terrainReprResponse = await tauriBridge.invoke<{ [id: string]: IdRepresentation }, string>(
                TauriCommand.GET_REPRESENTATIONS_FOR_IDS,
                {ids: Object.keys(terrainData), tileLayer: TileLayer.Terrain}
            )

            if (terrainReprResponse.type === BackendResponseType.Error) {
                await logToastError(terrainReprResponse.error)
                return
            }

            newReprIndices = {...newReprIndices, ...terrainReprResponse.data}

            const furnitureReprResponse = await tauriBridge.invoke<{ [id: string]: IdRepresentation }, string>(
                TauriCommand.GET_REPRESENTATIONS_FOR_IDS,
                {ids: Object.keys(furnitureData), tileLayer: TileLayer.Furniture}
            )

            if (furnitureReprResponse.type === BackendResponseType.Error) {
                await logToastError(furnitureReprResponse.error)
                return
            }

            newReprIndices = {...newReprIndices, ...furnitureReprResponse.data}

            setReprIndices(newReprIndices)
        })()
    }, [])

    function getTileRepresentation(data: TerrainData | FurnitureData): React.JSX.Element[] {
        return Object.keys(data)
            .filter(id => id.toLowerCase().includes(query.toLowerCase()))
            .map(
                id => {
                    if (reprIndices[id].type === TerrainIdRepresentationKind.Fallback) {
                        return <div
                            className={clsx("character-mapping-container")}
                            key={id}
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-html={`Id: ${id}<br/>Foreground Fallback: ${reprIndices[id].ids}<br/>`}
                            onMouseMove={handleMouseMove}>
                            <TilesheetSprite
                                className={"character-mapping-sprite"}
                                tilesheets={initialData.slimTilesheets}
                                spritesheetConfig={initialData.spritesheetConfig}
                                index={reprIndices[id].ids}
                                scale={2}
                                isFallback={true}
                            />
                        </div>
                    }

                    return <div
                        className={clsx("character-mapping-container")}
                        key={id}
                        data-tooltip-id={"info-tooltip"}
                        data-tooltip-html={`Id: ${id}<br/>Foreground: ${reprIndices[id].ids.fg}<br/>Background: ${reprIndices[id].ids.bg}`}
                        onMouseMove={handleMouseMove}>
                        <TilesheetSprite
                            className={"character-mapping-sprite"}
                            tilesheets={initialData.slimTilesheets}
                            spritesheetConfig={initialData.spritesheetConfig}
                            index={reprIndices[id].ids.bg}
                            scale={2}
                        />
                        <TilesheetSprite
                            className={"character-mapping-sprite"}
                            tilesheets={initialData.slimTilesheets}
                            spritesheetConfig={initialData.spritesheetConfig}
                            index={reprIndices[id].ids.fg}
                            scale={2}
                        />
                    </div>
                }
            )
    }

    return (
        <GenericWindow title={"Tile Search"} hasSearch={true} onSearchQueryChanged={v => setQuery(v)}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}
                     style={{zIndex: 2}}/>

            <MultiMenu
                tabs={
                    [
                        {
                            name: "Terrain",
                            content: <div className={"character-mapping"}>
                                {
                                    initialData && reprIndices &&
                                    getTileRepresentation(terrain)
                                }
                            </div>
                        },
                        {
                            name: "Furniture",
                            content: <div className={"character-mapping"}>
                                {
                                    initialData && reprIndices &&
                                    getTileRepresentation(furniture)
                                }
                            </div>
                        }
                    ]
                }
            />
        </GenericWindow>
    )
}

export default Main;
