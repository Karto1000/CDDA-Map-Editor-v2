import {Canvas, ThreeConfig} from "../three/types/three.js";
import {RefObject, useContext, useEffect, useRef} from "react";
import {getTileInfo, SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {CHUNK_SIZE, MapEditorMode} from "./mapEditor.js";
import {BoxHelper, Mesh, MeshBasicMaterial, PlaneGeometry, Vector3} from "three";
import {getColorFromTheme} from "../../shared/hooks/useTheme.js";
import {TabContext, ThemeContext} from "../../app.js";
import {useWorldMousePosition} from "../three/hooks/useWorldMousePosition.js";
import {MapEditorData, Project} from "../../tauri/types/editor.js";
import {TextGeometry} from "three/examples/jsm/geometries/TextGeometry.js";
import {openWindow, WindowLabel} from "../../windows/lib.js";
import {emit, UnlistenFn} from "@tauri-apps/api/event";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {TauriEvent} from "../../tauri/events/types.js";

const TEXT_SIZE = 6

export function useChunkSelect(
    threeConfig: RefObject<ThreeConfig>,
    spritesheetConfig: RefObject<SpritesheetConfig>,
    canvas: Canvas,
    mapEditorMode: MapEditorMode,
    project: Project<MapEditorData>,
    z: RefObject<number>,
    palettesWindowRef: RefObject<WebviewWindow>
) {
    const theme = useContext(ThemeContext)
    const tabs = useContext(TabContext)

    const chunkSelectInnerMeshRef = useRef<Mesh>(null)
    const chunkSelectOuterMeshRef = useRef<BoxHelper>(null)
    const textRef = useRef<Mesh>(null)

    const palettesUnlistenFn = useRef<UnlistenFn>(null)

    const worldMousePosition = useWorldMousePosition(
        {
            spritesheetConfig,
            threeConfig,
            canvas,
        }
    )

    function isMouseOutsideBounds() {
        const isSmallerThan0 = worldMousePosition.x < 0 || worldMousePosition.y < 0
        const isGreaterThanMapSize = worldMousePosition.x >= project.project_type.mapEditor.size[z.current] ||
            worldMousePosition.y >= project.project_type.mapEditor.size[1]

        return isSmallerThan0 || isGreaterThanMapSize
    }

    function updateMeshPositions() {
        const tileInfo = getTileInfo(spritesheetConfig.current)

        const chunkSizeX = Math.min(project.project_type.mapEditor.size[0], CHUNK_SIZE)
        const chunkSizeY = Math.min(project.project_type.mapEditor.size[1], CHUNK_SIZE)

        const currentChunkX = Math.floor(worldMousePosition.x / chunkSizeX)
        const currentChunkY = Math.floor(worldMousePosition.y / chunkSizeY)

        if (isMouseOutsideBounds()) {
            chunkSelectInnerMeshRef.current.visible = false
            chunkSelectOuterMeshRef.current.visible = false
            textRef.current.visible = false
            return
        }

        chunkSelectInnerMeshRef.current.visible = true
        chunkSelectOuterMeshRef.current.visible = true
        textRef.current.visible = true

        // TODO: z-level
        const map = project.project_type.mapEditor.maps[0].maps[`${currentChunkX},${currentChunkY}`]

        const halfOffsetX = tileInfo.width / 2
        const halfOffsetY = tileInfo.height / 2

        const chunkSizePixelsX = tileInfo.width * chunkSizeX
        const chunkSizePixelsY = tileInfo.height * chunkSizeY

        const posX = chunkSizePixelsX * (currentChunkX + 1) - chunkSizePixelsX / 2 - halfOffsetX
        const posY = -(chunkSizePixelsY * (currentChunkY + 1) - chunkSizePixelsY / 2 + halfOffsetY)

        chunkSelectInnerMeshRef.current.position.set(
            posX,
            posY,
            0
        )

        chunkSelectOuterMeshRef.current.update()

        textRef.current.geometry = new TextGeometry(
            `${map.id} at ${currentChunkX}, ${currentChunkY}`,
            {
                font: threeConfig.current.font,
                size: TEXT_SIZE
            }
        )

        const textOffsetXPx = TEXT_SIZE / 2
        const textOffsetYPx = TEXT_SIZE / 2

        textRef.current.position.set(-chunkSizePixelsX / 2 + textOffsetXPx, chunkSizePixelsY / 2 - TEXT_SIZE - textOffsetYPx, 0)
    }

    useEffect(() => {
        async function onMouseDown(e: MouseEvent) {
            if (e.button !== 0) return;
            if (isMouseOutsideBounds()) return

            const chunkSizeX = Math.min(project.project_type.mapEditor.size[0], CHUNK_SIZE)
            const chunkSizeY = Math.min(project.project_type.mapEditor.size[1], CHUNK_SIZE)

            const currentChunkX = Math.floor(worldMousePosition.x / chunkSizeX)
            const currentChunkY = Math.floor(worldMousePosition.y / chunkSizeY)

            if (!palettesWindowRef.current) {
                palettesUnlistenFn.current = (await openWindow(
                    WindowLabel.Chunk,
                    theme.theme,
                    palettesWindowRef,
                    {},
                    new Vector3(currentChunkX, currentChunkY, z.current)
                ))[1]
            }

            await emit(
                TauriEvent.MAPGEN_CHUNK_SELECTED,
                new Vector3(currentChunkX, currentChunkY, z.current)
            )
        }

        switch (mapEditorMode) {
            case MapEditorMode.ChunkSelect:
                if (!chunkSelectInnerMeshRef.current) return;
                if (!chunkSelectOuterMeshRef.current) return;

                updateMeshPositions()

                canvas.canvasRef.current.addEventListener("mousedown", onMouseDown)
        }

        return () => {
            canvas.canvasRef.current.removeEventListener("mousedown", onMouseDown)
        }
    }, [mapEditorMode, worldMousePosition, project]);

    useEffect(() => {
        return () => {
            if (palettesUnlistenFn.current) palettesUnlistenFn.current()
            palettesUnlistenFn.current = null
            palettesWindowRef.current = null
        }
    }, [tabs.openedTab, theme]);

    useEffect(() => {
        switch (mapEditorMode) {
            case MapEditorMode.ChunkSelect:
                const tileInfo = getTileInfo(spritesheetConfig.current)

                const projectSize = project.project_type.mapEditor.size
                const chunkSizeX = Math.min(projectSize[0], CHUNK_SIZE)
                const chunkSizeY = Math.min(projectSize[1], CHUNK_SIZE)

                const chunkSelectGeo = new PlaneGeometry(tileInfo.width * chunkSizeX, tileInfo.height * chunkSizeY)
                const chunkSelectMaterial = new MeshBasicMaterial({color: getColorFromTheme(theme.theme, "darkBlue")})
                chunkSelectMaterial.transparent = true
                chunkSelectMaterial.opacity = 0.2

                const chunkSelectInnerMesh = new Mesh(chunkSelectGeo, chunkSelectMaterial)

                const boxHelper = new BoxHelper(
                    chunkSelectInnerMesh,
                    getColorFromTheme(theme.theme, "selected")
                )
                boxHelper.material.depthTest = false
                boxHelper.material.transparent = true

                const textGeometry = new TextGeometry("", {font: threeConfig.current.font, size: TEXT_SIZE})

                const textMaterial = new MeshBasicMaterial({color: getColorFromTheme(theme.theme, "selected")})
                const textMesh = new Mesh(textGeometry, textMaterial)

                chunkSelectInnerMeshRef.current = chunkSelectInnerMesh
                chunkSelectOuterMeshRef.current = boxHelper
                textRef.current = textMesh

                chunkSelectInnerMesh.add(textMesh)

                threeConfig.current.scene.add(chunkSelectInnerMesh)
                threeConfig.current.scene.add(boxHelper)

                updateMeshPositions()
        }

        return () => {
            threeConfig.current.scene.remove(chunkSelectInnerMeshRef.current)
            chunkSelectInnerMeshRef.current = null

            threeConfig.current.scene.remove(chunkSelectOuterMeshRef.current)
            chunkSelectOuterMeshRef.current = null
        }
    }, [mapEditorMode, theme, project]);
}