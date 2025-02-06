//===================================================================
// Import References 
//===================================================================

import React, {useContext, useEffect, useRef, useState} from "react";
import "./main.scss"
import {Vec2, vec2} from "gl-matrix";
import Stats from "stats.js"
import {
    AmbientLight, GridHelper,
    PerspectiveCamera,
    Scene,
    Vector2,
    WebGLRenderer,
} from "three"
import {TextureAtlas} from "./rendering/texture-atlas.ts";
import {getColorFromTheme} from "./hooks/useTheme.tsx";
import {ThemeContext} from "./app.tsx";
import {invoke} from "@tauri-apps/api/core";
import {TilesetConfig} from "./lib/map_data";
import {degToRad} from "three/src/math/MathUtils";

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
function getCameraZForSpriteSize(canvasWidth: number, spriteSize: number, fov: number) {
    const numSprites = canvasWidth / spriteSize;
    const worldWidth = numSprites * spriteSize;
    const fovRadians = (fov * Math.PI) / 180;
    return worldWidth / (2 * Math.tan(fovRadians / 2));
}

const setupThreeJS = (
    canvas: HTMLCanvasElement,
    mainRef: HTMLDivElement,
    width: number,
    height: number
): {
    stats: Stats,
    scene: Scene,
    perspectiveCamera: PerspectiveCamera,
    renderer: WebGLRenderer
} => {
    const stats = new Stats()
    stats.showPanel(0)
    stats.dom.style.top = "64px"
    stats.dom.style.left = "unset"
    stats.dom.style.right = "2px"

    mainRef.appendChild(stats.dom)

    const scene = new Scene()
    const perspectiveCamera = new PerspectiveCamera(75, width / height, 0.01, 7000)
    perspectiveCamera.position.z = getCameraZForSpriteSize(width, 32, 75);

    const renderer = new WebGLRenderer({canvas, alpha: true})
    renderer.setSize(width, height)

    const ambientLight = new AmbientLight("#FFFFFF", 5)
    scene.add(ambientLight)

    return {
        stats,
        scene,
        perspectiveCamera: perspectiveCamera,
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
    const {theme} = useContext(ThemeContext)

    const canvasRef = useRef<HTMLCanvasElement>();
    const sceneRef = useRef<Scene>();
    const rendererRef = useRef<WebGLRenderer>()
    const perspectiveCameraRef = useRef<PerspectiveCamera>()

    const [isLoaded, setIsLoaded] = useState<boolean>(false)

    // Setup Three.js
    useEffect(() => {
        if (!mainRef.current) return;
        if (!canvasRef.current) return;

        const canvasWidth = mainRef.current.clientWidth;
        const canvasHeight = mainRef.current.clientHeight;

        (async () => {
            const {stats, scene, perspectiveCamera, renderer} = setupThreeJS(
                canvasRef.current,
                mainRef.current,
                canvasWidth,
                canvasHeight
            )

            sceneRef.current = scene
            rendererRef.current = renderer
            perspectiveCameraRef.current = perspectiveCamera

            setIsLoaded(true)

            function run() {
                stats.begin()

                if (isRightPressed) perspectiveCamera.position.x += 10
                if (isUpPressed) perspectiveCamera.position.y += 10
                if (isDownPressed) perspectiveCamera.position.y -= 10
                if (isLeftPressed) perspectiveCamera.position.x -= 10

                perspectiveCamera.updateProjectionMatrix()
                renderer.render(scene, perspectiveCamera)

                stats.end()

                requestAnimationFrame(run)
            }

            run()
        })()

    }, []);

    // Load the tileset
    useEffect(() => {
        (async () => {
            const metadata = await invoke<TilesetConfig>("get_tileset_metadata", {name: "MSX++UnDeadPeopleEdition"})

            const downloadPromises: Promise<ArrayBuffer>[] = []

            for (let tileInfo of metadata["tiles-new"]) {
                console.log(`Loading ${tileInfo.file}`)
                downloadPromises.push(invoke<ArrayBuffer>("download_spritesheet", {
                    tileset: "MSX++UnDeadPeopleEdition",
                    name: tileInfo.file
                }))
            }

            const arrayBuffs = await Promise.all(downloadPromises)

            const atlases = {}
            for (let i = 0; i < arrayBuffs.length; i++) {
                const arrayBuffer = arrayBuffs[i]
                const tileInfo = metadata["tiles-new"][i]

                const blob = new Blob([arrayBuffer], {type: "image/png"});
                const url = URL.createObjectURL(blob)

                atlases[tileInfo.file] = TextureAtlas.loadFromURL(
                    url,
                    {
                        "grass": {
                            name: "grass",
                            position: new Vector2(128, 2624)
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
                )
            }

            const mapSizeY = 24 * 8
            const mapSizeX = 24 * 16

            for (let y = 0; y < mapSizeY; y++) {
                for (let x = 0; x < mapSizeX; x++) {
                    atlases["normal_terrain.png"].setTileAt(new Vector2(x, y), "grass")
                }
            }

            for (let atlasKey of Object.keys(atlases)) {
                const atlas = atlases[atlasKey]

                sceneRef.current.add(atlas.mesh)
            }
        })()
    }, []);

    useEffect(() => {
        if (!isLoaded) return

        rendererRef.current.setClearColor(getColorFromTheme(theme, "darker"))

        const gridHelper = new GridHelper(
            1,
            16 * 8 * 32 * 24 / 32,
            getColorFromTheme(theme, "disabled"), getColorFromTheme(theme, "light")
        )
        gridHelper.scale.x = 16 * 8 * 32 * 24
        gridHelper.scale.z = 16 * 8 * 32 * 24

        gridHelper.position.x -= 16
        gridHelper.position.y -= 16

        gridHelper.rotateX(degToRad(90))
        sceneRef.current.add(gridHelper)
    }, [theme, isLoaded]);

// Set up the listeners
    useEffect(() => {
        const scrollListener = (e: WheelEvent) => {
            const zoomScale = 5
            const newSize: number = Math.min(MIN_ZOOM, Math.max(MAX_ZOOM, perspectiveCameraRef.current.position.z + (perspectiveCameraRef.current.position.z / e.deltaY * zoomScale)));
            perspectiveCameraRef.current.position.z = newSize
        }

        const resizeListener = (e: Event) => {
            const newWidth = mainRef.current.clientWidth
            const newHeight = mainRef.current.clientHeight

            rendererRef.current.setSize(newWidth, newHeight)
            perspectiveCameraRef.current.aspect = newWidth / newHeight
        }

        const mouseDownListener = (e: MouseEvent) => {
            if (e.button === MouseButton.MIDDLE) {

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