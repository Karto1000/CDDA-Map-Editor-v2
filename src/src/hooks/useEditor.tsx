import {
    AmbientLight,
    GridHelper,
    Group,
    Mesh,
    MeshBasicMaterial,
    OrthographicCamera,
    Plane,
    PlaneGeometry,
    Raycaster,
    Scene,
    Vector2,
    Vector3,
    WebGLRenderer
} from "three";
import React, {MutableRefObject, useCallback, useEffect, useRef, useState} from "react";
import Stats from "stats.js";
import {getColorFromTheme, Theme} from "./useTheme.ts";
import {degToRad} from "three/src/math/MathUtils.js";
import {DrawAnimatedSprite, DrawStaticSprite, MAX_DEPTH, Tilesheets} from "../rendering/tilesheets.ts";
import {ArcballControls} from "three/examples/jsm/controls/ArcballControls.js";
import {useMousePosition} from "./useMousePosition.ts";
import {invokeTauri, makeCancelable, serializedVec2ToVector2, serializedVec3ToVector3} from "../lib/index.ts";
import {listen} from "@tauri-apps/api/event";
import {ItemDataEvent, MapDataEvent, MapDataSendCommand, MapGenItem, PlaceSpritesEvent} from "../lib/map_data.ts";

const MIN_ZOOM: number = 500;
const MAX_ZOOM: number = 0.05;

type Props = {
    sceneRef: MutableRefObject<Scene>,
    canvasRef: MutableRefObject<HTMLCanvasElement>
    canvasContainerRef: MutableRefObject<HTMLDivElement>
    tilesheetsRef: MutableRefObject<Tilesheets>

    openedTab: number
    theme: Theme
    isDisplaying: boolean
    isTilesheetLoaded: boolean
}

export function useEditor(props: Props) {
    const rendererRef = useRef<WebGLRenderer>()
    const cameraRef = useRef<OrthographicCamera>()
    const controlsRef = useRef<ArcballControls>()
    const gridHelperRef = useRef<GridHelper>()
    const ambientLightRef = useRef<AmbientLight>()
    const raycasterRef = useRef<Raycaster>()
    const statsRef = useRef<Stats>()
    const itemTooltipGroupRef = useRef<Group>()

    const [currentZLayer, setCurrentZLayer] = useState<number>(0)
    const isLeftMousePressedRef = useRef<boolean>(false)
    const mousePosition = useMousePosition(props.canvasRef)

    const [displayInLeftPanel, setDisplayInLeftPanel] = useState<React.JSX.Element[]>([])

    const onResize = useCallback(() => {
        if (!rendererRef.current) return

        const newWidth = props.canvasContainerRef.current.clientWidth
        const newHeight = props.canvasContainerRef.current.clientHeight

        rendererRef.current.setSize(newWidth, newHeight)
        cameraRef.current.position.z = 999999
        cameraRef.current.left = newWidth / -2
        cameraRef.current.right = newWidth / 2
        cameraRef.current.top = newHeight / 2
        cameraRef.current.bottom = newHeight / -2
    }, [props.canvasContainerRef])

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

            raycasterRef.current = new Raycaster()
            statsRef.current = stats
            cameraRef.current = camera
            rendererRef.current = renderer
            controlsRef.current = controls
            ambientLightRef.current = ambientLight
        }

        setup()

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
        props.canvasContainerRef.current.addEventListener("resize", onResize)
        props.canvasRef.current.addEventListener("mousedown", mouseDownListener)
        props.canvasRef.current.addEventListener("mouseup", mouseUpListener)

        return () => {
            props.sceneRef.current.remove(ambientLightRef.current)
            props.sceneRef.current.remove(gridHelperRef.current)
            props.canvasRef.current.removeEventListener("mousedown", mouseDownListener)
            props.canvasRef.current.removeEventListener("mouseup", mouseUpListener)
            window.removeEventListener("resize", onResize)
        }
    }, [onResize, props.canvasContainerRef, props.canvasRef, props.sceneRef]);

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

        initialValueUpdate();

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

                    // invokeTauri<PlaceSpriteCommand, unknown>(MapDataSendCommand.Place, args).then()
                }
            }
        }

        function handleItemTooltip() {
            if (!itemTooltipGroupRef.current) return

            const rect = rendererRef.current.domElement.getBoundingClientRect();
            const mouseNormalized = new Vector2();
            mouseNormalized.x = ((mousePosition.current.x - rect.left) / (rect.right - rect.left)) * 2 - 1;
            mouseNormalized.y = -((mousePosition.current.y - rect.top) / (rect.bottom - rect.top)) * 2 + 1;

            raycasterRef.current.setFromCamera(mouseNormalized, cameraRef.current);
            const intersects = raycasterRef.current.intersectObject(itemTooltipGroupRef.current);

            if (intersects.length !== 0) {
                let obj = intersects[0].object;
                setDisplayInLeftPanel(obj.userData['items'])
                return
            }

            setDisplayInLeftPanel([<></>])
        }

        const itemDataUnlistenFn = makeCancelable(listen<ItemDataEvent>(
            MapDataEvent.ItemData,
            d => {
                const group = new Group()

                for (const stringCoordinates of Object.keys(d.payload)) {
                    const vec3 = serializedVec3ToVector3(stringCoordinates)
                    const data = d.payload[stringCoordinates].map(i => i.item).join(", ")

                    const geometry = new PlaneGeometry(32, 32);
                    const material = new MeshBasicMaterial({visible: false});
                    const mesh = new Mesh(geometry, material);

                    mesh.position.x = vec3.x * 32
                    mesh.position.y = vec3.y * 32
                    mesh.position.z = MAX_DEPTH + 1

                    mesh.userData = {
                        "items": data
                    }

                    group.add(mesh)
                }

                itemTooltipGroupRef.current = group
                props.sceneRef.current.add(group)
            }))

        if (props.tilesheetsRef.current) props.tilesheetsRef.current.clearAll();

        let handler: number;

        window.addEventListener("mousemove", handleItemTooltip)

        function loop() {
            statsRef.current.begin()

            cameraRef.current.updateProjectionMatrix()
            if (props.tilesheetsRef.current) props.tilesheetsRef.current.updateAnimatedSprites()

            handleEvents()

            controlsRef.current.update()
            rendererRef.current.render(props.sceneRef.current, cameraRef.current)

            statsRef.current.end()

            handler = requestAnimationFrame(loop)
        }

        loop()

        return () => {
            cancelAnimationFrame(handler)
            itemDataUnlistenFn.cancel()
            window.removeEventListener("mousemove", handleItemTooltip)
        }
    }, [props.tilesheetsRef, props.canvasContainerRef, props.isDisplaying, props.sceneRef, mousePosition, props.openedTab]);

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

    useEffect(() => {
        const keydownListener = async (e: KeyboardEvent) => {
            e.preventDefault()

            if (e.key === "s") {
                const response = await invokeTauri<never, never>(MapDataSendCommand.SaveCurrentProject, {})
            }

            if (e.key === "PageUp") {
                const newZLayer = currentZLayer + 1
                setCurrentZLayer(newZLayer)
                props.tilesheetsRef.current.switchZLevel(newZLayer)
            }

            if (e.key === "PageDown") {
                const newZLayer = currentZLayer - 1
                setCurrentZLayer(newZLayer)
                props.tilesheetsRef.current.switchZLevel(newZLayer)
            }
        }

        window.addEventListener("keydown", keydownListener)

        return () => {
            window.removeEventListener("keydown", keydownListener)
        }
    }, [currentZLayer, props.tilesheetsRef]);

    // Should run when the tilesheet has finished loading
    useEffect(() => {
        if (!props.isTilesheetLoaded) return;

        let placeMultiUnlistenFn = makeCancelable(listen<PlaceSpritesEvent>(MapDataEvent.PlaceSprites, d => {
            const drawStaticSprites: DrawStaticSprite[] = d.payload.static_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= 32;
                vec2.y *= 32;

                return {
                    ...ds,
                    position: vec2
                }
            })

            const drawAnimatedSprites: DrawAnimatedSprite[] = d.payload.animated_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= 32;
                vec2.y *= 32;

                return {
                    ...ds,
                    position: vec2,
                }
            })

            const drawFallbackSprites: DrawStaticSprite[] = d.payload.fallback_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= 32;
                vec2.y *= 32;

                return {
                    ...ds,
                    layer: 0,
                    position: vec2,
                    rotate_deg: 0
                }
            })

            props.tilesheetsRef.current.drawFallbackSpritesBatched(drawFallbackSprites)
            props.tilesheetsRef.current.drawStaticSpritesBatched(drawStaticSprites)
            props.tilesheetsRef.current.drawAnimatedSpritesBatched(drawAnimatedSprites)
        }))

        return () => {
            placeMultiUnlistenFn.cancel()
        }
    }, [props.isTilesheetLoaded, props.tilesheetsRef]);

    return {resize: onResize, displayInLeftPanel: displayInLeftPanel}
}