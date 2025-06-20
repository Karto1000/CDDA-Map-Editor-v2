import React, {useEffect, useRef} from "react";
import GenericWindow, {WINDOW_CLOSED} from "../generic-window.js";
import "./main.scss"
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {Tooltip} from "react-tooltip";
import {MapEditorData, Project} from "../../tauri/types/editor.js";
import {useInitialData} from "../useInitialData.js";
import {
    CDDADistributionInner,
    MapGenValue,
    MeabyVec,
    meabyVecToArray,
    MeabyWeighted
} from "../../tauri/types/map_data.js";
import {Window} from "@tauri-apps/api/window";
import {openWindow, WindowLabel} from "../lib.js";
import {Theme} from "../../shared/hooks/useTheme.js";
import {listen, UnlistenFn} from "@tauri-apps/api/event";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";

function Main() {
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()
    const [project, setProject] = useInitialData<Project<MapEditorData>>()
    const addPaletteWindowRef = useRef<WebviewWindow>(null)
    const addPaletteWindowCloseRef = useRef<UnlistenFn>(null)

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

    function getPaletteVisualization(palette: MapGenValue, i: number) {
        if (Array.isArray(palette)) {
            return <div className={"palette-item"} key={i} data-tooltip-id={"info-tooltip"}
                        data-tooltip-html={getListFromDistribution(palette).join(", <br/>")}
                        onMouseMove={handleMouseMove}>
                <button>X</button>
                <span>{getStringFromMapGenValue(palette[0])} (+{palette.length - 1})</span>
            </div>
        }

        return <div className={"palette-item"} key={i}>
            <button>X</button>
            <span>{getStringFromMapGenValue(palette)}</span>
        </div>
    }

    async function onAddPalette() {
        // TODO: Theme
        const [window, close] = await openWindow(WindowLabel.AddPalette, Theme.Dark, {}, null, WindowLabel.Palettes)

        addPaletteWindowRef.current = window
        addPaletteWindowCloseRef.current = close
    }

    return (
        <GenericWindow title={"Palettes"}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>
            <p>In this window you can see a list of palettes defined in the current map</p>
            <div className={"line-break"}/>
            {
                project &&
                <div className={"palettes-list"}>
                    <button onClick={onAddPalette}>Add Palette</button>

                    {
                        project.project_type.mapEditor.maps[0].maps["0,0"].palettes.map(getPaletteVisualization)
                    }
                </div>
            }
        </GenericWindow>
    );
}

export default Main;
