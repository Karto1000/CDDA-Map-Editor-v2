import React, {RefObject} from "react";

export type MainCanvasProps = {
    canvasRef: RefObject<HTMLCanvasElement>,
    canvasContainerRef: RefObject<HTMLDivElement>,
    displayState: "flex" | "none"
}

export function MainCanvas(props: MainCanvasProps) {
    return (
        <div ref={props.canvasContainerRef}
             style={{width: "100%", height: "100%", display: props.displayState}}>
            {/* This should always be in the dom because then we only have to load the sprites once */}
            <canvas ref={props.canvasRef} className={"main-canvas"} tabIndex={0}/>
        </div>
    )
}