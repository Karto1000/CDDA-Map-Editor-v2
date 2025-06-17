import {DrawAnimatedSprite, DrawStaticSprite, MAX_DEPTH, Tilesheets} from "../sprites/tilesheets.js";
import React, {RefObject, useContext, useEffect, useReducer, useRef, useState} from "react";
import {SHOW_STATS} from "../three/hooks/useThreeSetup.js";
import {Canvas, ThreeConfig} from "../three/types/three.ts";
import {GridHelper, Vector3} from "three";
import {getColorFromTheme, Theme} from "../../shared/hooks/useTheme.js";
import {degToRad} from "three/src/math/MathUtils.js";
import {getTileInfo, SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {TabContext, ThemeContext} from "../../app.js";
import Icon, {IconName} from "../../shared/components/icon.js";
import {SideMenuRef} from "../../shared/components/imguilike/sideMenu.js";
import {logRender} from "../../shared/utils/log.js";
import {LocalEvent, ToggleGridEvent} from "../../shared/utils/localEvent.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {
    BackendResponseType,
    serializedVec2ToVector2,
    Sprites,
    TauriCommand,
    TauriEvent
} from "../../tauri/events/types.js";
import toast from "react-hot-toast";
import {useTauriEvent} from "../../shared/hooks/useTauriEvent.js";
import "./mapViewer.scss"
import {useWorldMousePosition} from "../three/hooks/useWorldMousePosition.js";
import {useMouseCells} from "../three/hooks/useMouseCells.js";
import {clsx} from "clsx";
import {useKeybindActionEvent} from "../../shared/hooks/useKeybindings.js";
import {KeybindAction} from "../../tauri/types/editor.js";

export type MapViewerProps = {
    spritesheetConfig: RefObject<SpritesheetConfig>
    tilesheets: RefObject<Tilesheets>
    threeConfig: RefObject<ThreeConfig>
    canvas: Canvas
    sideMenuRef: RefObject<SideMenuRef>
    eventBus: RefObject<EventTarget>
    showGridRef: RefObject<boolean>
}

export enum MapViewerTab {
    MapInfo = "map-info"
}

const CHUNK_SIZE = 24

export function MapViewer(props: MapViewerProps) {
    const [, forceUpdate] = useReducer(x => x + 1, 0);
    const grid = useRef<GridHelper>(null)
    const zLevel = useRef<number>(0)

    const {theme} = useContext(ThemeContext)
    const tabs = useContext(TabContext)

    const {
        hoveredCellMeshRef,
        selectedCellMeshRef,
        updateCellSize
    } = useMouseCells(props.threeConfig, props.spritesheetConfig, theme)
    const worldMousePosition = useWorldMousePosition(
        {
            spritesheetConfig: props.spritesheetConfig,
            threeConfig: props.threeConfig,
            canvas: props.canvas,
            onMouseMove: (mousePosition) => {
                const tileInfo = getTileInfo(props.spritesheetConfig.current)

                hoveredCellMeshRef.current.position.set(
                    mousePosition.x * tileInfo.width,
                    -mousePosition.y * tileInfo.height - tileInfo.height,
                    MAX_DEPTH + 1
                )
            }
        }
    )
    const [selectedMousePosition, setSelectedMousePosition] = useState<Vector3 | null>(null)
    const [isLoading, setIsLoading] = useState<boolean>(false)

    useKeybindActionEvent(
        KeybindAction.ReloadMap,
        onReload,
        props.eventBus,
        []
    )

    async function clearAndLoadSprites() {
        const tileInfo = getTileInfo(props.spritesheetConfig.current)

        props.tilesheets.current.clearAll()

        const response = await tauriBridge.invoke<Sprites, string>(TauriCommand.GET_SPRITES, {})

        if (response.type === BackendResponseType.Error) {
            toast.error(response.error)
            return
        }

        const drawStaticSprites: DrawStaticSprite[] = response.data.static_sprites.map(ds => {
            const vec2 = serializedVec2ToVector2(ds.position)
            vec2.x *= tileInfo.width;
            vec2.y *= tileInfo.height;

            return {
                ...ds,
                position: vec2
            }
        })

        const drawAnimatedSprites: DrawAnimatedSprite[] = response.data.animated_sprites.map(ds => {
            const vec2 = serializedVec2ToVector2(ds.position)
            vec2.x *= tileInfo.width;
            vec2.y *= tileInfo.height;

            return {
                ...ds,
                position: vec2,
            }
        })

        const drawFallbackSprites: DrawStaticSprite[] = response.data.fallback_sprites.map(ds => {
            const vec2 = serializedVec2ToVector2(ds.position)
            vec2.x *= tileInfo.width;
            vec2.y *= tileInfo.height;

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
    }

    async function onReload() {
        setIsLoading(true)

        await tauriBridge.invoke<unknown, string>(TauriCommand.RELOAD_PROJECT, {})
        await clearAndLoadSprites()
        toast.success("Reloaded viewer")

        setIsLoading(false)
    }

    // We receive this event any time the files the current project is linked to change
    useTauriEvent(
        TauriEvent.UPDATE_LIVE_VIEWER,
        () => {
            (async () => {
                await onReload()
            })()
        },
        []
    )

    useEffect(() => {
        function onMouseDown(e: MouseEvent) {
            if (e.button !== 0) return;

            if (selectedMousePosition) {
                const selectedSamePosition = selectedMousePosition.x === worldMousePosition.x && selectedMousePosition.y === worldMousePosition.y
                if (selectedSamePosition) {
                    setSelectedMousePosition(null)
                    selectedCellMeshRef.current.visible = false
                    return
                }
            }

            const tileInfo = getTileInfo(props.spritesheetConfig.current)

            selectedCellMeshRef.current.position.set(
                worldMousePosition.x * tileInfo.width,
                -worldMousePosition.y * tileInfo.height - tileInfo.height,
                MAX_DEPTH + 1
            )
            selectedCellMeshRef.current.visible = true

            setSelectedMousePosition(worldMousePosition)
        }

        async function onKeyDown(e: KeyboardEvent) {
            if (e.key === "PageUp") {
                zLevel.current += 1
                props.tilesheets.current.switchZLevel(zLevel.current)
                forceUpdate()
            } else if (e.key === "PageDown") {
                zLevel.current -= 1
                props.tilesheets.current.switchZLevel(zLevel.current)
                forceUpdate()
            }
        }

        props.canvas.canvasRef.current.addEventListener("keydown", onKeyDown)
        props.canvas.canvasRef.current.addEventListener("mousedown", onMouseDown)

        return () => {
            props.canvas.canvasRef.current.removeEventListener("keydown", onKeyDown)
            props.canvas.canvasRef.current.removeEventListener("mousedown", onMouseDown)
        }
    }, [worldMousePosition, selectedMousePosition]);

    // Main Draw Loop
    useEffect(() => {
        logRender("Setting up main draw loop")

        let handler: number;

        function setColors(theme: Theme) {
            props.threeConfig.current.renderer.setClearColor(getColorFromTheme(theme, "darker"))
        }

        function setRenderBounds() {
            const newWidth = props.canvas.canvasContainerRef.current.clientWidth
            const newHeight = props.canvas.canvasContainerRef.current.clientHeight

            props.threeConfig.current.renderer.setSize(newWidth, newHeight)
            props.threeConfig.current.camera.left = newWidth / -2
            props.threeConfig.current.camera.right = newWidth / 2
            props.threeConfig.current.camera.top = newHeight / 2
            props.threeConfig.current.camera.bottom = newHeight / -2
            props.threeConfig.current.camera.position.z = 999999
        }

        function setupGrid(theme: Theme) {
            const tileInfo = getTileInfo(props.spritesheetConfig.current)

            const gridHelper = new GridHelper(
                1,
                16 * 8 * tileInfo.width * 24 / tileInfo.height,
                getColorFromTheme(theme, "disabled"), getColorFromTheme(theme, "light")
            )

            gridHelper.scale.x = 16 * 8 * tileInfo.width * 24
            gridHelper.scale.z = 16 * 8 * tileInfo.height * 24

            gridHelper.position.x -= tileInfo.width / 2
            gridHelper.position.y -= tileInfo.height / 2

            gridHelper.rotateX(degToRad(90))

            if (grid.current) {
                props.threeConfig.current.scene.remove(grid.current)
                grid.current.dispose()
                grid.current = null
            }

            props.threeConfig.current.scene.add(gridHelper)
            grid.current = gridHelper
            grid.current.visible = props.showGridRef.current
        }

        function setupSideMenuTabs() {
            props.sideMenuRef.current.registerTab(
                MapViewerTab.MapInfo,
                {
                    icon: <Icon name={IconName.InfoMedium}/>,
                    content: <div>

                    </div>
                }
            )
        }

        setupGrid(theme)
        setRenderBounds()
        setColors(theme)
        setupSideMenuTabs()

        async function onTilesetLoaded() {
            setIsLoading(true)

            setupGrid(theme)
            await clearAndLoadSprites()
            updateCellSize()

            setIsLoading(false)
        }

        function onToggleGrid(e: ToggleGridEvent) {
            grid.current.visible = e.detail.state
        }

        props.eventBus.current.addEventListener(
            LocalEvent.TILESET_LOADED,
            onTilesetLoaded,
        )

        props.eventBus.current.addEventListener(
            LocalEvent.TOGGLE_GRID,
            onToggleGrid,
        )

        if (props.tilesheets) {
            (async () => {
                setIsLoading(true)

                await clearAndLoadSprites()

                setIsLoading(false)
            })()
        }

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

            props.threeConfig.current.scene.remove(grid.current)
            grid.current.dispose()

            props.sideMenuRef.current.removeTab(MapViewerTab.MapInfo)
            props.tilesheets.current.clearAll()

            props.eventBus.current.removeEventListener(
                LocalEvent.TILESET_LOADED,
                onTilesetLoaded,
            )

            props.eventBus.current.removeEventListener(
                LocalEvent.TOGGLE_GRID,
                onToggleGrid,
            )
        }
    }, [tabs.openedTab, theme])

    return (
        <>
            <div className={clsx("loader-container", isLoading && "visible")}>
                <div className={"loader"}/>
                <span>Loading Map Data</span>
            </div>
            <div className={"top-right-objects"}>
                {
                    selectedMousePosition !== null &&
                    <div className={"selected-chunk-indicator"}>
                        <span>{Math.floor(selectedMousePosition.x / CHUNK_SIZE)}, {Math.floor(selectedMousePosition.y / CHUNK_SIZE)}</span>
                    </div>
                }
                {
                    selectedMousePosition !== null &&
                    <div className={"selected-position-indicator"}>
                        <span>{selectedMousePosition.x}, {selectedMousePosition.y}, {zLevel.current}</span>
                    </div>
                }
                {
                    worldMousePosition !== null &&
                    <div className={"world-chunk-indicator"}>
                        <span>{Math.floor(worldMousePosition.x / CHUNK_SIZE)}, {Math.floor(worldMousePosition.y / CHUNK_SIZE)}</span>
                    </div>
                }
                {
                    worldMousePosition !== null &&
                    <div className={"world-position-indicator"}>
                        <span>{worldMousePosition.x}, {worldMousePosition.y}, {zLevel.current}</span>
                    </div>
                }
                {
                    tabs.shouldDisplayCanvas() &&
                    <button onClick={onReload} className={"reload-button"}>
                        <Icon name={IconName.ReloadMedium} pointerEvents={"none"}/>
                    </button>
                }
            </div>
        </>

    )
}