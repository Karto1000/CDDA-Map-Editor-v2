import React, {RefObject, useContext, useEffect, useRef} from "react"
import "./mapEditor.scss"
import {TabContext, ThemeContext} from "../../app.js";
import {getTileInfo, SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {Tilesheets} from "../sprites/tilesheets.js";
import {Canvas, ThreeConfig} from "../three/types/three.js";
import {Object3D} from "three";
import {getColorFromTheme, Theme} from "../../shared/hooks/useTheme.js";
import {createGrid} from "../three/hooks/useThreeSetup.js";
import {MapEditorData} from "../../tauri/types/editor.js";
import {useCurrentProject} from "../../shared/hooks/useCurrentProject.js";
import {openWindow, WindowLabel} from "../../windows/lib.js";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {UnlistenFn} from "@tauri-apps/api/event";
import {useTauriEvent} from "../../shared/hooks/useTauriEvent.js";
import {TauriEvent} from "../../tauri/events/types.js";

export type MapEditorProps = {
    spritesheetConfig: RefObject<SpritesheetConfig>
    tilesheets: RefObject<Tilesheets>
    threeConfig: RefObject<ThreeConfig>
    canvas: Canvas
    showGridRef: RefObject<boolean>
    mapInfoWindowRef: RefObject<WebviewWindow>
    palettesWindowRef: RefObject<WebviewWindow>
}

export function MapEditor(props: MapEditorProps) {
    const tabs = useContext(TabContext)
    const theme = useContext(ThemeContext)
    const grid = useRef<Object3D>(null)
    const project = useCurrentProject<MapEditorData>(tabs.openedTab)

    const mapInfoUnlistenFn = useRef<UnlistenFn>(null)
    const palettesUnlistenFn = useRef<UnlistenFn>(null)

    let handler: number;

    useTauriEvent(
        TauriEvent.TOGGLE_GRID,
        data => {
            grid.current.visible = data.state
        },
        []
    )

    useTauriEvent(
        TauriEvent.OPEN_MAPGEN_INFO_WINDOW,
        _ => {
            openWindow(WindowLabel.MapInfo, theme.theme, {}).then(value => {
                const [window, close] = value
                mapInfoUnlistenFn.current = close
                props.mapInfoWindowRef.current = window
            })
        }
    )

    useTauriEvent(
        TauriEvent.OPEN_PALETTES_WINDOW,
        _ => {
            openWindow(WindowLabel.Palettes, theme.theme, {}).then(value => {
                const [window, close] = value
                palettesUnlistenFn.current = close
                props.palettesWindowRef.current = window
            })
        }
    )

    useEffect(() => {
        if (!project) return
        if ("mapViewer" in project.project_type) return

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

            const gridWidth = project.project_type.mapEditor.size[0] * tileInfo.width / 2
            const gridHeight = project.project_type.mapEditor.size[1] * tileInfo.height / 2

            const gridHelper = createGrid(
                {
                    width: gridWidth,
                    height: gridHeight,
                    linesHeight: gridHeight / tileInfo.height * 2,
                    linesWidth: gridWidth / tileInfo.width * 2,
                    color: getColorFromTheme(theme, "disabled")
                }
            )

            gridHelper.position.x += gridWidth - tileInfo.width / 2
            gridHelper.position.y += -gridHeight - tileInfo.height / 2

            if (grid.current) {
                props.threeConfig.current.scene.remove(grid.current)
                grid.current = null
            }

            props.threeConfig.current.scene.add(gridHelper)
            grid.current = gridHelper
            grid.current.visible = props.showGridRef.current
        }

        setRenderBounds()
        setupGrid(theme.theme)

        function loop() {
            props.threeConfig.current.camera.updateProjectionMatrix()

            props.threeConfig.current.controls.update()
            props.threeConfig.current.renderer.render(props.threeConfig.current.scene, props.threeConfig.current.camera)

            handler = requestAnimationFrame(loop)
        }

        loop()

        return () => {
            cancelAnimationFrame(handler)

            props.threeConfig.current.scene.remove(grid.current)
            props.tilesheets.current.clearAll()
        }
    }, [project, theme]);

    useEffect(() => {
        return () => {
            if (mapInfoUnlistenFn.current) mapInfoUnlistenFn.current()
            if (palettesUnlistenFn.current) palettesUnlistenFn.current()
        }
    }, []);

    return (
        <div>
        </div>
    )
}