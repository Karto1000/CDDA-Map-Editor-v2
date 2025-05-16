import {Mesh, MeshBasicMaterial, PlaneGeometry, Vector3} from "three";
import {MutableRefObject, useEffect, useRef} from "react";
import {ThreeConfig} from "../types/three.js";
import {TileInfo} from "../../../tauri/types/spritesheet.js";
import {MAX_DEPTH} from "../../sprites/tilesheets.js";
import {getColorFromTheme, Theme} from "../../../shared/hooks/useTheme.js";

export function useMouseCells(
    threeConfig: MutableRefObject<ThreeConfig>,
    tile_info: TileInfo
) {
    const hoveredCellMeshRef = useRef<Mesh>()
    const selectedCellMeshRef = useRef<Mesh>()

    function regenerate(theme: Theme) {
        const hovered = new PlaneGeometry(tile_info.width, tile_info.height)
        const hoveredMaterial = new MeshBasicMaterial({color: getColorFromTheme(theme, "darkBlue")})
        hoveredMaterial.transparent = true
        hoveredMaterial.opacity = 0.5

        const selected = new PlaneGeometry(tile_info.width, tile_info.height)
        const selectedMaterial = new MeshBasicMaterial({color: getColorFromTheme(theme, "selected")})
        selectedMaterial.transparent = true
        selectedMaterial.opacity = 0.5

        const selectedMesh = new Mesh(selected, selectedMaterial)
        selectedMesh.visible = false

        const highlightedMesh = new Mesh(hovered, hoveredMaterial)

        threeConfig.current.scene.remove(selectedCellMeshRef.current)
        threeConfig.current.scene.remove(hoveredCellMeshRef.current)

        selectedCellMeshRef.current = selectedMesh
        threeConfig.current.scene.add(selectedMesh)

        hoveredCellMeshRef.current = highlightedMesh
        threeConfig.current.scene.add(highlightedMesh)
    }

    return {hoveredCellMeshRef, selectedCellMeshRef, regenerate}
}