import {TextureAtlas} from "../rendering/texture-atlas.ts";
import {MutableRefObject, useEffect, useRef} from "react";
import {invoke} from "@tauri-apps/api/core";
import {TilesetConfig} from "../lib/map_data/recv";
import {LegacyTilesetCommand} from "../lib/tileset/legacy/send";
import {BackendResponse, BackendResponseType, invokeTauri} from "../lib";
import {Scene, Vector2} from "three";
import {EditorData} from "../lib/editor_data/recv";

export type Atlases = { [file: string]: TextureAtlas }

export function useTileset(editorData: EditorData, sceneRef: MutableRefObject<Scene>): MutableRefObject<Atlases> {
    const atlases = useRef<Atlases>({})

    useEffect(() => {
        if (!editorData?.config.selected_tileset) return

        (async () => {
            const metadata = await invoke<TilesetConfig>(
                LegacyTilesetCommand.GetTilesetMetadata,
                {name: editorData.config.selected_tileset}
            )

            const downloadPromises: Promise<BackendResponse<ArrayBuffer, unknown>>[] = []

            for (let tileInfo of metadata["tiles-new"]) {
                console.log(`Loading ${tileInfo.file}`)

                const response = invokeTauri<ArrayBuffer, unknown>(
                    LegacyTilesetCommand.DownloadSpritesheet, {
                        tileset: editorData.config.selected_tileset,
                        name: tileInfo.file
                    }
                );

                downloadPromises.push(response)
            }

            const arrayBuffs = await Promise.all(downloadPromises)

            for (let i = 0; i < arrayBuffs.length; i++) {
                const response = arrayBuffs[i]

                if (response.type === BackendResponseType.Error) {
                    console.log(`Failed to load Tileset`)
                    return
                }

                const tileInfo = metadata["tiles-new"][i]

                const blob = new Blob([response.data], {type: "image/png"});
                const url = URL.createObjectURL(blob)

                atlases.current[tileInfo.file] = TextureAtlas.loadFromURL(
                    url,
                    {
                        "t_grass": {
                            name: "t_grass",
                            position: new Vector2(128, 2624)
                        },
                    },
                    {
                        atlasWidth: tileInfo.spritesheet_dimensions[0],
                        atlasHeight: tileInfo.spritesheet_dimensions[1],
                        tileWidth: tileInfo.sprite_width,
                        tileHeight: tileInfo.sprite_height,
                        maxInstances: 73728,
                        yLayer: 0
                    }
                )
            }

            for (let atlasKey of Object.keys(atlases.current)) {
                const atlas = atlases.current[atlasKey]
                sceneRef.current.add(atlas.mesh)
            }
        })()
    }, [editorData?.config.selected_tileset, sceneRef]);

    return atlases
}