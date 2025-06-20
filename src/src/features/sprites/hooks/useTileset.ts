import {RefObject, useRef} from "react";
import {Tilesheets} from "../tilesheets.js";
import {SpritesheetConfig, TileInfo} from "../../../tauri/types/spritesheet.js";
import {useTauriEvent} from "../../../shared/hooks/useTauriEvent.js";
import {tauriBridge} from "../../../tauri/events/tauriBridge.js";
import {BackendResponse, BackendResponseType, TauriCommand, TauriEvent} from "../../../tauri/events/types.js";
import {Tilesheet} from "../tilesheet.js";
import {ThreeConfig} from "../../three/types/three.js";
import {logDeletion, logError, logRender} from "../../../shared/utils/log.js";
import {emit} from "@tauri-apps/api/event";

export type UseTilesetRet = {
    tilesheets: RefObject<Tilesheets | null>,
    spritesheetConfig: RefObject<SpritesheetConfig | null>,
}

export function useTileset(threeConfig: RefObject<ThreeConfig>): UseTilesetRet {
    const tilesheets = useRef<Tilesheets>(null)
    const spritesheetConfig = useRef<SpritesheetConfig>(null)
    const storedObjectURLS = useRef<string[]>([])

    function addTilesheets() {
        for (const tilesheetKey of Object.keys(tilesheets.current.tilesheets)) {
            logRender(`Adding ${tilesheetKey} to scene`)

            const tilesheet = tilesheets.current.tilesheets[tilesheetKey]
            threeConfig.current.scene.add(tilesheet.mesh)
        }

        logRender("Adding fallback to scene")
        threeConfig.current.scene.add(tilesheets.current.fallback.mesh)
    }

    function removeTilesheets() {
        for (const objectURL of storedObjectURLS.current) {
            logDeletion(`[RENDERING] Revoking URL for tilesheet ${objectURL}`)
            URL.revokeObjectURL(objectURL)
        }

        tilesheets.current.fallback.dispose()
        tilesheets.current.dispose(threeConfig)
        tilesheets.current = null
        storedObjectURLS.current = []
    }

    useTauriEvent(
        TauriEvent.TILESET_CHANGED,
        () => {
            (async () => {
                logRender("Loading Tileset")

                if (tilesheets.current) removeTilesheets()

                const infoResponse = await tauriBridge.invoke<SpritesheetConfig, unknown>(
                    TauriCommand.GET_INFO_OF_CURRENT_TILESET,
                    {}
                )

                if (infoResponse.type === BackendResponseType.Error) {
                    console.error(infoResponse.error)
                    return
                }

                spritesheetConfig.current = infoResponse.data

                const loadFromBackend = async (): Promise<{
                    atlases: { [key: string]: Tilesheet },
                    fallback: Tilesheet,
                    tileInfo: TileInfo
                }> => {
                    const downloadPromises: Promise<BackendResponse<ArrayBuffer, unknown>>[] = []

                    for (let tileInfo of infoResponse.data["tiles-new"]) {
                        logRender(`Loading ${tileInfo.file}`)

                        const promise = tauriBridge.invoke<ArrayBuffer, unknown>(
                            TauriCommand.DOWNLOAD_SPRITESHEET,
                            {name: tileInfo.file},
                        )

                        downloadPromises.push(promise)
                    }

                    const arrayBuffs = await Promise.all(downloadPromises)
                    const atlases = {}
                    let fallback: Tilesheet;

                    storedObjectURLS.current.forEach(url => URL.revokeObjectURL(url))

                    for (let i = 0; i < infoResponse.data["tiles-new"].length; i++) {
                        const response = arrayBuffs[i]

                        if (response.type === BackendResponseType.Error) {
                            logError(`%c[RENDERING] Failed to load Tileset ${response.error}`)
                            return
                        }

                        const spritesheetInfo = infoResponse.data["tiles-new"][i]

                        const blob = new Blob([response.data], {type: "image/png"});
                        const url = URL.createObjectURL(blob)
                        storedObjectURLS.current.push(url)
                        logRender(`[RENDERING] Created URL for ${spritesheetInfo.file}: ${url}`)

                        if (spritesheetInfo.file === "fallback.png") {
                            fallback = await Tilesheet.fromURL(url, infoResponse.data.tile_info[0], spritesheetInfo)
                            continue
                        }

                        atlases[spritesheetInfo.file] = await Tilesheet.fromURL(
                            url,
                            infoResponse.data.tile_info[0],
                            spritesheetInfo
                        )
                    }

                    return {atlases, fallback, tileInfo: infoResponse.data.tile_info[0]}
                }

                logRender("Loading Tilesheet Sprites")
                const atlases = await loadFromBackend();
                tilesheets.current = new Tilesheets(atlases.atlases, atlases.fallback, atlases.tileInfo)

                addTilesheets()

                await emit(TauriEvent.TILESET_LOADED)
            })();

            return () => {
            }
        },
        []
    )

    return {tilesheets, spritesheetConfig}
}