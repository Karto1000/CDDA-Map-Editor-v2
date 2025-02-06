//===================================================================
// Import References 
//===================================================================

import React, {useContext, useEffect, useRef, useState} from "react";
import "./main.scss"
import {Vec2, vec2} from "gl-matrix";
import Stats from "stats.js"
import {
    AmbientLight,
    Camera, Mesh,
    MeshBasicMaterial,
    PerspectiveCamera,
    PlaneGeometry,
    Scene,
    Vector2,
    WebGLRenderer
} from "three"
import {TextureAtlas} from "./rendering/texture-atlas.ts";
import {getColorFromTheme, useTheme} from "./hooks/useTheme.tsx";
import {ThemeContext} from "./app.tsx";
import {invoke} from "@tauri-apps/api/core";
import {TilesetConfig} from "./lib/map_data";

//===================================================================
// Constant Variables Definitions
//===================================================================
enum MouseButton {
    LEFT = 0,
    MIDDLE = 1,
    RIGHT = 2
}

const MIN_ZOOM: number = 7000;
const MAX_ZOOM: number = 5;

//===================================================================
// Export Type Definitions
//===================================================================

//===================================================================
// Local Type Definitions
//===================================================================
type Grid = {
    screenSize: Vec2,
    offset: Vec2,
    cellSize: number,
}

type ScrollStart = {
    mouseX: number,
    mouseY: number,
    offsetX: number,
    offsetY: number,
}

//===================================================================
// Class Definitions
//===================================================================

//===================================================================
// Function Definitions
//===================================================================
const setupThreeJS = (
    canvas: HTMLCanvasElement,
    mainRef: HTMLDivElement,
    width: number,
    height: number
): {stats: Stats, scene: Scene, camera: PerspectiveCamera, renderer: WebGLRenderer} => {
    const stats = new Stats()
    stats.showPanel(0)
    stats.dom.style.top = "32px"
    stats.dom.style.left = "unset"
    stats.dom.style.right = "2px"

    mainRef.appendChild(stats.dom)

    const scene = new Scene()
    const camera = new PerspectiveCamera(75, width / height, 0.01, 7000)
    camera.position.z = 100;

    const renderer = new WebGLRenderer({canvas, alpha: true})
    renderer.setSize(width, height)

    const ambientLight = new AmbientLight("#FFFFFF", 5)
    scene.add(ambientLight)

    return {
        stats,
        scene,
        camera,
        renderer
    }
}

let isRightPressed = false
let isUpPressed = false
let isDownPressed = false
let isLeftPressed = false

//===================================================================
// Component Definition
//===================================================================
export default function Main() {
    const mainRef = useRef<HTMLDivElement>();
    const scrollStartRef = useRef<ScrollStart>();
    const gridData = useRef<Grid>({
        cellSize: 32.,
        offset: Vec2.create(),
        screenSize: new Vec2(window.innerWidth, window.innerHeight),
    })
    const {theme} = useContext(ThemeContext)

    const canvasRef = useRef<HTMLCanvasElement>();
    const sceneRef = useRef<Scene>();
    const rendererRef = useRef<WebGLRenderer>()
    const cameraRef = useRef<PerspectiveCamera>()

    // Setup Threejs
    useEffect(() => {
        if (!mainRef.current) return;
        if (!canvasRef.current) return;

        const {stats, scene, camera, renderer} = setupThreeJS(
            canvasRef.current,
            mainRef.current,
            mainRef.current.clientWidth,
            mainRef.current.clientHeight
        )

        // const terrainAtlas = TextureAtlas.loadFromURL(
        //     "normal_terrain.png",
        //     {
        //         "manhole": {
        //             name: "manhole",
        //             position: new Vector2(32, 9408)
        //         },
        //         "manhole2": {
        //             name: "manhole2",
        //             position: new Vector2(64, 9408)
        //         },
        //     },
        //     {
        //         atlasWidth: 512,
        //         atlasHeight: 9472,
        //         tileWidth: 32,
        //         tileHeight: 32,
        //         maxInstances: 73728,
        //         yLayer: 0
        //     }
        // )

        rendererRef.current = renderer
        cameraRef.current = camera
        sceneRef.current = scene

        function run() {
            stats.begin()

            if (isRightPressed) camera.position.x += 1
            if (isUpPressed) camera.position.y += 1
            if (isDownPressed) camera.position.y -= 1
            if (isLeftPressed) camera.position.x -= 1

            camera.updateProjectionMatrix()
            renderer.render(scene, camera)

            stats.end()

            requestAnimationFrame(run)
        }

        run()
    }, []);

    // Load the tileset
    useEffect(() => {
        (async () => {
            const metadata = await invoke<TilesetConfig>("get_tileset_metadata", {name: "MSX++UnDeadPeopleEdition"})

            const downloadPromises: Promise<ArrayBuffer>[] = []

            for (let tileInfo of metadata["tiles-new"]) {
                console.log(`Loading ${tileInfo.file}`)
                downloadPromises.push(invoke<ArrayBuffer>("download_spritesheet", {tileset: "MSX++UnDeadPeopleEdition", name: tileInfo.file}))
            }

            const arrayBuffs = await Promise.all(downloadPromises)

            const atlases = []
            for (let i = 0; i < arrayBuffs.length; i++) {
                const arrayBuffer = arrayBuffs[i]
                const tileInfo = metadata["tiles-new"][i]

                const blob  = new Blob([arrayBuffer], { type: "image/png" });
                const url = URL.createObjectURL(blob)

                atlases.push(TextureAtlas.loadFromURL(
                    url,
                    {
                        "manhole": {
                            name: "manhole",
                            position: new Vector2(32, 9408)
                        },
                        "manhole2": {
                            name: "manhole2",
                            position: new Vector2(64, 9408)
                        },
                    },
                    {
                        atlasWidth: tileInfo.spritesheet_dimensions[0],
                        atlasHeight: tileInfo.spritesheet_dimensions[1],
                        tileWidth: tileInfo.sprite_width,
                        tileHeight: tileInfo.sprite_height,
                        maxInstances: 73728,
                        yLayer: 0
                    }
                ))
            }

            const mapSizeY = 24 * 8
            const mapSizeX = 24 * 16

            for (let y = 0; y < mapSizeY; y++) {
                for (let x = 0; x < mapSizeX; x++) {
                    atlases[0].setTileAt(new Vector2(x, y), "manhole")
                }
            }

            for (let atlas of atlases) {
                sceneRef.current.add(atlas.mesh)
            }
        })()
    }, []);

    useEffect(() => {
        rendererRef.current.setClearColor(getColorFromTheme(theme, "darker"))
    }, [theme]);

    // Set up the listeners
    useEffect(() => {
        const scrollListener = (e: WheelEvent) => {
            const zoomScale = 5
            const newSize: number = Math.min(MIN_ZOOM, Math.max(MAX_ZOOM, cameraRef.current.position.z + (cameraRef.current.position.z / e.deltaY * zoomScale)));

            // const mousePos = new Vec2(e.clientX, e.clientY)
            //
            // const mousePosAndOffset = Vec2.create();
            // Vec2.add(mousePosAndOffset, mousePos, gridData.current.offset)
            //
            // const oldPosition = Vec2.create();
            // Vec2.div(oldPosition, mousePosAndOffset, new Vec2(oldSize, oldSize));
            //
            // const newPosition = Vec2.create();
            // Vec2.div(newPosition, mousePosAndOffset, new Vec2(newSize, newSize));
            //
            // const newOffset = (newPosition.sub(oldPosition)).mul(new Vec2(newSize, newSize));
            //
            // gridData.current.offset[0] -= newOffset.x
            // gridData.current.offset[1] -= newOffset.y
            // gridData.current.cellSize = newSize;

            cameraRef.current.position.z = newSize
        }

        const resizeListener = (e: Event) => {
            gridData.current.screenSize = new Vec2(window.innerWidth, window.innerHeight)
            rendererRef.current.setSize(window.innerWidth, window.innerHeight)
            cameraRef.current.aspect = window.innerWidth / window.innerHeight
        }

        const mouseDownListener = (e: MouseEvent) => {
            if (e.button === MouseButton.MIDDLE) {
                scrollStartRef.current = {
                    mouseX: e.clientX,
                    mouseY: e.clientY,
                    offsetX: gridData.current.offset.x,
                    offsetY: gridData.current.offset.y
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

            gridData.current.offset = new Vec2(
                scrollStartRef.current.offsetX - delta.x,
                scrollStartRef.current.offsetY - delta.y
            );
        }

        const keyDownListener = (e: KeyboardEvent) => {
            if (e.key === "ArrowRight") isRightPressed = true
            if (e.key === "ArrowUp") isUpPressed = true
            if (e.key === "ArrowDown") isDownPressed = true
            if (e.key === "ArrowLeft") isLeftPressed = true
        }

        const keyUpListener = (e: KeyboardEvent) => {
            if (e.key === "ArrowRight") isRightPressed = false
            if (e.key === "ArrowUp") isUpPressed = false
            if (e.key === "ArrowDown") isDownPressed = false
            if (e.key === "ArrowLeft") isLeftPressed = false
        }

        canvasRef.current.addEventListener("wheel", scrollListener);
        canvasRef.current.addEventListener("mousedown", mouseDownListener);
        canvasRef.current.addEventListener("mouseup", mouseUpListener);
        canvasRef.current.addEventListener("mousemove", mouseMoveListener);
        canvasRef.current.addEventListener("keydown", keyDownListener)
        canvasRef.current.addEventListener("keyup", keyUpListener)
        window.addEventListener("resize", resizeListener);

        const cpyRef = canvasRef.current
        return () => {
            cpyRef.removeEventListener("wheel", scrollListener);
            cpyRef.removeEventListener("mousedown", mouseDownListener);
            cpyRef.removeEventListener("mouseup", mouseUpListener);
            cpyRef.removeEventListener("mousemove", mouseMoveListener);
            cpyRef.removeEventListener("keydown", keyDownListener)
            cpyRef.removeEventListener("keyup", keyUpListener)
            window.removeEventListener("resize", resizeListener);
        }
    }, []);

    return (
        <main ref={mainRef}>
            <canvas ref={canvasRef} tabIndex={1}/>
        </main>
    )
}

//===================================================================
// Exports 
//===================================================================