import React, {RefObject, useContext, useEffect, useRef, useState} from "react"
import "./mapEditor.scss"
import {EditorDataContext, TabContext, ThemeContext} from "../../app.js";
import {getTileInfo, SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {Tilesheets} from "../sprites/tilesheets.js";
import {Canvas, ThreeConfig} from "../three/types/three.js";
import {Object3D} from "three";
import {getColorFromTheme, Theme} from "../../shared/hooks/useTheme.js";
import {createGrid} from "../three/hooks/useThreeSetup.js";
import {KeybindAction, MapEditorData} from "../../tauri/types/editor.js";
import {useCurrentProject} from "../../shared/hooks/useCurrentProject.js";
import {openWindow, WindowLabel} from "../../windows/lib.js";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {emit, UnlistenFn} from "@tauri-apps/api/event";
import {useTauriEvent} from "../../shared/hooks/useTauriEvent.js";
import {TauriCommand, TauriEvent} from "../../tauri/events/types.js";
import {useChunkSelect} from "./useChunkSelect.js";
import {useKeybindings} from "../../shared/hooks/useKeybindings.js";
import {InitialGlobalPaletteData} from "../../windows/global-character-select/main.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";

export type MapEditorProps = {
    spritesheetConfig: RefObject<SpritesheetConfig>
    tilesheets: RefObject<Tilesheets>
    threeConfig: RefObject<ThreeConfig>
    canvas: Canvas
    showGridRef: RefObject<boolean>
    mapInfoWindowRef: RefObject<WebviewWindow>
    chunkInfoWindowRef: RefObject<WebviewWindow>
    globalPalettesWindowRef: RefObject<WebviewWindow>
    globalCharacterSelectWindowRef: RefObject<WebviewWindow>
}

export enum MapEditorMode {
    Draw = "DRAW",
    Fill = "FILL",
    ChunkSelect = "CHUNK_SELECT"
}

export const CHUNK_SIZE = 24

export function MapEditor(props: MapEditorProps) {
    const tabs = useContext(TabContext)
    const theme = useContext(ThemeContext)
    const programData = useContext(EditorDataContext)
    const project = useCurrentProject<MapEditorData>(tabs.openedTab)

    const grid = useRef<Object3D>(null)

    const mapInfoUnlistenFn = useRef<UnlistenFn>(null)
    const globalPalettesUnlistenRef = useRef<UnlistenFn>(null)
    const globalCharacterSelectUnlistenRef = useRef<UnlistenFn>(null)

    const [mapEditorMode, setMapEditorMode] = useState<MapEditorMode>(MapEditorMode.Draw)
    const [selectedCharacter, setSelectedCharacter] = useState<string>(null)

    const zLevel = useRef<number>(0)

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

    useChunkSelect(
        props.threeConfig,
        props.spritesheetConfig,
        props.canvas,
        mapEditorMode,
        project,
        zLevel,
        props.chunkInfoWindowRef
    )

    useKeybindings(
        props.canvas.canvasRef.current,
        programData,
        []
    )

    let handler: number;

    useTauriEvent(
        TauriEvent.UPDATE_CDDA_DATA,
        (paths) => {
            (async () => {
                await tauriBridge.invoke(
                    TauriCommand.UPDATE_CDDA_DATA_AT,
                    {paths}
                )
            })()
        },
        []
    )

    useTauriEvent(
        TauriEvent.KEYBIND_PRESSED,
        kb => {
            let newMode: MapEditorMode = null;

            switch (kb) {
                case KeybindAction.ChunkSelect:
                    newMode = MapEditorMode.ChunkSelect
                    break;
                case KeybindAction.Draw:
                    newMode = MapEditorMode.Draw
                    break;
                case KeybindAction.Fill:
                    newMode = MapEditorMode.Fill
                    break
            }

            if (newMode != null)
                emit(
                    TauriEvent.CHANGE_EDITOR_MODE,
                    newMode
                )
        },
        []
    )

    useTauriEvent(
        TauriEvent.CHANGE_EDITOR_MODE,
        mode => {
            setMapEditorMode(mode)
        },
        [mapEditorMode]
    )

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
            (async () => {
                const [window, close] = await openWindow(WindowLabel.MapInfo, theme.theme, props.mapInfoWindowRef, {})
                mapInfoUnlistenFn.current = close
            })()
        },
        []
    )

    useTauriEvent(
        TauriEvent.OPEN_GLOBAL_PALETTES_WINDOW,
        _ => {
            (async () => {
                const [window, close] = await openWindow(WindowLabel.GlobalPalettes, theme.theme, props.globalPalettesWindowRef, {})
                globalPalettesUnlistenRef.current = close
            })()
        },
        []
    )

    useTauriEvent(
        TauriEvent.OPEN_GLOBAL_SELECT_WINDOW,
        _ => {
            (async () => {
                const [window, close] = await openWindow<InitialGlobalPaletteData>(
                    WindowLabel.GlobalCharacterSelect,
                    theme.theme,
                    props.globalCharacterSelectWindowRef,
                    {},
                    {
                        slimTilesheets: props.tilesheets.current.toSlimTilesheets(),
                        spritesheetConfig: props.spritesheetConfig,
                        selectedCharacter
                    }
                )
                globalCharacterSelectUnlistenRef.current = close
            })()
        },
        [selectedCharacter]
    )

    useTauriEvent(
        TauriEvent.CHARACTER_SELECTED,
        data => {
            setSelectedCharacter(data.character)
        },
        [selectedCharacter]
    )

    useTauriEvent(
        TauriEvent.TILESET_LOADED,
        () => {
            setupGrid(theme.theme)
        },
        [theme, project]
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
            props.mapInfoWindowRef.current = null
            props.chunkInfoWindowRef.current = null
            props.globalPalettesWindowRef.current = null

            if (mapInfoUnlistenFn.current) mapInfoUnlistenFn.current()
            if (globalPalettesUnlistenRef.current) globalPalettesUnlistenRef.current()
            if (globalCharacterSelectUnlistenRef.current) globalCharacterSelectUnlistenRef.current()
        }
    }, []);

    return (
        <>
            <div className={"editor-mode"}>
                {mapEditorMode}
            </div>
        </>
    )
}