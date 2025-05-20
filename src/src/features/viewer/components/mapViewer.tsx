import {Canvas, ThreeConfig} from "../../three/types/three.js";
import React, {MutableRefObject, useContext, useEffect, useState} from "react";
import {
    ChangedThemeEvent,
    LocalEvent,
    OpenLocalTabEvent,
    TilesetLoadedEvent
} from "../../../shared/utils/localEvent.js";
import {getColorFromTheme, Theme} from "../../../shared/hooks/useTheme.js";
import {GridHelper, Vector3} from "three";
import {degToRad} from "three/src/math/MathUtils.js";
import {SpritesheetConfig, TileInfo} from "../../../tauri/types/spritesheet.js";
import {DrawAnimatedSprite, DrawStaticSprite, MAX_DEPTH, Tilesheets} from "../../sprites/tilesheets.js";
import {TabContext, ThemeContext} from "../../../app.js";
import {useTauriEvent} from "../../../shared/hooks/useTauriEvent.js";
import {serializedVec2ToVector2, TauriCommand, TauriEvent} from "../../../tauri/events/types.js";
import {tauriBridge} from "../../../tauri/events/tauriBridge.js";
import {useWorldMousePosition} from "../../three/hooks/useWorldMousePosition.js";
import {useMouseCells} from "../../three/hooks/useMouseCells.js";
import {SHOW_STATS} from "../../three/hooks/useThreeSetup.js";

export type MapViewerProps = {
    threeConfig: MutableRefObject<ThreeConfig>
    eventBus: MutableRefObject<EventTarget>,
    spritesheetConfig: MutableRefObject<SpritesheetConfig>,
    tileInfo: TileInfo | null
    canvas: Canvas,
    isOpen: boolean
    tilesheets: MutableRefObject<Tilesheets>
}

export function MapViewer(props: MapViewerProps) {
    const theme = useContext(ThemeContext)
    const {hoveredCellMeshRef, selectedCellMeshRef, regenerate} = useMouseCells(props.threeConfig, props.tileInfo)
    const [selectedCellPosition, setSelectedCellPosition] = useState<Vector3 | null>(null)
    const worldMousePosition = useWorldMousePosition({
        threeConfig: props.threeConfig,
        canvas: props.canvas,
        tileWidth: props.tileInfo?.width,
        tileHeight: props.tileInfo?.height,
        onMouseMove: (mousePosition) => {
            hoveredCellMeshRef.current.position.set(
                mousePosition.x * props.tileInfo.width,
                // Remove one again for three.js since the top left tile is -1 in three.js
                (-mousePosition.y - 1) * props.tileInfo.height,
                MAX_DEPTH + 1
            )
        }
    })
    const tabs = useContext(TabContext)

    function setupSceneData(theme: Theme) {
        if (!props.threeConfig.current || !props.tileInfo) return;

        const tile_info = props.spritesheetConfig.current.tile_info[0]

        props.threeConfig.current.renderer.setClearColor(getColorFromTheme(theme, "darker"))

        regenerate(theme)

        const gridHelper = new GridHelper(
            1,
            16 * 8 * tile_info.width * 24 / tile_info.height,
            getColorFromTheme(theme, "disabled"), getColorFromTheme(theme, "light")
        )
        gridHelper.scale.x = 16 * 8 * tile_info.width * 24
        gridHelper.scale.z = 16 * 8 * tile_info.height * 24

        gridHelper.position.x -= tile_info.width / 2
        gridHelper.position.y -= tile_info.height / 2

        gridHelper.rotateX(degToRad(90))

        props.threeConfig.current.scene.remove(props.threeConfig.current.gridHelper)
        props.threeConfig.current.scene.add(gridHelper)
        props.threeConfig.current.gridHelper = gridHelper
    }

    useTauriEvent(
        TauriEvent.PLACE_SPRITES,
        (d) => {
            console.log("Placing sprites")
            props.tilesheets.current.clearAll()

            const drawStaticSprites: DrawStaticSprite[] = d.static_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= props.tileInfo.width;
                vec2.y *= props.tileInfo.height;

                return {
                    ...ds,
                    position: vec2
                }
            })

            const drawAnimatedSprites: DrawAnimatedSprite[] = d.animated_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= props.tileInfo.width;
                vec2.y *= props.tileInfo.height;

                return {
                    ...ds,
                    position: vec2,
                }
            })

            const drawFallbackSprites: DrawStaticSprite[] = d.fallback_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= props.tileInfo.width;
                vec2.y *= props.tileInfo.height;

                return {
                    ...ds,
                    layer: 0,
                    position: vec2,
                    rotate_deg: 0
                }
            })

            props.tilesheets.current.drawFallbackSpritesBatched(drawFallbackSprites)
            props.tilesheets.current.drawStaticSpritesBatched(drawStaticSprites)
            props.tilesheets.current.drawAnimatedSpritesBatched(drawAnimatedSprites)
        },
        [props.tilesheets, props.tileInfo],
    )

    useTauriEvent(
        TauriEvent.UPDATE_LIVE_VIEWER,
        async () => {
            await tauriBridge.invoke<unknown, unknown, TauriCommand.RELOAD_PROJECT>(TauriCommand.RELOAD_PROJECT, {})
            await tauriBridge.invoke<unknown, unknown, TauriCommand.GET_SPRITES>(TauriCommand.GET_SPRITES, {name: tabs.openedTab});
        },
        [tabs.openedTab]
    )

    useEffect(() => {
        const openLocalTabHandler = async (t: OpenLocalTabEvent) => {
            await tauriBridge.invoke<unknown, unknown, TauriCommand.RELOAD_PROJECT>(TauriCommand.RELOAD_PROJECT, {})
            await tauriBridge.invoke<unknown, unknown, TauriCommand.GET_SPRITES>(TauriCommand.GET_SPRITES, {name: t.detail.name});
        }

        const tilesetLoadedHandler = (e: TilesetLoadedEvent) => {
            for (const name of Object.keys(e.detail.tilesheets)) {
                const tilesheet = e.detail.tilesheets[name]
                console.log(`Adding tilesheet ${name} to scene`)
                props.threeConfig.current.scene.add(tilesheet.mesh)
            }

            props.threeConfig.current.scene.add(e.detail.fallback.mesh)
        }

        const changeThemeHandler = (e: ChangedThemeEvent) => {
            console.log(`Changing Map viewer theme to ${e.detail.theme}`)
            setupSceneData(e.detail.theme)
        }

        props.eventBus.current.addEventListener(
            LocalEvent.OPEN_LOCAL_TAB,
            openLocalTabHandler
        )

        props.eventBus.current.addEventListener(
            LocalEvent.TILESET_LOADED,
            tilesetLoadedHandler
        )

        props.eventBus.current.addEventListener(
            LocalEvent.CHANGED_THEME,
            changeThemeHandler
        )

        return () => {
            props.eventBus.current.removeEventListener(
                LocalEvent.OPEN_LOCAL_TAB,
                openLocalTabHandler
            )

            props.eventBus.current.removeEventListener(
                LocalEvent.TILESET_LOADED,
                tilesetLoadedHandler
            )

            props.eventBus.current.removeEventListener(
                LocalEvent.CHANGED_THEME,
                changeThemeHandler
            )
        }
    }, [props.eventBus, setupSceneData]);

    useEffect(() => {
        if (!props.isOpen) return
        if (!props.threeConfig.current || !props.tileInfo) return;

        function initialValueUpdate() {
            const newWidth = props.canvas.canvasContainerRef.current.clientWidth
            const newHeight = props.canvas.canvasContainerRef.current.clientHeight

            props.threeConfig.current.renderer.setSize(newWidth, newHeight)
            props.threeConfig.current.camera.left = newWidth / -2
            props.threeConfig.current.camera.right = newWidth / 2
            props.threeConfig.current.camera.top = newHeight / 2
            props.threeConfig.current.camera.bottom = newHeight / -2
            props.threeConfig.current.camera.position.z = 999999
        }

        initialValueUpdate();

        let handler: number;

        function loop() {
            if (SHOW_STATS) props.threeConfig.current.stats.begin()

            props.threeConfig.current.camera.updateProjectionMatrix()
            if (props.tilesheets.current) props.tilesheets.current.updateAnimatedSprites()

            props.threeConfig.current.controls.update()
            props.threeConfig.current.renderer.render(props.threeConfig.current.scene, props.threeConfig.current.camera)

            if (SHOW_STATS) props.threeConfig.current.stats.end()

            handler = requestAnimationFrame(loop)
        }

        loop()

        return () => {
            cancelAnimationFrame(handler)
        }
    }, [props.isOpen, props.threeConfig, props.tilesheets]);

    useEffect(() => {
        if (!props.threeConfig.current || !props.tileInfo) return;

        console.log("Setting up scene data")
        setupSceneData(theme.theme)
    }, [props.threeConfig, props.tileInfo]);

    useEffect(() => {
        const onMouseDown = (e: MouseEvent) => {
            if (e.button === 0) {
                if (selectedCellPosition?.x === worldMousePosition.current.x && selectedCellPosition?.y === worldMousePosition.current.y) {
                    selectedCellMeshRef.current.visible = false
                    setSelectedCellPosition(null)
                } else {
                    selectedCellMeshRef.current.position.set(
                        worldMousePosition.current.x * props.tileInfo.width,
                        (-worldMousePosition.current.y - 1) * props.tileInfo.height,
                        MAX_DEPTH + 1
                    )
                    selectedCellMeshRef.current.visible = true
                    setSelectedCellPosition(worldMousePosition.current)
                }
            }
        }

        props.canvas.canvasRef.current.addEventListener("mousedown", onMouseDown)

        return () => {
            props.canvas.canvasRef.current.removeEventListener("mousedown", onMouseDown)
        }
    }, [props.eventBus, props.tileInfo, selectedCellPosition, worldMousePosition]);

    return <></>
}