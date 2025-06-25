import React, {useEffect, useRef, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {Tooltip} from "react-tooltip";
import {useCurrentProject} from "../../shared/hooks/useCurrentProject.js";
import {MapEditorData} from "../../tauri/types/editor.js";
import {useForeignOpenedTab} from "../useForeignOpenedTab.js";
import {openWindow, WindowLabel} from "../lib.js";
import {Theme} from "../../shared/hooks/useTheme.js";
import {__PALETTE_ADDED} from "../add-palette/main.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";
import {BackendResponseType, ModifyPaletteActionKind, TauriCommand} from "../../tauri/events/types.js";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {UnlistenFn} from "@tauri-apps/api/event";
import {
    CDDADistributionInner,
    MapGenValue,
    MeabyVec,
    meabyVecToArray,
    MeabyWeighted
} from "../../tauri/types/map_data.js";

function Main() {
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()
    const openedTab = useForeignOpenedTab()
    const project = useCurrentProject<MapEditorData>(openedTab)

    const addPaletteWindowRef = useRef<WebviewWindow>(null)
    const addPaletteSelectedUnlistenRef = useRef<UnlistenFn>(null)
    const addPaletteWindowCloseRef = useRef<UnlistenFn>(null)

    const [globalPalettes, setGlobalPalettes] = useState<MapGenValue[]>([])

    useEffect(() => {
        (async () => {
            const response = await tauriBridge.invoke<MapGenValue[], string>(
                TauriCommand.GET_GLOBAL_PALETTES,
                {}
            )

            if (response.type === BackendResponseType.Error) {
                return
            }

            setGlobalPalettes(response.data)
        })()
    }, [project]);

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

    async function onPaletteRemove(name: string) {
        console.log("Removing palette", name)

        await tauriBridge.invoke(
            TauriCommand.MODIFY_GLOBAL_PALETTE,
            {
                action: {
                    type: ModifyPaletteActionKind.RemovePalette,
                    paletteName: name
                },
            }
        )

    }

    function getPaletteVisualization(palette: MapGenValue, i: number) {
        // TODO: Handle other mapgen values
        if (Array.isArray(palette)) {
            return <div className={"palette-item"} key={i} data-tooltip-id={"info-tooltip"}
                        data-tooltip-html={getListFromDistribution(palette).join(", <br/>")}
                        onMouseMove={handleMouseMove}>
                <button onClick={() => onPaletteRemove(palette[0] as string)}>X</button>
                <span>{getStringFromMapGenValue(palette[0])} (+{palette.length - 1})</span>
            </div>
        }

        return <div className={"palette-item"} key={i}>
            <button onClick={() => onPaletteRemove(palette as string)}>X</button>
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
            WindowLabel.GlobalPalettes,
        )

        addPaletteSelectedUnlistenRef.current = await window.once<string>(
            __PALETTE_ADDED,
            async (e) => {
                await tauriBridge.invoke(
                    TauriCommand.MODIFY_GLOBAL_PALETTE,
                    {
                        action: {
                            type: ModifyPaletteActionKind.AddPalette,
                            paletteName: e.payload,
                        },
                    }
                )

                addPaletteWindowCloseRef.current()
            }
        )

        addPaletteWindowCloseRef.current = unlistenFn
    }

    return (
        <GenericWindow
            title={`Global Palettes`}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>
            <p>Here you can define palettes which will be added to every existing Mapgen chunk in the current map</p>
            <div className={"line-break"}/>
            {
                project &&
                <div className={"palettes-list"}>
                    <button onClick={onAddPalette}>Add Palette</button>
                    {
                        globalPalettes.map(getPaletteVisualization)
                    }
                </div>
            }
        </GenericWindow>
    );
}

export default Main;
