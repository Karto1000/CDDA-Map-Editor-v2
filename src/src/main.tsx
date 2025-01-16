//===================================================================
// Import References 
//===================================================================

import React, {useContext, useEffect, useRef, useState} from "react";
import "./main.scss"
import * as PIXI from "pixi.js";
import {Application, Geometry, Mesh, MeshGeometry, Renderer, Shader, TextureShader, UniformGroup} from "pixi.js";
import {ThemeContext} from "./app.tsx";
import {getColorFromTheme} from "./hooks/useTheme.tsx";
import {Vec2, vec2} from "gl-matrix";

//===================================================================
// Constant Variables Definitions
//===================================================================
enum MouseButton {
    LEFT = 0,
    MIDDLE = 1,
    RIGHT = 2
}

const MIN_ZOOM: number = 5;
const MAX_ZOOM: number = 128;

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
function hexToRgbNorm(hex: string): [number, number, number] {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? [
        parseInt(result[1], 16) / 255,
        parseInt(result[2], 16) / 255,
        parseInt(result[3], 16) / 255,
    ] : null;
}

//===================================================================
// Component Definition
//===================================================================
export default function Main() {
    const appRef = useRef<Application<Renderer>>();
    const mainRef = useRef<HTMLDivElement>();
    const gridQuad = useRef<Mesh<Geometry, Shader>>();
    const scrollStartRef = useRef<{ mouseX: number, mouseY: number, offsetX: number, offsetY: number }>();
    const [isLoaded, setIsLoaded] = useState<boolean>(false);
    const {theme} = useContext(ThemeContext);
    const uniforms = useRef<UniformGroup>(
        new UniformGroup({
            uScreenSize: {value: [window.innerWidth, window.innerHeight], type: "vec2<f32>"},
            uOffset: {value: [0, 0], type: "vec2<f32>"},
            uCellSize: {value: 32, type: "i32"},
            uLightest: {value: hexToRgbNorm(getColorFromTheme(theme, "lightest")), type: "vec3<f32>"},
            uDarker: {value: hexToRgbNorm(getColorFromTheme(theme, "darker")), type: "vec3<f32>"},
        })
    );

    useEffect(() => {
        if (appRef.current) return;
        if (!mainRef.current) return;

        const app = new PIXI.Application();

        const scrollListener = (e: WheelEvent) => {
            // @ts-ignore
            const oldSize: number = uniforms.current.uniforms.uCellSize;
            // @ts-ignore
            uniforms.current.uniforms.uCellSize = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, uniforms.current.uniforms.uCellSize - (e.deltaY / 50)))
            // @ts-ignore
            const newSize: number = uniforms.current.uniforms.uCellSize;

            const mousePos = new Vec2(e.clientX, e.clientY)
            const offset = new Vec2(uniforms.current.uniforms.uOffset[0], uniforms.current.uniforms.uOffset[1]);

            const mousePosAndOffset = Vec2.create();
            Vec2.add(mousePosAndOffset, mousePos, offset)

            const oldPosition = Vec2.create();
            Vec2.div(oldPosition, mousePosAndOffset, new Vec2(oldSize, oldSize));

            const newPosition = Vec2.create();
            Vec2.div(newPosition, mousePosAndOffset, new Vec2(newSize, newSize));

            const newOffset = (newPosition.sub(oldPosition)).mul(new Vec2(newSize, newSize));

            uniforms.current.uniforms.uOffset[0] -= newOffset.x
            uniforms.current.uniforms.uOffset[1] -= newOffset.y
        }

        const resizeListener = (e: Event) => {
            uniforms.current.uniforms.uScreenSize = [window.innerWidth, window.innerHeight]
            gridQuad.current.width = window.innerWidth;
            gridQuad.current.height = window.innerHeight;
        }

        const mouseDownListener = (e: MouseEvent) => {
            if (e.button === MouseButton.MIDDLE) {
                scrollStartRef.current = {
                    mouseX: e.clientX,
                    mouseY: e.clientY,
                    // @ts-ignore
                    offsetX: uniforms.current.uniforms.uOffset[0],
                    // @ts-ignore
                    offsetY: uniforms.current.uniforms.uOffset[1]
                }
            }
        }

        const mouseUpListener = (e: MouseEvent) => {
            if (e.button === MouseButton.MIDDLE) {
                scrollStartRef.current = null
            }
        }

        const mouseMoveListener = (e: MouseEvent) => {
            if (!scrollStartRef.current) return;

            const delta = new vec2(
                e.clientX - scrollStartRef.current.mouseX,
                e.clientY - scrollStartRef.current.mouseY
            )

            uniforms.current.uniforms.uOffset = [
                scrollStartRef.current.offsetX - delta.x,
                scrollStartRef.current.offsetY - delta.y
            ];
        }

        (async () => {
            await app.init({
                resizeTo: mainRef.current,
                preference: "webgl",
            });

            app.canvas.tabIndex = 0
            app.canvas.style.overflow = "auto";
            app.canvas.focus()

            mainRef.current.appendChild(app.canvas)
            appRef.current = app

            app.canvas.addEventListener("wheel", scrollListener);
            app.canvas.addEventListener("mousedown", mouseDownListener);
            app.canvas.addEventListener("mouseup", mouseUpListener);
            app.canvas.addEventListener("mousemove", mouseMoveListener);
            window.addEventListener("resize", resizeListener);

            setIsLoaded(true);
        })()

        return () => {
            app.canvas.removeEventListener("wheel", scrollListener);
            app.canvas.removeEventListener("mousedown", mouseDownListener);
            app.canvas.removeEventListener("mouseup", mouseUpListener);
            app.canvas.removeEventListener("mousemove", mouseMoveListener);
            window.removeEventListener("resize", resizeListener);
            app.destroy();
        }
    }, []);

    useEffect(() => {
        if (!isLoaded) return;

        const initGrid = async () => {
            const gridFragment = await (await fetch(`/shaders/grid.frag`)).text()
            const gridVert = await (await (fetch(`/shaders/grid.vert`))).text()

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

            uniforms.current.uniforms.uLightest = hexToRgbNorm(getColorFromTheme(theme, "lightest"))
            uniforms.current.uniforms.uDarker = hexToRgbNorm(getColorFromTheme(theme, "darker"))

            const uniforms_ = uniforms.current;
            const shader = Shader.from({
                gl: {
                    fragment: gridFragment,
                    vertex: gridVert,
                },
                resources: {
                    uniforms_
                },
            });

            gridQuad.current = new Mesh({
                geometry: quadGeometry,
                shader,
            });

            gridQuad.current.blendMode = "add-npm";

            appRef.current.stage.addChild(gridQuad.current);
        }

        initGrid();

        return () => {
            appRef.current.stage.removeChild(gridQuad.current);
        }
    }, [theme, isLoaded]);

    return (
        <main ref={mainRef}>
        </main>
    )
}

//===================================================================
// Exports 
//===================================================================