import {MutableRefObject, useEffect, useRef} from "react";
import {Vector3} from "three";
import {Canvas, ThreeConfig} from "../types/three.js";
import {useMousePosition} from "../../../shared/hooks/useMousePosition.js";

export type UseWorldMousePositionProps = {
    tileWidth: number,
    tileHeight: number,
    threeConfig: MutableRefObject<ThreeConfig>
    canvas: Canvas
    onMouseMove?: (newPosition: Vector3) => void
    onWorldMousePositionChange?: (newPosition: Vector3) => void
}

export function useWorldMousePosition(props: UseWorldMousePositionProps): MutableRefObject<Vector3> {
    const mousePosition = useMousePosition(props.canvas.canvasRef)
    const worldMousePosition = useRef<Vector3>(new Vector3(0, 0, 0))

    useEffect(() => {
        function onMouseMove() {
            const rect = props.threeConfig.current.renderer.domElement.getBoundingClientRect();
            const mouseNormalized = new Vector3();
            mouseNormalized.x = ((mousePosition.current.x - rect.left) / (rect.right - rect.left)) * 2 - 1;
            mouseNormalized.y = -((mousePosition.current.y - rect.top) / (rect.bottom - rect.top)) * 2 + 1;
            mouseNormalized.z = 0

            const offset = new Vector3(0.5, 0.5, 0)

            const newWorldMousePosition = mouseNormalized.unproject(props.threeConfig.current.camera)
                .divide(new Vector3(props.tileWidth, props.tileHeight, 1))
                .add(offset)
                .floor()

            // We need to invert the world mouse position since the cdda map goes from up to down
            // Additionally, we need to remove 1 since the top left tile starts at +1
            newWorldMousePosition.y = -newWorldMousePosition.y - 1

            if (!newWorldMousePosition.equals(worldMousePosition.current)) {
                if (props.onWorldMousePositionChange) props.onWorldMousePositionChange(newWorldMousePosition)
            }

            worldMousePosition.current = newWorldMousePosition
            if (props.onMouseMove) props.onMouseMove(worldMousePosition.current)
        }

        props.canvas.canvasRef.current.addEventListener("mousemove", onMouseMove)

        return () => {
            props.canvas.canvasRef.current.removeEventListener("mousemove", onMouseMove)
        }
    }, [props]);

    return worldMousePosition
}