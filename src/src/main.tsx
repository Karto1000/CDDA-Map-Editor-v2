//===================================================================
// Import References 
//===================================================================

import React, {useContext, useEffect, useRef, useState} from "react";
import "./main.scss"
import Stats from "stats.js"
import {
    AmbientLight, GridHelper,
    PerspectiveCamera, Plane, Raycaster,
    Scene,
    Vector2, Vector3,
    WebGLRenderer,
} from "three"
import {TextureAtlas} from "./rendering/texture-atlas.ts";
import {getColorFromTheme} from "./hooks/useTheme.tsx";
import {ThemeContext} from "./app.tsx";
import {invoke} from "@tauri-apps/api/core";
import {degToRad} from "three/src/math/MathUtils";
import {listen} from "@tauri-apps/api/event";
import {PlaceTerrainEvent, TilesetConfig} from "./lib/map_data/recv";
import {PlaceCommand} from "./lib/map_data/send";
import {serializedVec2ToVector2} from "./lib";
import {OrbitControls} from "three/examples/jsm/controls/OrbitControls";

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

let isLeftMousePressed = false
let mousePosition = new Vector2()
const atlases: { [file: string]: TextureAtlas } = {}

//===================================================================
// Component Definition
//===================================================================
export default function Main() {
    const {theme} = useContext(ThemeContext)

    const mainRef = useRef<HTMLDivElement>();
    const canvasRef = useRef<HTMLCanvasElement>();
    const sceneRef = useRef<Scene>();
    const rendererRef = useRef<WebGLRenderer>()
    const perspectiveCameraRef = useRef<PerspectiveCamera>()
    const events = useRef<PlaceTerrainEvent[]>([])
    const gridHelperRef = useRef<GridHelper>()
    const controlsRef = useRef<OrbitControls>()

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

            const controls = new OrbitControls(perspectiveCamera, canvasRef.current)
            controls.maxDistance = MIN_ZOOM
            controls.minDistance = MAX_ZOOM
            controls.enableRotate = false
            controls.enableDamping = true
            controls.zoomToCursor = true

            sceneRef.current = scene
            rendererRef.current = renderer
            perspectiveCameraRef.current = perspectiveCamera
            controlsRef.current = controls

            setIsLoaded(true)

            function run() {
                stats.begin()

                perspectiveCamera.updateProjectionMatrix()

                let currentEvent = events.current.pop()
                while (currentEvent !== undefined) {
                    atlases["normal_terrain.png"].setTileAt(serializedVec2ToVector2(currentEvent.position), currentEvent.identifier)
                    currentEvent = events.current.pop()
                }

                if (isLeftMousePressed) {
                    const mouseNormalized = new Vector2();
                    const rect = renderer.domElement.getBoundingClientRect();
                    // ABSOLUTE LEGEND #2 -> https://discourse.threejs.org/t/custom-canvas-size-with-orbitcontrols-and-raycaster/18742/2
                    mouseNormalized.x = ((mousePosition.x - rect.left) / (rect.right - rect.left)) * 2 - 1;
                    mouseNormalized.y = -((mousePosition.y - rect.top) / (rect.bottom - rect.top)) * 2 + 1;

                    const raycaster = new Raycaster()
                    raycaster.setFromCamera(mouseNormalized.clone(), perspectiveCamera);
                    const planeZ = new Plane(new Vector3(0, 0, 1), 0);

                    const intersection = new Vector3();
                    const intersected = raycaster.ray.intersectPlane(planeZ, intersection);


                    const worldCellX = Math.round(intersected.x / 32)
                    const worldCellY = Math.round(intersected.y / 32)

                    if (worldCellX >= 0 && worldCellY >= 0) {
                        invoke<PlaceCommand>(
                            "place",
                            {
                                command: {
                                    position: `${worldCellX},${worldCellY}`,
                                    character: "g"
                                }
                            }
                        )
                    }
                }

                controls.update()
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

            for (let i = 0; i < arrayBuffs.length; i++) {
                const arrayBuffer = arrayBuffs[i]
                const tileInfo = metadata["tiles-new"][i]

                const blob = new Blob([arrayBuffer], {type: "image/png"});
                const url = URL.createObjectURL(blob)

                atlases[tileInfo.file] = TextureAtlas.loadFromURL(
                    url,
                    {
                        "t_grass": {
                            name: "t_grass",
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

            for (let atlasKey of Object.keys(atlases)) {
                const atlas = atlases[atlasKey]
                sceneRef.current.add(atlas.mesh)
            }

            await listen<PlaceTerrainEvent>(
                "place_terrain",
                e => {
                    const position = serializedVec2ToVector2(e.payload.position)
                    console.log(`Set ${e.payload.identifier} to position ${position.x};${position.y}`)
                    events.current.push(e.payload)
                }
            )

            // await invoke<PlaceCommand>(
            //     "place",
            //     {command: {position: `${x},${y}`, character: "g"}}
            // )

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
        gridHelperRef.current = gridHelper
    }, [theme, isLoaded]);

    // Set up the listeners
    useEffect(() => {
        const resizeListener = (e: Event) => {
            const newWidth = mainRef.current.clientWidth
            const newHeight = mainRef.current.clientHeight

            rendererRef.current.setSize(newWidth, newHeight)
            perspectiveCameraRef.current.aspect = newWidth / newHeight
        }

        const mouseDownListener = async (e: MouseEvent) => {
            if (e.button === MouseButton.LEFT) {
                isLeftMousePressed = true
            }
        }

        const mouseUpListener = (e: MouseEvent) => {
            if (e.button === MouseButton.LEFT) {
                isLeftMousePressed = false
            }
        }

        const mouseMoveListener = (e: MouseEvent) => {
            mousePosition.x = e.clientX
            mousePosition.y = e.clientY
        }

        canvasRef.current.addEventListener("mousedown", mouseDownListener);
        canvasRef.current.addEventListener("mouseup", mouseUpListener);
        canvasRef.current.addEventListener("mousemove", mouseMoveListener);
        window.addEventListener("resize", resizeListener);

        const cpyRef = canvasRef.current
        return () => {
            cpyRef.removeEventListener("mousedown", mouseDownListener);
            cpyRef.removeEventListener("mouseup", mouseUpListener);
            cpyRef.removeEventListener("mousemove", mouseMoveListener);
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