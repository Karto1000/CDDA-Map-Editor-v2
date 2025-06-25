import React, {useRef} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {Tooltip} from "react-tooltip";
import {
    CDDADistributionInner,
    MapGenValue,
    MeabyVec,
    meabyVecToArray,
    MeabyWeighted
} from "../../tauri/types/map_data.js";
import {openWindow, WindowLabel} from "../lib.js";
import {Theme} from "../../shared/hooks/useTheme.js";
import {UnlistenFn} from "@tauri-apps/api/event";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {useCurrentProject} from "../../shared/hooks/useCurrentProject.js";
import {MapEditorData} from "../../tauri/types/editor.js";
import {useForeignOpenedTab} from "../useForeignOpenedTab.js";
import {Vector3} from "three";
import {useInitialData} from "../useInitialData.js";
import {useTauriEvent} from "../../shared/hooks/useTauriEvent.js";
import {ModifyPaletteActionKind, TauriCommand, TauriEvent} from "../../tauri/events/types.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {MultiMenu} from "../../shared/components/imguilike/multimenu.js";
import {__PALETTE_ADDED} from "../add-palette/main.js";

function Main() {
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()

    const addPaletteWindowRef = useRef<WebviewWindow>(null)
    const addPaletteWindowCloseRef = useRef<UnlistenFn>(null)
    const addPaletteSelectedUnlistenRef = useRef<UnlistenFn>(null)

    const openedTab = useForeignOpenedTab()
    const project = useCurrentProject<MapEditorData>(openedTab)
    const [chunkPosition, setChunkPosition] = useInitialData<Vector3>()

    useTauriEvent(
        TauriEvent.MAPGEN_CHUNK_SELECTED,
        chunk => {
            if (addPaletteWindowCloseRef.current) addPaletteWindowCloseRef.current()

            setChunkPosition(chunk)
        },
        [chunkPosition]
    )

    function getStringFromMapGenValue(palette: MapGenValue): string {
        if (typeof palette === "string") return palette
        if ("param" in palette) return `(Param) ${palette.param}`
        if ("switch" in palette) return `(Switch) ${palette.switch.param}`
        if ("data" in palette) return `${getStringFromMapGenValue(palette.data)}`
    }

    function getListFromDistribution(distribution: MeabyVec<MeabyWeighted<CDDADistributionInner>>): string[] {
        const transformedValue = meabyVecToArray(distribution)
        return transformedValue.map((v) => {
            if (typeof v === "string") return `(1) ${v}`
            if ("weight" in v) return `(${v.weight}) ${getStringFromMapGenValue(v)}`
        })
    }

    async function onPaletteRemove(i: number) {
        console.log("Removing palette", i)
        await tauriBridge.invoke(
            TauriCommand.MODIFY_PALETTE,
            {
                action: {
                    type: ModifyPaletteActionKind.RemovePalette,
                    index: i
                },
                coordinates: [chunkPosition.x, chunkPosition.y, chunkPosition.z],
            }
        )
    }

    function getPaletteVisualization(palette: MapGenValue, i: number) {
        if (Array.isArray(palette)) {
            return <div className={"palette-item"} key={i} data-tooltip-id={"info-tooltip"}
                        data-tooltip-html={getListFromDistribution(palette).join(", <br/>")}
                        onMouseMove={handleMouseMove}>
                <button onClick={() => onPaletteRemove(i)}>X</button>
                <span>{getStringFromMapGenValue(palette[0])} (+{palette.length - 1})</span>
            </div>
        }

        return <div className={"palette-item"} key={i}>
            <button onClick={() => onPaletteRemove(i)}>X</button>
            <span>{getStringFromMapGenValue(palette)}</span>
        </div>
    }

    async function onAddPalette() {
        // TODO: Theme
        const [window, unlistenFn] = await openWindow(
            WindowLabel.AddPalette,
            Theme.Dark,
            addPaletteWindowRef,
            {},
            {},
            WindowLabel.Chunk
        )

        addPaletteSelectedUnlistenRef.current = await window.once<string>(
            __PALETTE_ADDED,
            async (e) => {
                await tauriBridge.invoke(
                    TauriCommand.MODIFY_PALETTE,
                    {
                        action: {
                            type: ModifyPaletteActionKind.AddPalette,
                            paletteName: e.payload,
                        },
                        coordinates: [chunkPosition.x, chunkPosition.y, chunkPosition.z],
                    }
                )

                addPaletteWindowCloseRef.current()
            }
        )

        addPaletteWindowCloseRef.current = unlistenFn
    }

    let windowTitle = "Loading..."
    if (project && chunkPosition) {
        const projectMap = project.project_type
            .mapEditor
            .maps[chunkPosition.z]
            .maps[`${chunkPosition.x},${chunkPosition.y}`]

        windowTitle = `${projectMap.id} at ${chunkPosition.x}, ${chunkPosition.y}, ${chunkPosition.z}`
    }


    return (
        <GenericWindow title={windowTitle}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>
            <MultiMenu tabs={
                [
                    {
                        name: "Palettes",
                        content: <>
                            <p>In this window you can see a list of palettes defined in the current map</p>
                            <div className={"line-break"}/>
                            {
                                project && chunkPosition &&
                                <div className={"palettes-list"}>
                                    <button onClick={onAddPalette}>Add Palette</button>

                                    {
                                        project.project_type
                                            .mapEditor
                                            .maps[chunkPosition.z]
                                            .maps[`${chunkPosition.x},${chunkPosition.y}`]
                                            .palettes
                                            .map(getPaletteVisualization)
                                    }
                                </div>
                            }
                        </>
                    }
                ]
            }/>
        </GenericWindow>
    );
}

export default Main;
