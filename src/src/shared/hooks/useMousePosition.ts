import {MutableRefObject, useEffect, useRef} from "react";
import {Vector2} from "three";

export function useMousePosition(canvasRef: MutableRefObject<HTMLCanvasElement>): MutableRefObject<Vector2> {
    const mousePositionRef = useRef<Vector2>(new Vector2(0, 0))

    useEffect(() => {
        const mouseMoveListener = (e: MouseEvent) => {
            mousePositionRef.current.x = e.clientX
            mousePositionRef.current.y = e.clientY
        }

        canvasRef.current.addEventListener("mousemove", mouseMoveListener)

        const refBinding = canvasRef.current
        return () => {
            refBinding.removeEventListener("mousemove", mouseMoveListener)
        }
    }, [canvasRef]);

    return mousePositionRef
}