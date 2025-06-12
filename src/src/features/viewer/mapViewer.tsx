import {DrawAnimatedSprite, DrawStaticSprite, Tilesheets} from "../sprites/tilesheets.js";
import React, {RefObject, useContext, useEffect, useRef} from "react";
import {SHOW_STATS} from "../three/hooks/useThreeSetup.js";
import {Canvas, ThreeConfig} from "../three/types/three.ts";
import {GridHelper, Vector2} from "three";
import {getColorFromTheme, Theme} from "../../shared/hooks/useTheme.js";
import {degToRad} from "three/src/math/MathUtils.js";
import {TileInfo} from "../../tauri/types/spritesheet.js";
import {TabContext, ThemeContext} from "../../app.js";
import Icon, {IconName} from "../../shared/components/icon.js";
import {SideMenuRef} from "../../shared/components/imguilike/sideMenu.js";
import {logRender} from "../../shared/utils/log.js";
import {LocalEvent} from "../../shared/utils/localEvent.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {BackendResponseType, serializedVec2ToVector2, Sprites, TauriCommand} from "../../tauri/events/types.js";
import toast from "react-hot-toast";

export type MapViewerProps = {
    tilesheets: RefObject<Tilesheets>
    tileInfo: TileInfo
    threeConfig: RefObject<ThreeConfig>
    canvas: Canvas
    sideMenuRef: RefObject<SideMenuRef>
    eventBus: RefObject<EventTarget>
}

export enum MapViewerTab {
    MapInfo = "map-info"
}

export function MapViewer(props: MapViewerProps) {
    const grid = useRef<GridHelper>(null)
    const zLevel = useRef<number>(0)

    const {theme} = useContext(ThemeContext)
    const tabs = useContext(TabContext)


    // Main Draw Loop
    useEffect(() => {
        logRender("Setting up main draw loop")

        let handler: number;

        {
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
                const gridHelper = new GridHelper(
                    1,
                    16 * 8 * props.tileInfo.width * 24 / props.tileInfo.height,
                    getColorFromTheme(theme, "disabled"), getColorFromTheme(theme, "light")
                )

                gridHelper.scale.x = 16 * 8 * props.tileInfo.width * 24
                gridHelper.scale.z = 16 * 8 * props.tileInfo.height * 24

                gridHelper.position.x -= props.tileInfo.width / 2
                gridHelper.position.y -= props.tileInfo.height / 2

                gridHelper.rotateX(degToRad(90))

                if (grid.current) {
                    props.threeConfig.current.scene.remove(grid.current)
                    grid.current.dispose()
                    grid.current = null
                }

                props.threeConfig.current.scene.add(gridHelper)
                grid.current = gridHelper
            }

            function setupSideMenuTabs() {
                props.sideMenuRef.current.registerTab(
                    MapViewerTab.MapInfo,
                    {
                        icon: <Icon name={IconName.InfoMedium}/>,
                        content: <div>
                            Test
                        </div>
                    }
                )
            }

            setupGrid(theme)
            setRenderBounds()
            setColors(theme)
            setupSideMenuTabs()
        }

        async function clearAndLoadSprites() {
            props.tilesheets.current.clearAll()

            const response = await tauriBridge.invoke<Sprites, unknown, TauriCommand.GET_SPRITES>(TauriCommand.GET_SPRITES, {})

            if (response.type === BackendResponseType.Error) {
                toast.error(response.error)
                return
            }

            const drawStaticSprites: DrawStaticSprite[] = response.data.static_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= props.tileInfo.width;
                vec2.y *= props.tileInfo.height;

                return {
                    ...ds,
                    position: vec2
                }
            })

            const drawAnimatedSprites: DrawAnimatedSprite[] = response.data.animated_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= props.tileInfo.width;
                vec2.y *= props.tileInfo.height;

                return {
                    ...ds,
                    position: vec2,
                }
            })

            const drawFallbackSprites: DrawStaticSprite[] = response.data.fallback_sprites.map(ds => {
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
        }

        props.eventBus.current.addEventListener(
            LocalEvent.TILESET_LOADED,
            clearAndLoadSprites,
        )

        if (props.tilesheets) clearAndLoadSprites()

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
            grid.current.dispose()

            props.sideMenuRef.current.removeTab(MapViewerTab.MapInfo)
            props.eventBus.current.removeEventListener(
                LocalEvent.TILESET_LOADED,
                clearAndLoadSprites,
            )
        }
    }, [props.tileInfo, tabs.openedTab, theme])

    return (
        <div>

        </div>
    )
}