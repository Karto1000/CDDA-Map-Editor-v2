import {MutableRefObject, RefObject, useRef} from "react";
import {Tilesheets} from "../tilesheets.js";
import {SpritesheetConfig, TileInfo} from "../../../tauri/types/spritesheet.js";
import {useTauriEvent} from "../../../shared/hooks/useTauriEvent.js";
import {tauriBridge} from "../../../tauri/events/tauriBridge.js";
import {BackendResponse, BackendResponseType, TauriCommand, TauriEvent} from "../../../tauri/events/types.js";
import {Tilesheet} from "../tilesheet.js";
import {LocalEvent, TilesetLoadedEvent} from "../../../shared/utils/localEvent.js";

export type UseTilesetRet = {
    tilesheets: RefObject<Tilesheets>,
    spritesheetConfig: RefObject<SpritesheetConfig>,
}

export function useTileset(eventBus: RefObject<EventTarget>): UseTilesetRet {
    const tilesheets = useRef<Tilesheets>(null)
    const spritesheetConfig = useRef<SpritesheetConfig>(null)
    const storedObjectURLS = useRef<string[]>([])

    useTauriEvent(
        TauriEvent.TILESET_CHANGED,
        () => {
            (async () => {
                console.log("Loading Tileset")

                const infoResponse = await tauriBridge.invoke<
                    SpritesheetConfig,
                    unknown,
                    TauriCommand.GET_INFO_OF_CURRENT_TILESET
                >(
                    TauriCommand.GET_INFO_OF_CURRENT_TILESET,
                    {}
                )

                if (infoResponse.type === BackendResponseType.Error) {
                    console.error(infoResponse.error)
                    return
                }

                spritesheetConfig.current = infoResponse.data;

                const loadFromBackend = async (): Promise<{
                    atlases: { [key: string]: Tilesheet },
                    fallback: Tilesheet,
                    tileInfo: TileInfo
                }> => {
                    const downloadPromises: Promise<BackendResponse<ArrayBuffer, unknown>>[] = []

                    for (let tileInfo of infoResponse.data["tiles-new"]) {
                        console.log(`Loading ${tileInfo.file}`)

                        const promise = tauriBridge.invoke<
                            ArrayBuffer,
                            unknown,
                            TauriCommand.DOWNLOAD_SPRITESHEET
                        >(
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
                            console.log(`Failed to load Tileset ${response.error}`)
                            return
                        }

                        const spritesheetInfo = infoResponse.data["tiles-new"][i]

                        const blob = new Blob([response.data], {type: "image/png"});
                        const url = URL.createObjectURL(blob)
                        storedObjectURLS.current.push(url)

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

                console.log("Loading Tilesheet Sprites")
                const atlases = await loadFromBackend();

                const localTilesheets = new Tilesheets(atlases.atlases, atlases.fallback, atlases.tileInfo)
                eventBus.current.dispatchEvent(
                    new TilesetLoadedEvent(
                        LocalEvent.TILESET_LOADED,
                        {detail: localTilesheets}
                    )
                )
                tilesheets.current = localTilesheets;
            })()
        },
        [eventBus]
    )

    return {tilesheets, spritesheetConfig}
}