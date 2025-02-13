import {
    AmbientLight,
    GridHelper,
    PerspectiveCamera,
    Scene, Vector2,
    WebGLRenderer
} from "three";
import {MutableRefObject, useContext, useEffect, useRef} from "react";
import {ThemeContext} from "../app.tsx";
import Stats from "stats.js";
import {OrbitControls} from "three/examples/jsm/controls/OrbitControls";
import {getColorFromTheme, Theme} from "../hooks/useTheme.tsx";
import {degToRad} from "three/src/math/MathUtils";
import {Atlases} from "./useTileset.ts";
import {Tilesheets} from "../rendering/tilesheets.ts";
import {DragControls} from "three/examples/jsm/controls/DragControls";
import {ArcballControls} from "three/examples/jsm/controls/ArcballControls";

const MIN_ZOOM: number = 7000;
const MAX_ZOOM: number = 5;

function getCameraZForSpriteSize(canvasWidth: number, spriteSize: number, fov: number) {
    const numSprites = canvasWidth / spriteSize;
    const worldWidth = numSprites * spriteSize;
    const fovRadians = (fov * Math.PI) / 180;
    return worldWidth / (2 * Math.tan(fovRadians / 2));
}

type Props = {
    sceneRef: MutableRefObject<Scene>,
    canvasRef: MutableRefObject<HTMLCanvasElement>
    canvasContainerRef: MutableRefObject<HTMLDivElement>
    tilesheetsRef: MutableRefObject<Tilesheets>

    theme: Theme
    isDisplaying: boolean
}

export function useEditor(props: Props): void {
    const rendererRef = useRef<WebGLRenderer>()
    const cameraRef = useRef<PerspectiveCamera>()
    const controlsRef = useRef<ArcballControls>()
    const gridHelperRef = useRef<GridHelper>()
    const ambientLightRef = useRef<AmbientLight>()
    const statsRef = useRef<Stats>()

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

            const perspectiveCamera = new PerspectiveCamera(75, canvasWidth / canvasHeight, 0.01, 7000)
            perspectiveCamera.position.z = getCameraZForSpriteSize(canvasWidth, 32, 75);

            const renderer = new WebGLRenderer({canvas: props.canvasRef.current, alpha: true})
            renderer.setSize(canvasWidth, canvasHeight)

            const controls = new ArcballControls(perspectiveCamera, props.canvasRef.current)
            controls.maxDistance = MIN_ZOOM
            controls.minDistance = MAX_ZOOM
            controls.enableRotate = false
            controls.cursorZoom = true

            const ambientLight = new AmbientLight("#FFFFFF", 5)
            props.sceneRef.current.add(ambientLight)

            statsRef.current = stats
            cameraRef.current = perspectiveCamera
            rendererRef.current = renderer
            controlsRef.current = controls
            ambientLightRef.current = ambientLight
        }

        setup()

        function onResize() {
            const newWidth = props.canvasContainerRef.current.clientWidth
            const newHeight = props.canvasContainerRef.current.clientHeight

            rendererRef.current.setSize(newWidth, newHeight)
            cameraRef.current.aspect = newWidth / newHeight
            cameraRef.current.position.z = getCameraZForSpriteSize(newWidth, 32, 75);
        }

        window.addEventListener("resize", onResize)

        return () => {
            props.sceneRef.current.remove(ambientLightRef.current)
            props.sceneRef.current.remove(gridHelperRef.current)
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
            cameraRef.current.aspect = newWidth / newHeight
            cameraRef.current.position.z = 100
        }

        initialValueUpdate();

        const indices = []
        const positions = []
        for (let y = 0; y < 400; y++) {
            for (let x = 0; x < 400; x++) {
                indices.push(y * 400 + x)
                positions.push(new Vector2(x * 32, y * 32))
            }
        }

        props.tilesheetsRef.current.drawSpritesBatched(indices, positions)

        let handler: number;

        function loop() {
            statsRef.current.begin()

            cameraRef.current.updateProjectionMatrix()

            controlsRef.current.update()
            rendererRef.current.render(props.sceneRef.current, cameraRef.current)

            statsRef.current.end()

            handler = requestAnimationFrame(loop)
        }

        loop()

        return () => {
            cancelAnimationFrame(handler)
        }
    }, [props.tilesheetsRef, props.canvasContainerRef, props.isDisplaying, props.sceneRef]);

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
}