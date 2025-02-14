import {
    AmbientLight,
    GridHelper,
    OrthographicCamera,
    Plane,
    Raycaster,
    Scene,
    Vector2,
    Vector3,
    WebGLRenderer
} from "three";
import {MutableRefObject, useEffect, useRef} from "react";
import Stats from "stats.js";
import {getColorFromTheme, Theme} from "../hooks/useTheme.tsx";
import {degToRad} from "three/src/math/MathUtils";
import {Tilesheets} from "../rendering/tilesheets.ts";
import {ArcballControls} from "three/examples/jsm/controls/ArcballControls";
import {useMousePosition} from "./useMousePosition.ts";
import {invokeTauri, makeCancelable, serializedVec2ToVector2} from "../lib";
import {MapDataSendCommand, PlaceCommand} from "../lib/map_data/send";
import {listen} from "@tauri-apps/api/event";
import {MapDataEvent} from "../lib/map_data/recv";
import {SpriteLayer} from "../rendering/tilesheet.ts";

const MIN_ZOOM: number = 500;
const MAX_ZOOM: number = 0.05;

function getCameraZForSpriteSize(canvasWidth: number, spriteSize: number, fov: number) {
    const numSprites = canvasWidth / spriteSize;
    const worldWidth = numSprites * spriteSize;
    const fovRadians = (fov * Math.PI) / 180;
    return worldWidth / (2 * Math.tan(fovRadians / 2));
}

function getRandomInt(max) {
    return Math.floor(Math.random() * max);
}


type Props = {
    sceneRef: MutableRefObject<Scene>,
    canvasRef: MutableRefObject<HTMLCanvasElement>
    canvasContainerRef: MutableRefObject<HTMLDivElement>
    tilesheetsRef: MutableRefObject<Tilesheets>

    theme: Theme
    isDisplaying: boolean
    isTilesheetLoaded: boolean
}

export function useEditor(props: Props): void {
    const rendererRef = useRef<WebGLRenderer>()
    const cameraRef = useRef<OrthographicCamera>()
    const controlsRef = useRef<ArcballControls>()
    const gridHelperRef = useRef<GridHelper>()
    const ambientLightRef = useRef<AmbientLight>()
    const statsRef = useRef<Stats>()

    const isLeftMousePressedRef = useRef<boolean>(false)
    const mousePosition = useMousePosition(props.canvasRef)

    // Should only run once on application startup
    useEffect(() => {
        function setup() {
            const stats = new Stats()
            stats.showPanel(0)
            stats.dom.style.top = "64px"
            stats.dom.style.left = "unset"
            stats.dom.style.right = "2px"
            props.canvasContainerRef.current.appendChild(stats.dom)

            const canvasWidth = props.canvasContainerRef.current.clientWidth
            const canvasHeight = props.canvasContainerRef.current.clientHeight

            const camera = new OrthographicCamera(
                canvasWidth / -2,
                canvasWidth / 2,
                canvasHeight / 2,
                canvasHeight / -2,
                0.01,
                999999
            )
            camera.position.z = 999999

            const renderer = new WebGLRenderer({canvas: props.canvasRef.current, alpha: true})
            renderer.setSize(canvasWidth, canvasHeight)

            const controls = new ArcballControls(camera, props.canvasRef.current)
            controls.maxZoom = MIN_ZOOM
            controls.minZoom = MAX_ZOOM
            controls.enableRotate = false
            controls.cursorZoom = true

            const ambientLight = new AmbientLight("#FFFFFF", 5)
            props.sceneRef.current.add(ambientLight)

            statsRef.current = stats
            cameraRef.current = camera
            rendererRef.current = renderer
            controlsRef.current = controls
            ambientLightRef.current = ambientLight
        }

        setup()

        function onResize() {
            const newWidth = props.canvasContainerRef.current.clientWidth
            const newHeight = props.canvasContainerRef.current.clientHeight

            rendererRef.current.setSize(newWidth, newHeight)
            cameraRef.current.position.z = 999999
            cameraRef.current.left = newWidth / -2
            cameraRef.current.right = newWidth / 2
            cameraRef.current.top = newHeight / 2
            cameraRef.current.bottom = newHeight / -2
        }

        const mouseDownListener = async (e: MouseEvent) => {
            if (e.button === 0) {
                isLeftMousePressedRef.current = true
            }
        }

        const mouseUpListener = (e: MouseEvent) => {
            if (e.button === 0) {
                isLeftMousePressedRef.current = false
            }
        }

        window.addEventListener("resize", onResize)
        props.canvasRef.current.addEventListener("mousedown", mouseDownListener)
        props.canvasRef.current.addEventListener("mouseup", mouseUpListener)

        return () => {
            props.sceneRef.current.remove(ambientLightRef.current)
            props.sceneRef.current.remove(gridHelperRef.current)
            props.canvasRef.current.removeEventListener("mousedown", mouseDownListener)
            props.canvasRef.current.removeEventListener("mouseup", mouseUpListener)
            window.removeEventListener("resize", onResize)
        }
    }, [props.canvasContainerRef, props.canvasRef, props.sceneRef]);

    // Should run when the MapEditor is opened
    useEffect(() => {
        if (!props.isDisplaying) return

        // Because the canvas' parent display value was just set to 'unset'
        // The width and height of the canvas is wrong. This is why we're updating it here
        function initialValueUpdate() {
            const newWidth = props.canvasContainerRef.current.clientWidth
            const newHeight = props.canvasContainerRef.current.clientHeight

            rendererRef.current.setSize(newWidth, newHeight)
            cameraRef.current.left = newWidth / -2
            cameraRef.current.right = newWidth / 2
            cameraRef.current.top = newHeight / 2
            cameraRef.current.bottom = newHeight / -2
            cameraRef.current.position.z = 999999
        }

        function handleEvents() {
            if (isLeftMousePressedRef.current) {
                const mouseNormalized = new Vector2();
                const rect = rendererRef.current.domElement.getBoundingClientRect();
                // ABSOLUTE LEGEND #2 -> https://discourse.threejs.org/t/custom-canvas-size-with-orbitcontrols-and-raycaster/18742/2
                mouseNormalized.x = ((mousePosition.current.x - rect.left) / (rect.right - rect.left)) * 2 - 1;
                mouseNormalized.y = -((mousePosition.current.y - rect.top) / (rect.bottom - rect.top)) * 2 + 1;

                const raycaster = new Raycaster()
                raycaster.setFromCamera(mouseNormalized.clone(), cameraRef.current);
                const planeZ = new Plane(new Vector3(0, 0, 1), 0);

                const intersection = new Vector3();
                const intersected = raycaster.ray.intersectPlane(planeZ, intersection);

                const worldCellX = Math.round(intersected.x / 32)
                const worldCellY = Math.round(intersected.y / 32)

                if (worldCellX >= 0 && worldCellY >= 0) {
                    const args = {
                        command: {
                            position: `${worldCellX},${worldCellY}`,
                            character: "g"
                        }
                    }

                    invokeTauri<PlaceCommand, unknown>(MapDataSendCommand.Place, args).then()
                }
            }
        }

        initialValueUpdate();

        let handler: number;

        function loop() {
            statsRef.current.begin()

            cameraRef.current.updateProjectionMatrix()

            handleEvents()
            controlsRef.current.update()
            rendererRef.current.render(props.sceneRef.current, cameraRef.current)

            statsRef.current.end()

            handler = requestAnimationFrame(loop)
        }

        loop()

        return () => {
            cancelAnimationFrame(handler)
        }
    }, [props.tilesheetsRef, props.canvasContainerRef, props.isDisplaying, props.sceneRef, mousePosition]);

    // Should run when the theme changes to change colors
    useEffect(() => {
        rendererRef.current.setClearColor(getColorFromTheme(props.theme, "darker"))

        const gridHelper = new GridHelper(
            1,
            16 * 8 * 32 * 24 / 32,
            getColorFromTheme(props.theme, "disabled"), getColorFromTheme(props.theme, "light")
        )
        gridHelper.scale.x = 16 * 8 * 32 * 24
        gridHelper.scale.z = 16 * 8 * 32 * 24

        gridHelper.position.x -= 16
        gridHelper.position.y -= 16

        gridHelper.rotateX(degToRad(90))
        props.sceneRef.current.add(gridHelper)
        gridHelperRef.current = gridHelper
    }, [props.sceneRef, props.theme]);

    // Should run when the tilesheet has finished loading
    useEffect(() => {
        if (!props.isTilesheetLoaded) return;

        let unlistenFn = makeCancelable(listen<PlaceCommand>(MapDataEvent.PlaceTerrain, d => {
            const vec2 = serializedVec2ToVector2(d.payload.position)
            vec2.x *= 32;
            vec2.y *= 32;

            props.tilesheetsRef.current.removeSprite(vec2, SpriteLayer.Bg)
            props.tilesheetsRef.current.removeSprite(vec2, SpriteLayer.Fg)

            props.tilesheetsRef.current.drawSprite(22366, vec2, SpriteLayer.Fg, 0)
        }))

        let positions = []
        let indices = []
        let layers = []
        for (let y = 0; y < 10; y++) {
            for (let x = 0; x < 10; x++) {
                positions.push(new Vector2(x * 32, y * 32))
                indices.push(17066)
                layers.push(SpriteLayer.Bg)
            }
        }

        props.tilesheetsRef.current.drawSpritesBatched(indices, positions, layers, 0)

        positions = []
        indices = []
        layers = []
        for (let y = 0; y < 10; y++) {
            for (let x = 0; x < 10; x++) {
                positions.push(new Vector2(x * 32, y * 32))
                indices.push(15177)
                layers.push(SpriteLayer.Fg)
            }
        }

        props.tilesheetsRef.current.drawSpritesBatched(indices, positions, layers, 1)

        return () => {
            unlistenFn.cancel()
        }
    }, [props.isTilesheetLoaded, props.tilesheetsRef]);
}