import {AmbientLight, GridHelper, Group, OrthographicCamera, Raycaster, Scene, Vector3, WebGLRenderer} from "three";
import React, {MutableRefObject, useCallback, useEffect, useRef, useState} from "react";
import Stats from "stats.js";
import {getColorFromTheme, Theme} from "./useTheme.ts";
import {degToRad} from "three/src/math/MathUtils.js";
import {DrawAnimatedSprite, DrawStaticSprite, Tilesheets} from "../rendering/tilesheets.ts";
import {ArcballControls} from "three/examples/jsm/controls/ArcballControls.js";
import {useMousePosition} from "./useMousePosition.ts";
import {BackendResponseType, invokeTauri, makeCancelable, serializedVec2ToVector2} from "../lib/index.ts";
import {listen} from "@tauri-apps/api/event";
import {
    DisplayItemGroup,
    DisplayItemGroupType,
    MapDataEvent,
    MapDataSendCommand,
    PlaceSpritesEvent
} from "../lib/map_data.ts";
import {Project} from "../lib/project.js";

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

type UseEditorRet = {
    resize: () => void,
    displayInLeftPanel: {
        items: React.JSX.Element[] | React.JSX.Element
        monsters: React.JSX.Element[] | React.JSX.Element
    }
}

export function useEditor(props: Props): UseEditorRet {
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
    const worldMousePosition = useRef<Vector3>(new Vector3(0, 0, 0))

    const [itemDisplay, setItemDisplay] = useState<React.JSX.Element>()
    const [cellData, setCellData] = useState<{
        [coords: string]: { item_groups: DisplayItemGroup[] }
    }>({})

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

    useEffect(() => {
        function getChildrenOfDisplayGroup(
            level: number,
            itemGroup: DisplayItemGroup,
        ): React.JSX.Element[] {
            const innerGroup: React.JSX.Element[] = []

            if (itemGroup.type === DisplayItemGroupType.Single) {
                innerGroup.push(<div key={`${level}-${itemGroup.item}`}>{itemGroup.item}, {itemGroup.probability}</div>)
            } else if (itemGroup.type === DisplayItemGroupType.Distribution || itemGroup.type === DisplayItemGroupType.Collection) {
                if (itemGroup.items.length === 0) return innerGroup

                innerGroup.push(
                    <fieldset style={{marginLeft: level * 2}} className={"item-container"} key={`${level}-${itemGroup.type}`}>
                        <legend>{itemGroup.name} {itemGroup.type} {itemGroup.probability}</legend>
                        {itemGroup.items.map(ig => getChildrenOfDisplayGroup(level + 1, ig))}
                    </fieldset>
                )
            }

            return innerGroup
        }

        function onCellChange(newCoordinates: Vector3) {
            const group = cellData[`${newCoordinates.x},${newCoordinates.y},${currentZLayer}`]

            if (!group) return;

            const newItemDisplay = []

            for (const itemGroup of group.item_groups) {
                newItemDisplay.push(getChildrenOfDisplayGroup(0, itemGroup))
            }

            setItemDisplay(
                <div style={{display: "flex", height: "100%", flexDirection: "column"}}>
                    {newItemDisplay}
                </div>
            )
        }

        function setHoveredCell() {
            if (!itemTooltipGroupRef.current) return

            const rect = rendererRef.current.domElement.getBoundingClientRect();
            const mouseNormalized = new Vector3();
            mouseNormalized.x = ((mousePosition.current.x - rect.left) / (rect.right - rect.left)) * 2 - 1;
            mouseNormalized.y = -((mousePosition.current.y - rect.top) / (rect.bottom - rect.top)) * 2 + 1;
            mouseNormalized.z = 0

            const offset = new Vector3(0.5, 0.5, 0)
            const worldCoords = mouseNormalized.unproject(cameraRef.current).divide(new Vector3(32, 32, 1)).add(offset).floor();

            if (worldCoords.x !== worldMousePosition.current.x || worldCoords.y !== worldMousePosition.current.y) {
                onCellChange(worldCoords)
            }

            worldMousePosition.current = worldCoords
        }

        window.addEventListener("mousemove", setHoveredCell)

        return () => {
            window.removeEventListener("mousemove", setHoveredCell)
        }
    }, [cellData, currentZLayer, mousePosition]);

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

        async function getCellData() {
            const response = await invokeTauri<{
                [coords: string]: { item_groups: DisplayItemGroup[] }
            }, unknown>(MapDataSendCommand.GetProjectCellData, {});

            if (response.type === BackendResponseType.Error) {
                return
            }

            setCellData(response.data)
        }

        async function getCurrentProjectData() {
            const response = await invokeTauri<Project, unknown>(MapDataSendCommand.GetCurrentProjectData, {})

            if (response.type === BackendResponseType.Error) {
                console.error(response.error)
                return
            }

            const group = new Group()

            itemTooltipGroupRef.current = group
            props.sceneRef.current.add(group)
        }

        getCurrentProjectData()
        getCellData()

        if (props.tilesheetsRef.current) props.tilesheetsRef.current.clearAll();

        let handler: number;

        function loop() {
            statsRef.current.begin()

            cameraRef.current.updateProjectionMatrix()
            if (props.tilesheetsRef.current) props.tilesheetsRef.current.updateAnimatedSprites()

            controlsRef.current.update()
            rendererRef.current.render(props.sceneRef.current, cameraRef.current)

            statsRef.current.end()

            handler = requestAnimationFrame(loop)
        }

        loop()

        return () => {
            cancelAnimationFrame(handler)
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

    return {resize: onResize, displayInLeftPanel: {items: itemDisplay, monsters: []}}
}