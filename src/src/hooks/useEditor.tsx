import {
    AmbientLight,
    GridHelper,
    Group,
    Mesh,
    MeshBasicMaterial,
    OrthographicCamera,
    PlaneGeometry,
    Raycaster,
    Scene,
    Vector2,
    Vector3,
    WebGLRenderer
} from "three";
import React, {MutableRefObject, ReactElement, useCallback, useEffect, useRef, useState} from "react";
import Stats from "stats.js";
import {getColorFromTheme, Theme} from "./useTheme.ts";
import {degToRad} from "three/src/math/MathUtils.js";
import {DrawAnimatedSprite, DrawStaticSprite, MAX_DEPTH, Tilesheets} from "../rendering/tilesheets.ts";
import {ArcballControls} from "three/examples/jsm/controls/ArcballControls.js";
import {useMousePosition} from "./useMousePosition.ts";
import {BackendResponseType, invokeTauri, makeCancelable, serializedVec2ToVector2} from "../lib/index.ts";
import {listen} from "@tauri-apps/api/event";
import {
    CellData,
    DisplayItemGroup,
    DisplayItemGroupType,
    MapDataEvent,
    MapDataSendCommand,
    PlaceSpritesEvent
} from "../lib/map_data.ts";
import {Project} from "../lib/project.js";
import {SpritesheetConfig} from "../lib/tileset/legacy.js";
import {Fieldset} from "../components/fieldset.tsx";

const MIN_ZOOM: number = 500;
const MAX_ZOOM: number = 0.05;

type UseEditorProps = {
    sceneRef: MutableRefObject<Scene>,
    canvasRef: MutableRefObject<HTMLCanvasElement>
    canvasContainerRef: MutableRefObject<HTMLDivElement>
    tilesheetsRef: MutableRefObject<Tilesheets>
    spritesheetConfig: MutableRefObject<SpritesheetConfig>

    openedTab: number
    theme: Theme
    isDisplaying: boolean
    isTilesheetLoaded: boolean
}

type UseEditorRet = {
    resize: () => void,
    displayInLeftPanel: {
        items: React.JSX.Element[] | React.JSX.Element
        monsters: React.JSX.Element[] | React.JSX.Element,
        signs: React.JSX.Element[] | React.JSX.Element
    }
}

type ItemPanelProps = {
    rendererRef: MutableRefObject<WebGLRenderer>
    cameraRef: MutableRefObject<OrthographicCamera>
    mousePosition: MutableRefObject<Vector2>
    worldMousePosition: MutableRefObject<Vector3>
    currentZLayer: number
    cellData: CellData
    selectedCellPosition: Vector3 | null
}

export function ItemPanel(props: ItemPanelProps) {
    const [query, setQuery] = useState<string>("")
    const [displayItem, setDisplayItem] = useState<React.JSX.Element[]>([])
    const [currentGroup, setCurrentGroup] = useState<{ item_groups: DisplayItemGroup[] }>()

    function resetCurrentGroup() {
        setCurrentGroup(undefined)
        setQuery("")
        setDisplayItem([])
    }

    useEffect(() => {
        if (!currentGroup) return

        function getChildrenOfDisplayGroup(
            level: number,
            itemGroup: DisplayItemGroup,
            index: number
        ): React.JSX.Element[] | null {
            const innerGroup: React.JSX.Element[] = []
            const probability = itemGroup.probability * 100

            if (itemGroup.type === DisplayItemGroupType.Single) {
                if (!itemGroup.item.includes(query)) return null

                innerGroup.push(
                    <div
                        key={`${index}-${level}-${itemGroup.item}-${itemGroup.probability}`}>{itemGroup.item}, {probability < 0.009 ? "0.00<" : probability.toFixed(2)}%
                    </div>
                )
            } else if (itemGroup.type === DisplayItemGroupType.Distribution || itemGroup.type === DisplayItemGroupType.Collection) {
                if (itemGroup.items.length === 0) return null

                const children = itemGroup.items
                    .map(ig => getChildrenOfDisplayGroup(level + 1, ig, index))
                    .filter(v => v !== null)

                if (children.length === 0) return null

                innerGroup.push(
                    <Fieldset
                        legend={`${itemGroup.name} ${itemGroup.type} ${probability < 0.009 ? "0.00<" : probability.toFixed(2)}%`}
                        key={`${index}-${level}-${itemGroup.type}-${itemGroup.probability}`}
                        style={{marginLeft: level * 2}}
                        className={"item-container"}>
                        {children}
                    </Fieldset>
                )
            }

            return innerGroup
        }

        const displayGroups = []

        currentGroup.item_groups.forEach((itemGroup, i) => {
            const display = getChildrenOfDisplayGroup(0, itemGroup, i)

            if (!display) return

            displayGroups.push(...display)

        })

        setDisplayItem(displayGroups)
    }, [currentGroup, query]);

    useEffect(() => {
        if (!props.selectedCellPosition) {
            resetCurrentGroup()
            return;
        }

        const group = props.cellData[`${props.selectedCellPosition.x},${props.selectedCellPosition.y},${props.currentZLayer}`]

        if (!group) {
            resetCurrentGroup()
            return;
        }

        setCurrentGroup(group)
    }, [props.cellData, props.currentZLayer, props.selectedCellPosition]);

    return (
        <div className={"menu-body-container"}>
            <input
                placeholder={"Search..."}
                value={query}
                type={"text"}
                onChange={e => setQuery(e.target.value)}
            />
            {displayItem}
        </div>
    )
}

export function useEditor(props: UseEditorProps): UseEditorRet {
    const rendererRef = useRef<WebGLRenderer>()
    const cameraRef = useRef<OrthographicCamera>()
    const controlsRef = useRef<ArcballControls>()
    const gridHelperRef = useRef<GridHelper>()
    const ambientLightRef = useRef<AmbientLight>()
    const raycasterRef = useRef<Raycaster>()
    const statsRef = useRef<Stats>()

    const [currentZLayer, setCurrentZLayer] = useState<number>(0)
    const mousePosition = useMousePosition(props.canvasRef)
    const worldMousePosition = useRef<Vector3>(new Vector3(0, 0, 0))

    const hoveredCellMeshRef = useRef<Mesh<PlaneGeometry, MeshBasicMaterial> | null>(null)
    const selectedCellMeshRef = useRef<Mesh<PlaneGeometry, MeshBasicMaterial> | null>(null)

    const [selectedCellPosition, setSelectedCellPosition] = useState<Vector3 | null>(null)

    const [itemDisplay, setItemDisplay] = useState<ReactElement<ItemPanelProps>>()
    const [signDisplay, setSignDisplay] = useState<React.JSX.Element>()
    const [cellData, setCellData] = useState<CellData>({})

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

        window.addEventListener("resize", onResize)
        props.canvasContainerRef.current.addEventListener("resize", onResize)

        return () => {
            props.sceneRef.current.remove(ambientLightRef.current)
            props.sceneRef.current.remove(gridHelperRef.current)
            window.removeEventListener("resize", onResize)
        }
    }, [onResize, props.canvasContainerRef, props.canvasRef, props.sceneRef]);

    useEffect(() => {
        setItemDisplay(
            <ItemPanel
                rendererRef={rendererRef}
                cameraRef={cameraRef}
                mousePosition={mousePosition}
                worldMousePosition={worldMousePosition}
                currentZLayer={currentZLayer}
                cellData={cellData}
                selectedCellPosition={selectedCellPosition}
            />
        )

        const selectedData = cellData[`${selectedCellPosition?.x},${selectedCellPosition?.y},${currentZLayer}`]

        setSignDisplay(
            <>
                {
                    selectedData?.signs.signage &&
                    <p>
                        Signage: {selectedData.signs.signage}
                    </p>
                }
                {
                    selectedData?.signs.snippet &&
                    <p>
                        Snippet: {selectedData.signs.snippet}
                    </p>
                }
            </>
        )
    }, [cellData, currentZLayer, mousePosition, selectedCellPosition]);

    // Should run when the MapEditor is opened
    useEffect(() => {
        if (!props.isDisplaying) return;
        if (!props.spritesheetConfig) return;

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
            const response = await invokeTauri<CellData, unknown>(MapDataSendCommand.GetProjectCellData, {});

            if (response.type === BackendResponseType.Error) {
                return
            }

            setCellData(response.data)
        }

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
            props.sceneRef.current.remove(selectedCellMeshRef.current)
            props.sceneRef.current.remove(hoveredCellMeshRef.current)
        }
    }, [
        props.tilesheetsRef,
        props.canvasContainerRef,
        props.isDisplaying,
        props.sceneRef,
        mousePosition,
        props.openedTab,
        props.spritesheetConfig
    ]);

    // Should run when the theme changes to change colors
    useEffect(() => {
        if (!props.isDisplaying) return;
        if (!props.spritesheetConfig) return;

        const tile_info = props.spritesheetConfig.current.tile_info[0]

        rendererRef.current.setClearColor(getColorFromTheme(props.theme, "darker"))

        const hovered = new PlaneGeometry(tile_info.width, tile_info.height)
        const hoveredMaterial = new MeshBasicMaterial({color: getColorFromTheme(props.theme, "darkBlue")})
        hoveredMaterial.transparent = true
        hoveredMaterial.opacity = 0.5
        const highlightedMesh = new Mesh(hovered, hoveredMaterial)
        highlightedMesh.position.set(
            worldMousePosition.current.x * tile_info.width,
            worldMousePosition.current.y * tile_info.height,
            MAX_DEPTH + 1
        )

        const selected = new PlaneGeometry(tile_info.width, tile_info.height)
        const selectedMaterial = new MeshBasicMaterial({color: getColorFromTheme(props.theme, "selected")})
        selectedMaterial.transparent = true
        selectedMaterial.opacity = 0.5

        const selectedMesh = new Mesh(selected, selectedMaterial)
        selectedMesh.visible = false

        selectedCellMeshRef.current = selectedMesh
        props.sceneRef.current.add(selectedMesh)

        hoveredCellMeshRef.current = highlightedMesh
        props.sceneRef.current.add(highlightedMesh)

        const gridHelper = new GridHelper(
            1,
            16 * 8 * tile_info.width * 24 / tile_info.height,
            getColorFromTheme(props.theme, "disabled"), getColorFromTheme(props.theme, "light")
        )
        gridHelper.scale.x = 16 * 8 * tile_info.width * 24
        gridHelper.scale.z = 16 * 8 * tile_info.height * 24

        gridHelper.position.x -= tile_info.width / 2
        gridHelper.position.y -= tile_info.height / 2

        gridHelper.rotateX(degToRad(90))
        props.sceneRef.current.add(gridHelper)
        gridHelperRef.current = gridHelper
    }, [props.isDisplaying, props.sceneRef, props.spritesheetConfig, props.theme]);

    useEffect(() => {
        if (!props.isDisplaying) return;
        if (!props.spritesheetConfig) return;

        const tile_info = props.spritesheetConfig.current.tile_info[0]

        const keydownListener = async (e: KeyboardEvent) => {
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

        function onMouseMove() {
            const rect = rendererRef.current.domElement.getBoundingClientRect();
            const mouseNormalized = new Vector3();
            mouseNormalized.x = ((mousePosition.current.x - rect.left) / (rect.right - rect.left)) * 2 - 1;
            mouseNormalized.y = -((mousePosition.current.y - rect.top) / (rect.bottom - rect.top)) * 2 + 1;
            mouseNormalized.z = 0

            const offset = new Vector3(0.5, 0.5, 0)
            worldMousePosition.current = mouseNormalized.unproject(cameraRef.current)
                .divide(new Vector3(tile_info.width, tile_info.height, 1))
                .add(offset)
                .floor()

            // We need to invert the world mouse position since the cdda map goes from up to down
            // Additionally, we need to remove 1 since the top left tile starts at +1
            worldMousePosition.current.y = -worldMousePosition.current.y - 1

            // Here we need to invert the y again to make it fit correctly for three.js
            hoveredCellMeshRef.current.position.set(
                worldMousePosition.current.x * tile_info.width,
                // Remove one again for three.js since the top left tile is -1 in three.js
                (-worldMousePosition.current.y - 1) * tile_info.height,
                MAX_DEPTH + 1
            )
        }

        function onMouseDown(e: MouseEvent) {
            if (e.button === 0) {
                if (selectedCellPosition?.x === worldMousePosition.current.x && selectedCellPosition?.y === worldMousePosition.current.y) {
                    setSelectedCellPosition(null)
                } else {
                    setSelectedCellPosition(worldMousePosition.current)
                }
            }
        }

        props.canvasRef.current.addEventListener("keydown", keydownListener)
        props.canvasRef.current.addEventListener("mousemove", onMouseMove)
        props.canvasRef.current.addEventListener("mousedown", onMouseDown)

        return () => {
            props.canvasRef.current.removeEventListener("keydown", keydownListener)
            props.canvasRef.current.removeEventListener("mousemove", onMouseMove)
            props.canvasRef.current.removeEventListener("mousedown", onMouseDown)
        }
    }, [currentZLayer, mousePosition, props.canvasRef, props.isDisplaying, props.spritesheetConfig, props.tilesheetsRef, selectedCellPosition]);

    // Should run when the tilesheet has finished loading
    useEffect(() => {
        if (!props.isTilesheetLoaded) return;

        const tileInfo = props.spritesheetConfig.current.tile_info[0]

        let placeMultiUnlistenFn = makeCancelable(listen<PlaceSpritesEvent>(MapDataEvent.PlaceSprites, d => {
            const drawStaticSprites: DrawStaticSprite[] = d.payload.static_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= tileInfo.width;
                vec2.y *= tileInfo.height;

                return {
                    ...ds,
                    position: vec2
                }
            })

            const drawAnimatedSprites: DrawAnimatedSprite[] = d.payload.animated_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= tileInfo.width;
                vec2.y *= tileInfo.height;

                return {
                    ...ds,
                    position: vec2,
                }
            })

            const drawFallbackSprites: DrawStaticSprite[] = d.payload.fallback_sprites.map(ds => {
                const vec2 = serializedVec2ToVector2(ds.position)
                vec2.x *= tileInfo.width;
                vec2.y *= tileInfo.height;

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
    }, [props.isTilesheetLoaded, props.spritesheetConfig, props.tilesheetsRef]);

    // Runs when a new cell is selected
    useEffect(() => {
        if (!props.isDisplaying) return;
        if (!props.spritesheetConfig) return;

        const tile_info = props.spritesheetConfig.current.tile_info[0]

        if (!selectedCellPosition) {
            selectedCellMeshRef.current.visible = false
            return
        }

        selectedCellMeshRef.current.position.set(
            selectedCellPosition.x * tile_info.width,
            (-selectedCellPosition.y - 1) * tile_info.height,
            MAX_DEPTH + 1
        )
        selectedCellMeshRef.current.visible = true
    }, [props.isDisplaying, props.spritesheetConfig, selectedCellPosition]);

    return {resize: onResize, displayInLeftPanel: {items: itemDisplay, monsters: [], signs: signDisplay}}
}