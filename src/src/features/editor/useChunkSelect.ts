import {Canvas, ThreeConfig} from "../three/types/three.js";
import {RefObject, useContext, useEffect, useRef} from "react";
import {getTileInfo, SpritesheetConfig} from "../../tauri/types/spritesheet.js";
import {CHUNK_SIZE, MapEditorMode} from "./mapEditor.js";
import {BoxHelper, Mesh, MeshBasicMaterial, PlaneGeometry} from "three";
import {getColorFromTheme} from "../../shared/hooks/useTheme.js";
import {ThemeContext} from "../../app.js";
import {useWorldMousePosition} from "../three/hooks/useWorldMousePosition.js";
import {MapEditorData, Project} from "../../tauri/types/editor.js";

export function useChunkSelect(
    threeConfig: RefObject<ThreeConfig>,
    spritesheetConfig: RefObject<SpritesheetConfig>,
    canvas: Canvas,
    mapEditorMode: MapEditorMode,
    project: Project<MapEditorData>
) {
    const theme = useContext(ThemeContext)

    const chunkSelectInnerMeshRef = useRef<Mesh>(null)
    const chunkSelectOuterMeshRef = useRef<BoxHelper>(null)

    const worldMousePosition = useWorldMousePosition(
        {
            spritesheetConfig,
            threeConfig,
            canvas,
            onMouseMove: (mousePosition) => {
                if (!chunkSelectInnerMeshRef.current) return;
                if (!chunkSelectOuterMeshRef.current) return;

                const tileInfo = getTileInfo(spritesheetConfig.current)

                const chunkSizeX = Math.min(project.project_type.mapEditor.size[0], CHUNK_SIZE)
                const chunkSizeY = Math.min(project.project_type.mapEditor.size[1], CHUNK_SIZE)

                const currentChunkXDisplay = Math.floor(mousePosition.x / chunkSizeX) + 1
                const currentChunkYDisplay = Math.floor(mousePosition.y / chunkSizeY) + 1

                const halfOffsetX = tileInfo.width / 2
                const halfOffsetY = tileInfo.height / 2

                const chunkSizePixelsX = tileInfo.width * chunkSizeX
                const chunkSizePixelsY = tileInfo.height * chunkSizeY

                const posX = chunkSizePixelsX * currentChunkXDisplay - chunkSizePixelsX / 2 - halfOffsetX
                const posY = -(chunkSizePixelsY * currentChunkYDisplay - chunkSizePixelsY / 2 + halfOffsetY)

                chunkSelectInnerMeshRef.current.position.set(
                    posX,
                    posY,
                    0
                )

                chunkSelectOuterMeshRef.current.update()
            }
        }
    )

    useEffect(() => {
        function onMouseDown(e: MouseEvent) {
            if (e.button !== 0) return;

            console.log(worldMousePosition)
        }

        switch (mapEditorMode) {
            case MapEditorMode.ChunkSelect:
                canvas.canvasRef.current.addEventListener("mousedown", onMouseDown)
        }

        return () => {
            canvas.canvasRef.current.removeEventListener("mousedown", onMouseDown)
        }
    }, [mapEditorMode, worldMousePosition, project]);

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

                const halfOffsetX = tileInfo.width / 2
                const halfOffsetY = tileInfo.height / 2

                const chunkSizePixelsX = tileInfo.width * chunkSizeX
                const chunkSizePixelsY = tileInfo.height * chunkSizeY

                const posX = chunkSizePixelsX - chunkSizePixelsX / 2 - halfOffsetX
                const posY = -(chunkSizePixelsY - chunkSizePixelsY / 2 + halfOffsetY)

                chunkSelectInnerMesh.position.set(
                    posX,
                    posY,
                    0
                )

                const boxHelper = new BoxHelper(
                    chunkSelectInnerMesh,
                    getColorFromTheme(theme.theme, "selected")
                )
                boxHelper.material.depthTest = false
                boxHelper.material.transparent = true

                chunkSelectInnerMeshRef.current = chunkSelectInnerMesh
                chunkSelectOuterMeshRef.current = boxHelper

                threeConfig.current.scene.add(chunkSelectInnerMesh)
                threeConfig.current.scene.add(boxHelper)

        }

        return () => {
            threeConfig.current.scene.remove(chunkSelectInnerMeshRef.current)
            chunkSelectInnerMeshRef.current = null

            threeConfig.current.scene.remove(chunkSelectOuterMeshRef.current)
            chunkSelectOuterMeshRef.current = null
        }
    }, [mapEditorMode, theme, project]);
}