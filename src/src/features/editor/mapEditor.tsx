import React, {RefObject, useContext, useEffect, useRef} from "react"
import "./mapEditor.scss"
import {TabContext, ThemeContext} from "../../app.js";
import {getTileInfo, SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {Tilesheets} from "../sprites/tilesheets.js";
import {Canvas, ThreeConfig} from "../three/types/three.js";
import {SideMenuRef} from "../../shared/components/imguilike/sideMenu.js";
import {Object3D, Vector2} from "three";
import {getColorFromTheme, Theme} from "../../shared/hooks/useTheme.js";
import {createGrid} from "../three/hooks/useThreeSetup.js";
import {MapEditorData} from "../../tauri/types/editor.js";
import {useCurrentProject} from "../../shared/hooks/useCurrentProject.js";
import {LocalEvent, ToggleGridEvent} from "../../shared/utils/localEvent.js";

export type MapEditorProps = {
    spritesheetConfig: RefObject<SpritesheetConfig>
    tilesheets: RefObject<Tilesheets>
    threeConfig: RefObject<ThreeConfig>
    canvas: Canvas
    eventBus: RefObject<EventTarget>
    showGridRef: RefObject<boolean>
}

export function MapEditor(props: MapEditorProps) {
    const tabs = useContext(TabContext)
    const theme = useContext(ThemeContext)
    const grid = useRef<Object3D>(null)
    const project = useCurrentProject<MapEditorData>(tabs)

    let handler: number;

    useEffect(() => {
        if (!project) return

        function setRenderBounds() {
            const newWidth = props.canvas.canvasContainerRef.current.clientWidth
            const newHeight = props.canvas.canvasContainerRef.current.clientHeight

            props.threeConfig.current.renderer.setSize(newWidth, newHeight)
            props.threeConfig.current.camera.left = newWidth / -2
            props.threeConfig.current.camera.right = newWidth / 2
            props.threeConfig.current.camera.top = newHeight / 2
            props.threeConfig.current.camera.bottom = newHeight / -2
            props.threeConfig.current.camera.position.z = 999999
            console.log(newWidth, newHeight)
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

        function onToggleGrid(e: ToggleGridEvent) {
            grid.current.visible = e.detail.state
        }

        props.eventBus.current.addEventListener(
            LocalEvent.TOGGLE_GRID,
            onToggleGrid,
        )

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

            props.eventBus.current.removeEventListener(
                LocalEvent.TOGGLE_GRID,
                onToggleGrid
            )

            props.tilesheets.current.clearAll()
        }
    }, [project, theme]);

    return (
        <div>
        </div>
    )
}