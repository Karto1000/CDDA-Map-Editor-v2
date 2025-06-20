import React, {useEffect, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {WindowLabel} from "../lib.js";
import {emitTo, once} from "@tauri-apps/api/event";
import {MapEditorData, Project} from "../../tauri/types/editor.js";
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {Tooltip} from "react-tooltip";
import {useInitialData} from "../useInitialData.js";
import {useCurrentProject} from "../../shared/hooks/useCurrentProject.js";
import {useForeignOpenedTab} from "../useForeignOpenedTab.js";

function Main() {
    const openedTab = useForeignOpenedTab()
    const project = useCurrentProject<MapEditorData>(openedTab)
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()

    return (
        <GenericWindow title={"Map Info"}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>

            <p>
                Here you can find information about the currently opened map.
            </p>

            {
                project &&
                <div className={"form-elements"}>
                    <div className={"form-element"}>
                        <input
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-html="The name of the Project"
                            onMouseMove={handleMouseMove}
                            value={project.name}
                        />
                        <label>Project Name</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-html="The map width"
                            onMouseMove={handleMouseMove}
                            value={project.project_type.mapEditor.size[0]}
                        />
                        <label>Width</label>
                    </div>
                    <div className={"form-element"}>
                        <input
                            data-tooltip-id={"info-tooltip"}
                            data-tooltip-html="The map height"
                            onMouseMove={handleMouseMove}
                            value={project.project_type.mapEditor.size[1]}
                        />
                        <label>Height</label>
                    </div>
                </div>
            }
        </GenericWindow>
    );
}

export default Main;
