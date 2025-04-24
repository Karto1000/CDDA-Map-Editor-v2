import {MutableRefObject, useEffect, useRef, useState} from "react";
import {BackendResponse, BackendResponseType, invokeTauri} from "../lib/index.ts";
import {LinearMipMapNearestFilter, NearestFilter, Scene, SRGBColorSpace, TextureLoader, Vector2} from "three";
import {Tilesheet} from "../rendering/tilesheet.ts";
import {Tilesheets} from "../rendering/tilesheets.ts";
import {EditorData} from "../lib/editor_data.ts";
import {LegacyTilesetCommand, SpritesheetConfig, TileInfo} from "../lib/tileset/legacy.ts";

export type Atlases = { [file: string]: Tilesheet }

export function useTileset(editorData: EditorData, sceneRef: MutableRefObject<Scene>): [MutableRefObject<Tilesheets>, MutableRefObject<SpritesheetConfig>, boolean] {
    const tilesheets = useRef<Tilesheets>()
    const spritesheetConfig = useRef<SpritesheetConfig>()
    const [isLoaded, setIsLoaded] = useState<boolean>(false)

    useEffect(() => {
        if (!editorData?.config.selected_tileset) return

        setIsLoaded(false);

        (async () => {
            const infoResponse = await invokeTauri<SpritesheetConfig, unknown>(LegacyTilesetCommand.GetInfoOfCurrentTileset, {})

            if (infoResponse.type === BackendResponseType.Error) {
                console.error(infoResponse.error)
                return
            }

            spritesheetConfig.current = infoResponse.data;

            const loadFromBackend = async (): Promise<{
                atlases: { [key: string]: Tilesheet },
                fallback: Tilesheet
            }> => {
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
                let fallback;

                for (let i = 0; i < infoResponse.data["tiles-new"].length; i++) {
                    const response = arrayBuffs[i]

                    if (response.type === BackendResponseType.Error) {
                        console.log(`Failed to load Tileset ${response.error}`)
                        return
                    }

                    const spritesheetInfo = infoResponse.data["tiles-new"][i]

                    const blob = new Blob([response.data], {type: "image/png"});
                    const url = URL.createObjectURL(blob)

                    if (spritesheetInfo.file === "fallback.png") {
                        fallback = Tilesheet.fromURL(url, infoResponse.data.tile_info[0], spritesheetInfo)
                        continue
                    }

                    atlases[spritesheetInfo.file] = await Tilesheet.fromURL(
                        url,
                        infoResponse.data.tile_info[0],
                        spritesheetInfo
                    )
                }

                return {atlases, fallback}
            }

            const loadFromPublic = async (): Promise<{
                atlases: { [key: string]: Tilesheet },
                fallback: Tilesheet,
                tileInfo: TileInfo
            }> => {
                const atlases = {}
                let fallback: Tilesheet;
                const tileInfo = infoResponse.data.tile_info[0]

                for (let i = 0; i < infoResponse.data["tiles-new"].length; i++) {
                    const spritesheetInfo = infoResponse.data["tiles-new"][i]

                    const texture = await new TextureLoader()
                        .loadAsync(`/MSX++UnDeadPeopleEdition/${spritesheetInfo.file}`,
                            () => console.log(`Loading ${spritesheetInfo.file}`))

                    texture.magFilter = NearestFilter;
                    texture.minFilter = LinearMipMapNearestFilter;
                    // https://stackoverflow.com/a/77944452
                    texture.colorSpace = SRGBColorSpace

                    if (spritesheetInfo.file === "fallback.png") {
                        fallback = new Tilesheet(texture, tileInfo, spritesheetInfo)
                        continue
                    }

                    atlases[spritesheetInfo.file] = new Tilesheet(
                        texture,
                        tileInfo,
                        spritesheetInfo
                    )
                }

                return {atlases, fallback, tileInfo}
            }

            const atlases = await loadFromPublic();

            sceneRef.current.add(atlases.fallback.mesh)
            for (let atlasKey of Object.keys(atlases.atlases)) {
                console.log(`Adding ${atlasKey} to the scene`)

                const atlas = atlases.atlases[atlasKey]
                sceneRef.current.add(atlas.mesh)
            }

            setIsLoaded(true)
            tilesheets.current = new Tilesheets(atlases.atlases, atlases.fallback, atlases.tileInfo);
        })()
    }, [editorData?.config.selected_tileset, sceneRef]);

    return [tilesheets, spritesheetConfig, isLoaded]
}