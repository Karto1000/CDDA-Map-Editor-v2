import {RefObject, useEffect, useRef} from "react";
import Stats from "stats.js";
import {
    AmbientLight,
    BufferGeometry,
    LineBasicMaterial,
    LineSegments,
    Object3D,
    OrthographicCamera,
    Raycaster,
    Scene,
    Vector3,
    WebGLRenderer
} from "three";
import {ThreeConfig} from "../types/three.js";
import {OrbitControls} from "three/examples/jsm/controls/OrbitControls.js";
import {getColorFromTheme, Theme} from "../../../shared/hooks/useTheme.js";

const MIN_ZOOM: number = 500;
const MAX_ZOOM: number = 0.05;
export const SHOW_STATS: boolean = false;

export type UseThreeSetupRet = {
    threeConfigRef: RefObject<ThreeConfig>
    onResize: () => void
}

export type GridConfig = {
    height: number,
    width: number,
    linesHeight: number,
    linesWidth: number,
    color: string
}

// Thanks -> https://stackoverflow.com/a/77012282
export function createGrid(
    config: GridConfig
) {
    const points = [];

    const material = new LineBasicMaterial({
        color: config.color,
        transparent: true,
        opacity: 0.2
    });

    const gridObject = new Object3D();
    const stepw = 2 * config.width / config.linesWidth;
    const steph = 2 * config.height / config.linesHeight;

    // Add horizontal lines
    for (let i = -config.height; i <= config.height; i += steph) {
        points.push(new Vector3(-config.width, i, 0));
        points.push(new Vector3(config.width, i, 0));
    }

    // Add vertical lines
    for (let i = -config.width; i <= config.width; i += stepw) {
        points.push(new Vector3(i, -config.height, 0));
        points.push(new Vector3(i, config.height, 0));
    }

    const gridGeo = new BufferGeometry().setFromPoints(points);
    const line = new LineSegments(gridGeo, material);

    gridObject.add(line);
    return gridObject
}

export function useThreeSetup(
    theme: Theme,
    canvasRef: RefObject<HTMLCanvasElement>,
    canvasContainerRef: RefObject<HTMLDivElement>
): UseThreeSetupRet {
    const threeConfigRef = useRef<ThreeConfig>({
        ambientLight: undefined,
        camera: undefined,
        controls: undefined,
        gridHelper: undefined,
        raycaster: undefined,
        renderer: undefined,
        scene: undefined,
        stats: undefined
    })

    function onResize() {
        if (!threeConfigRef.current.renderer) return

        // TODO: Dirty hack to make the resizing work when collapsing the panel
        // For some reason the onresize on the panel is called before the actual container is resized, so we need to
        // have a small wait duration here to give it time to resize before updating the canvas size
        setTimeout(() => {
            const newWidth = canvasContainerRef.current.clientWidth
            const newHeight = canvasContainerRef.current.clientHeight

            threeConfigRef.current.renderer.setSize(newWidth, newHeight)
            threeConfigRef.current.camera.position.z = 999999
            threeConfigRef.current.camera.left = newWidth / -2
            threeConfigRef.current.camera.right = newWidth / 2
            threeConfigRef.current.camera.top = newHeight / 2
            threeConfigRef.current.camera.bottom = newHeight / -2
            canvasRef.current.width = newWidth
            canvasRef.current.height = newHeight
        }, 5)
    }

    useEffect(() => {
        threeConfigRef.current.scene = new Scene()

        if (SHOW_STATS) {
            const stats = new Stats()
            stats.showPanel(0)
            stats.dom.style.top = "64px"
            stats.dom.style.left = "unset"
            stats.dom.style.right = "2px"
            canvasContainerRef.current.appendChild(stats.dom)
            threeConfigRef.current.stats = stats
        }

        const canvasWidth = canvasContainerRef.current.clientWidth
        const canvasHeight = canvasContainerRef.current.clientHeight

        const camera = new OrthographicCamera(
            canvasWidth / -2,
            canvasWidth / 2,
            canvasHeight / 2,
            canvasHeight / -2,
            0.01,
            999999
        )
        camera.position.z = 999999

        const renderer = new WebGLRenderer({canvas: canvasRef.current, alpha: true})
        renderer.setSize(canvasWidth, canvasHeight)

        const controls = new OrbitControls(camera, canvasRef.current)
        controls.maxZoom = MIN_ZOOM
        controls.minZoom = MAX_ZOOM
        controls.enableRotate = false
        controls.enablePan = true
        controls.zoomToCursor = true
        controls.enableDamping = true
        controls.dampingFactor = 0.07

        const ambientLight = new AmbientLight("#FFFFFF", 5)

        threeConfigRef.current.scene.add(ambientLight)
        threeConfigRef.current.raycaster = new Raycaster()
        threeConfigRef.current.camera = camera
        threeConfigRef.current.renderer = renderer
        threeConfigRef.current.controls = controls
        threeConfigRef.current.ambientLight = ambientLight
    }, []);

    useEffect(() => {
        threeConfigRef.current.renderer.setClearColor(getColorFromTheme(theme, "darker"))
    }, [theme]);

    useEffect(() => {
        window.addEventListener("resize", onResize)
        canvasContainerRef.current.addEventListener("resize", onResize)

        return () => {
            threeConfigRef.current.scene.remove(threeConfigRef.current.ambientLight)
            threeConfigRef.current.scene.remove(threeConfigRef.current.gridHelper)
            window.removeEventListener("resize", onResize)
        }
    }, []);

    return {threeConfigRef, onResize}
}