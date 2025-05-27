import {Canvas, ThreeConfig} from "../../three/types/three.js";
import React, {MutableRefObject, useContext, useEffect, useRef, useState} from "react";
import {
    ChangedThemeEvent, ChangeSelectedPositionEvent,
    ChangeWorldMousePositionEvent,
    ChangeZLevelEvent,
    CloseLocalTabEvent,
    LocalEvent,
    TilesetLoadedEvent
} from "../../../shared/utils/localEvent.js";
import {getColorFromTheme, Theme} from "../../../shared/hooks/useTheme.js";
import {GridHelper, Vector3} from "three";
import {degToRad} from "three/src/math/MathUtils.js";
import {SpritesheetConfig, TileInfo} from "../../../tauri/types/spritesheet.js";
import {DrawAnimatedSprite, DrawStaticSprite, MAX_DEPTH, Tilesheets} from "../../sprites/tilesheets.js";
import {SidebarContent, TabContext, ThemeContext} from "../../../app.js";
import {useTauriEvent} from "../../../shared/hooks/useTauriEvent.js";
import {BackendResponseType, serializedVec2ToVector2, TauriCommand, TauriEvent} from "../../../tauri/events/types.js";
import {tauriBridge} from "../../../tauri/events/tauriBridge.js";
import {useWorldMousePosition} from "../../three/hooks/useWorldMousePosition.js";
import {useMouseCells} from "../../three/hooks/useMouseCells.js";
import {SHOW_STATS} from "../../three/hooks/useThreeSetup.js";
import "./mapViewer.scss"
import {clsx} from "clsx";
import toast from "react-hot-toast";
import {CellData} from "../../../tauri/types/map_data.js";

export type MapViewerProps = {
    threeConfig: MutableRefObject<ThreeConfig>
    eventBus: MutableRefObject<EventTarget>,
    spritesheetConfig: MutableRefObject<SpritesheetConfig>,
    tileInfo: TileInfo | null
    canvas: Canvas,
    isOpen: boolean
    tilesheets: MutableRefObject<Tilesheets>
    setSidebarContent: React.SetStateAction<SidebarContent>
}

export function MapViewer(props: MapViewerProps) {
    const theme = useContext(ThemeContext)
    const {hoveredCellMeshRef, selectedCellMeshRef, regenerate} = useMouseCells(props.threeConfig, props.tileInfo)
    const [selectedCellPosition, setSelectedCellPosition] = useState<Vector3 | null>(null)
    const [isLoading, setIsLoading] = useState<boolean>(false)
    const zLevel = useRef<number>(0)
    const worldMousePosition = useWorldMousePosition({
        threeConfig: props.threeConfig,
        canvas: props.canvas,
        tileWidth: props.tileInfo?.width,
        tileHeight: props.tileInfo?.height,
        onWorldMousePositionChange: (newPos) => {
            props.eventBus.current.dispatchEvent(
                new ChangeWorldMousePositionEvent(
                    LocalEvent.CHANGE_WORLD_MOUSE_POSITION,
                    {detail: {position: {x: newPos.x, y: newPos.y}}}
                )
            )
        },
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
    const cellRepresentation = useRef<CellData>()

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

            props.tilesheets.current.drawFallbackSpritesBatched(drawFallbackSprites, zLevel.current)
            props.tilesheets.current.drawStaticSpritesBatched(drawStaticSprites, zLevel.current)
            props.tilesheets.current.drawAnimatedSpritesBatched(drawAnimatedSprites)
        },
        [props.tilesheets, props.tileInfo],
    )

    useTauriEvent(
        TauriEvent.UPDATE_LIVE_VIEWER,
        async () => {
            console.log("Updating live viewer")
            setIsLoading(true)

            props.tilesheets.current.clearAll()

            const reloadResponse = await tauriBridge.invoke<unknown, unknown, TauriCommand.RELOAD_PROJECT>(TauriCommand.RELOAD_PROJECT, {})

            if (reloadResponse.type === BackendResponseType.Error) {
                toast.error(reloadResponse.error)
                setIsLoading(false)
                return
            }

            const getSpritesResponse = await tauriBridge.invoke<unknown, unknown, TauriCommand.GET_SPRITES>(TauriCommand.GET_SPRITES, {name: tabs.openedTab});

            if (getSpritesResponse.type === BackendResponseType.Error) {
                toast.error(getSpritesResponse.error)
                setIsLoading(false)
                return
            }

            const getRepresentationResponse = await tauriBridge.invoke<CellData, unknown, TauriCommand.GET_PROJECT_CELL_DATA>(TauriCommand.GET_PROJECT_CELL_DATA, {})

            if (getRepresentationResponse.type === BackendResponseType.Error) {
                toast.error(getRepresentationResponse.error)
                setIsLoading(false)
                return
            }

            cellRepresentation.current = getRepresentationResponse.data

            setIsLoading(false)
            toast.success("Reloaded Viewer")
        },
        [tabs.openedTab, props.tilesheets]
    )

    useEffect(() => {
        const closeLocalTabHandler = async (t: CloseLocalTabEvent) => {
            props.tilesheets.current.clearAll()
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

        const keydownHandler = (e: KeyboardEvent) => {
            if (e.key === "PageUp") {
                zLevel.current += 1
                props.tilesheets.current.switchZLevel(zLevel.current)
                props.eventBus.current.dispatchEvent(
                    new ChangeZLevelEvent(
                        LocalEvent.CHANGE_Z_LEVEL,
                        {detail: {zLevel: zLevel.current}}
                    )
                )
            } else if (e.key === "PageDown") {
                zLevel.current -= 1
                props.tilesheets.current.switchZLevel(zLevel.current)
                props.eventBus.current.dispatchEvent(
                    new ChangeZLevelEvent(
                        LocalEvent.CHANGE_Z_LEVEL,
                        {detail: {zLevel: zLevel.current}}
                    )
                )
            }
        }

        props.canvas.canvasRef.current.addEventListener("keydown", keydownHandler)

        props.eventBus.current.addEventListener(
            LocalEvent.CLOSE_LOCAL_TAB,
            closeLocalTabHandler
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
            props.canvas.canvasRef.current.removeEventListener("keydown", keydownHandler)

            props.eventBus.current.removeEventListener(
                LocalEvent.CLOSE_LOCAL_TAB,
                closeLocalTabHandler
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
    }, [props.eventBus, props.tilesheets, setupSceneData]);

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

        props.eventBus.current.dispatchEvent(
            new ChangeZLevelEvent(
                LocalEvent.CHANGE_Z_LEVEL,
                {detail: {zLevel: zLevel.current}}
            )
        )

        function loop() {
            if (SHOW_STATS) props.threeConfig.current.stats.begin()

            props.threeConfig.current.camera.updateProjectionMatrix()
            if (props.tilesheets.current) props.tilesheets.current.updateAnimatedSprites(zLevel.current)

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
                    props.setSidebarContent({})
                    props.eventBus.current.dispatchEvent(
                        new ChangeSelectedPositionEvent(
                            LocalEvent.CHANGE_SELECTED_POSITION,
                            {detail: {position: null}}
                        )
                    )
                } else {
                    selectedCellMeshRef.current.position.set(
                        worldMousePosition.current.x * props.tileInfo.width,
                        (-worldMousePosition.current.y - 1) * props.tileInfo.height,
                        MAX_DEPTH + 1
                    )
                    selectedCellMeshRef.current.visible = true
                    setSelectedCellPosition(worldMousePosition.current)

                    props.eventBus.current.dispatchEvent(
                        new ChangeSelectedPositionEvent(
                            LocalEvent.CHANGE_SELECTED_POSITION,
                            {detail: {position: {x: worldMousePosition.current.x, y: worldMousePosition.current.y}}}
                        )
                    )

                    const positionString = `${worldMousePosition.current.x},${worldMousePosition.current.y},${zLevel.current}`

                    const selectedMapZ = cellRepresentation.current[zLevel.current]

                    if (!selectedMapZ) {
                        props.setSidebarContent({})
                        return
                    }

                    const selectedRepr = selectedMapZ[positionString]

                    if (!selectedRepr) {
                        props.setSidebarContent({})
                        return
                    }

                    const newSidebarContent: SidebarContent = {
                        chosenProperties:
                            <div className={"sidebar-chosen-properties"}>
                                {
                                    selectedRepr.terrain &&
                                    <fieldset>
                                        <legend>Terrain</legend>

                                        terrain: {selectedRepr.terrain.tilesheet_id.id}
                                    </fieldset>
                                }
                                {
                                    selectedRepr.furniture &&
                                    <fieldset>
                                        <legend>Furniture</legend>

                                        furniture: {selectedRepr.furniture.tilesheet_id.id}
                                    </fieldset>
                                }
                                {
                                    selectedRepr.field &&
                                    <fieldset>
                                        <legend>Field</legend>

                                        field: {selectedRepr.field.tilesheet_id.id}
                                    </fieldset>
                                }
                                {
                                    selectedRepr.monster &&
                                    <fieldset>
                                        <legend>Monster</legend>

                                        monster: {selectedRepr.monster.tilesheet_id.id}
                                    </fieldset>

                                }
                            </div>
                    }

                    props.setSidebarContent(newSidebarContent)
                }
            }
        }

        props.canvas.canvasRef.current.addEventListener("mousedown", onMouseDown)

        return () => {
            props.canvas.canvasRef.current.removeEventListener("mousedown", onMouseDown)
        }
    }, [props.eventBus, props.tileInfo, selectedCellPosition, worldMousePosition]);

    return <>
        <div className={clsx("loader-container", isLoading && "visible")}>
            <div className={"loader"}/>
            <span>Loading Map Data</span>
        </div>
    </>
}