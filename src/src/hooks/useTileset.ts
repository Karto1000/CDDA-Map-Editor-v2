import {MutableRefObject, useEffect, useRef, useState} from "react";
import {LegacyTilesetCommand} from "../lib/tileset/legacy/send/index.ts";
import {BackendResponse, BackendResponseType, invokeTauri} from "../lib/index.ts";
import {LinearMipMapNearestFilter, NearestFilter, Scene, SRGBColorSpace, TextureLoader, Vector2} from "three";
import {EditorData} from "../lib/editor_data/recv/index.ts";
import {Tilesheet} from "../rendering/tilesheet.ts";
import {Tilesheets} from "../rendering/tilesheets.ts";

export type Atlases = { [file: string]: Tilesheet }

export function useTileset(editorData: EditorData, sceneRef: MutableRefObject<Scene>): [MutableRefObject<Tilesheets>, boolean] {
    const tilesheets = useRef<Tilesheets>()
    const [isLoaded, setIsLoaded] = useState<boolean>(false)

    useEffect(() => {
        if (!editorData?.config.selected_tileset) return

        setIsLoaded(false);

        (async () => {
            const infoResponse = await invokeTauri<SpritesheetConfig, unknown>(LegacyTilesetCommand.GetInfoOfCurrentTileset, {})

            if (infoResponse.type === BackendResponseType.Error) {
                return
            }

            const loadFromBackend = async (): Promise<{ [key: string]: Tilesheet }> => {
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

                for (let i = 0; i < infoResponse.data["tiles-new"].length; i++) {
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

                return atlases
            }

            const loadFromPublic = async (): Promise<{ [key: string]: Tilesheet }> => {
                const atlases = {}

                for (let i = 0; i < infoResponse.data["tiles-new"].length; i++) {
                    const spritesheetInfo = infoResponse.data["tiles-new"][i]

                    const texture = await new TextureLoader()
                        .loadAsync(`/MSX++UnDeadPeopleEdition/${spritesheetInfo.file}`,
                            () => console.log(`Loading ${spritesheetInfo.file}`))

                    texture.magFilter = NearestFilter;
                    texture.minFilter = LinearMipMapNearestFilter;
                    // https://stackoverflow.com/a/77944452
                    texture.colorSpace = SRGBColorSpace

                    atlases[spritesheetInfo.file] = new Tilesheet(
                        texture,
                        infoResponse.data.tile_info[0],
                        spritesheetInfo
                    )
                }

                return atlases
            }

            const atlases = await loadFromPublic();

            for (let atlasKey of Object.keys(atlases)) {
                console.log(`Adding ${atlasKey} to the scene`)

                const atlas = atlases[atlasKey]
                sceneRef.current.add(atlas.mesh)
            }

            setIsLoaded(true)
            tilesheets.current = new Tilesheets(atlases);
        })()
    }, [editorData?.config.selected_tileset, sceneRef]);

    return [tilesheets, isLoaded]
}