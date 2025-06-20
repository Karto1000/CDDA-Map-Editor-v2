import {RefObject, useEffect, useState} from "react";
import {Vector3} from "three";
import {Canvas, ThreeConfig} from "../types/three.js";
import {useMousePosition} from "../../../shared/hooks/useMousePosition.js";
import {SpritesheetConfig} from "../../../tauri/types/spritesheet.js";

export type UseWorldMousePositionProps = {
    spritesheetConfig: RefObject<SpritesheetConfig>
    threeConfig: RefObject<ThreeConfig>
    canvas: Canvas
    onMouseMove?: (newPosition: Vector3) => void
    onWorldMousePositionChange?: (newPosition: Vector3) => void
}

export function useWorldMousePosition(props: UseWorldMousePositionProps): Vector3 {
    const mousePosition = useMousePosition(props.canvas.canvasRef)
    const [worldMousePosition, setWorldMousePosition] = useState<Vector3>(new Vector3(0, 0, 0))

    useEffect(() => {
        function onMouseMove() {
            const tileInfo = props.spritesheetConfig.current.tile_info[0]

            const rect = props.threeConfig.current.renderer.domElement.getBoundingClientRect();
            const mouseNormalized = new Vector3();
            mouseNormalized.x = ((mousePosition.current.x - rect.left) / (rect.right - rect.left)) * 2 - 1;
            mouseNormalized.y = -((mousePosition.current.y - rect.top) / (rect.bottom - rect.top)) * 2 + 1;
            mouseNormalized.z = 0

            const offset = new Vector3(0.5, 0.5, 0)

            const newWorldMousePosition = mouseNormalized.unproject(props.threeConfig.current.camera)
                .divide(new Vector3(tileInfo.width, tileInfo.height, 1))
                .add(offset)
                .floor()

            // We need to invert the world mouse position since the cdda map goes from up to down
            // Additionally, we need to remove 1 since the top left tile starts at +1
            newWorldMousePosition.y = -newWorldMousePosition.y - 1

            if (!newWorldMousePosition.equals(worldMousePosition)) {
                if (props.onWorldMousePositionChange) props.onWorldMousePositionChange(newWorldMousePosition)
            }

            setWorldMousePosition(newWorldMousePosition)
            if (props.onMouseMove) props.onMouseMove(newWorldMousePosition)
        }

        props.canvas.canvasRef.current.addEventListener("mousemove", onMouseMove)

        return () => {
            props.canvas.canvasRef.current.removeEventListener("mousemove", onMouseMove)
        }
    }, [props.canvas, mousePosition, worldMousePosition, props.onMouseMove, props.onWorldMousePositionChange]);

    return worldMousePosition
}