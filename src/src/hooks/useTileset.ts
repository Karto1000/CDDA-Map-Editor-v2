import {MutableRefObject, useEffect, useRef} from "react";
import {LegacyTilesetCommand} from "../lib/tileset/legacy/send";
import {BackendResponse, BackendResponseType, invokeTauri} from "../lib";
import {Scene, Vector2} from "three";
import {EditorData} from "../lib/editor_data/recv";
import {Tilesheet} from "../rendering/tilesheet.ts";
import {Tilesheets} from "../rendering/tilesheets.ts";

export type Atlases = { [file: string]: Tilesheet }

export function useTileset(editorData: EditorData, sceneRef: MutableRefObject<Scene>): MutableRefObject<Tilesheets> {
    const tilesheets = useRef<Tilesheets>()

    useEffect(() => {
        if (!editorData?.config.selected_tileset) return

        (async () => {
            const infoResponse = await invokeTauri<SpritesheetConfig, unknown>(LegacyTilesetCommand.GetInfoOfCurrentTileset, {})

            if (infoResponse.type === BackendResponseType.Error) {
                return
            }

            const downloadPromises: Promise<BackendResponse<ArrayBuffer, unknown>>[] = []

            for (let tileInfo of infoResponse.data["tiles-new"]) {
                console.log(`Loading ${tileInfo.file}`)

                const response = invokeTauri<ArrayBuffer, unknown>(
                    LegacyTilesetCommand.DownloadSpritesheet, {name: tileInfo.file}
                );

                downloadPromises.push(response)
            }

            const arrayBuffs = await Promise.all(downloadPromises)
            const atlases = {}

            for (let i = 0; i < arrayBuffs.length; i++) {
                const response = arrayBuffs[i]

                if (response.type === BackendResponseType.Error) {
                    console.log(`Failed to load Tileset ${response.error}`)
                    return
                }

                const spritesheetInfo = infoResponse.data["tiles-new"][i]

                const blob = new Blob([response.data], {type: "image/png"});
                const url = URL.createObjectURL(blob)


                atlases[spritesheetInfo.file] = await Tilesheet.fromURL(
                    url,
                    infoResponse.data.tile_info[0],
                    spritesheetInfo
                )
            }

            for (let atlasKey of Object.keys(atlases)) {
                console.log(`Adding ${atlasKey} to the scene`)

                const atlas = atlases[atlasKey]
                sceneRef.current.add(atlas.mesh)
            }

            tilesheets.current = new Tilesheets(atlases);
        })()
    }, [editorData?.config.selected_tileset, sceneRef]);

    return tilesheets
}