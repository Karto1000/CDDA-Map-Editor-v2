import {Mesh, MeshBasicMaterial, PlaneGeometry} from "three";
import {RefObject, useEffect, useRef, useState} from "react";
import {ThreeConfig} from "../types/three.js";
import {SpritesheetConfig} from "../../../tauri/types/spritesheet.js";
import {getColorFromTheme, Theme} from "../../../shared/hooks/useTheme.js";
import {logRender} from "../../../shared/utils/log.js";

export function useMouseCells(
    threeConfig: RefObject<ThreeConfig>,
    spritesheetConfig: RefObject<SpritesheetConfig>,
    theme: Theme,
): {
    hoveredCellMeshRef: RefObject<Mesh>,
    selectedCellMeshRef: RefObject<Mesh>,
    updateCellSize: () => void
} {
    const hoveredCellMeshRef = useRef<Mesh>(null)
    const selectedCellMeshRef = useRef<Mesh>(null)
    const [updateCellMeshes, setUpdateCellMeshes] = useState<boolean>(false)

    useEffect(() => {
        const tileInfo = spritesheetConfig.current.tile_info[0]

        const hovered = new PlaneGeometry(tileInfo.width, tileInfo.height)
        const hoveredMaterial = new MeshBasicMaterial({color: getColorFromTheme(theme, "darkBlue")})
        hoveredMaterial.transparent = true
        hoveredMaterial.opacity = 0.5

        const selected = new PlaneGeometry(tileInfo.width, tileInfo.height)
        const selectedMaterial = new MeshBasicMaterial({color: getColorFromTheme(theme, "selected")})
        selectedMaterial.transparent = true
        selectedMaterial.opacity = 0.5

        const selectedMesh = new Mesh(selected, selectedMaterial)
        selectedMesh.visible = false

        const highlightedMesh = new Mesh(hovered, hoveredMaterial)

        selectedCellMeshRef.current = selectedMesh
        threeConfig.current.scene.add(selectedMesh)

        hoveredCellMeshRef.current = highlightedMesh
        threeConfig.current.scene.add(highlightedMesh)

        return () => {
            threeConfig.current.scene.remove(selectedCellMeshRef.current)
            threeConfig.current.scene.remove(hoveredCellMeshRef.current)

            selectedCellMeshRef.current.geometry.dispose()
            hoveredCellMeshRef.current.geometry.dispose()

            selectedCellMeshRef.current = null
            hoveredCellMeshRef.current = null
        }
    }, [theme, updateCellMeshes]);

    function updateCellSize() {
        logRender("Updating cell size")
        setUpdateCellMeshes((v) => !v)
    }

    return {hoveredCellMeshRef, selectedCellMeshRef, updateCellSize}
}