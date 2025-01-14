//===================================================================
// Import References 
//===================================================================

import React, {useEffect, useRef} from "react";
import "./main.scss"
import * as PIXI from "pixi.js";
import {Geometry, Mesh, Shader, UniformGroup} from "pixi.js";

//===================================================================
// Constant Variables Definitions
//===================================================================

//===================================================================
// Export Type Definitions
//===================================================================

//===================================================================
// Local Type Definitions
//===================================================================

//===================================================================
// Class Definitions
//===================================================================

//===================================================================
// Function Definitions
//===================================================================


//===================================================================
// Component Definition
//===================================================================
export default function Main() {
    const canvas = useRef<HTMLCanvasElement>();
    const mainRef = useRef<HTMLDivElement>();

    useEffect(() => {
        const initPixi = async () => {
            console.log(process.env.PUBLIC_URL)
            const gridFragment = await (await fetch(`/shaders/grid.frag`)).text()
            const gridVert = await (await (fetch(`/shaders/grid.vert`))).text()
            console.log(gridVert)

            const app = new PIXI.Application();
            await app.init({
                resizeTo: window,
                preference: "webgl",
            });

            mainRef.current.appendChild(app.canvas);

            const quadGeometry = new Geometry({
                attributes: {
                    aPosition: [
                        0,
                        0,
                        window.innerWidth,
                        0,
                        window.innerWidth,
                        window.innerHeight,
                        0,
                        window.innerHeight,
                    ],
                    aUV: [0, 0, 1, 0, 1, 1, 0, 1],
                },
                indexBuffer: [0, 1, 2, 0, 2, 3],
            });

            const uniform = new UniformGroup({
                uScreenSize: {value: {x: window.innerWidth, y: window.innerHeight}, type: "vec2<f32>"},
                uOffset: {value: {x: 0, y: 0}, type: "vec2<f32>"},
            })

            const shader = Shader.from({
                gl: {
                    fragment: gridFragment,
                    vertex: gridVert,
                },
                resources: {
                    uniform
                },
            });

            const quad = new Mesh({
                geometry: quadGeometry,
                shader,
            });

            quad.blendMode = "add-npm";

            app.stage.addChild(quad)

            app.ticker.add(() => {
                uniform.uniforms.uScreenSize = {x: window.innerWidth, y: window.innerHeight};
                uniform.uniforms.uOffset = {x: uniform.uniforms.uOffset.x + 0.5, y: 0};

                quad.width = window.innerWidth;
                quad.height = window.innerHeight;
            })
        }

        initPixi()
    }, []);

    return (
        <main ref={mainRef}>

        </main>
    )
}

//===================================================================
// Exports 
//===================================================================