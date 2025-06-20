import React from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {useMouseTooltip} from "../../shared/hooks/useMouseTooltip.js";
import {Tooltip} from "react-tooltip";
import {MapDataCollection, MapEditorData, Project} from "../../tauri/types/editor.js";
import {useInitialData} from "../useInitialData.js";

function Main() {
    const [tooltipPosition, handleMouseMove] = useMouseTooltip()
    const [project, setProject] = useInitialData<Project<MapEditorData>>()

    return (
        <GenericWindow title={"Palettes"}>
            <Tooltip id="info-tooltip" positionStrategy={"fixed"} position={tooltipPosition} delayShow={500}
                     noArrow={true} className="tooltip" opacity={1} offset={20} place={"bottom-end"}/>
            <p>In this window you can see a list of palettes defined in the current map</p>
            <div className={"line-break"}/>
            {
                project &&
                <div className={"palettes-container"}>
                    {
                        Object.keys(project.project_type.mapEditor.maps).map(m => {
                            const mapCollection: MapDataCollection = project.project_type.mapEditor.maps[m]

                            return Object.keys(mapCollection).map(m => {
                                const map = mapCollection[m]

                                return (
                                    <div key={m}>
                                        {map.palettes}
                                    </div>
                                )
                            })
                        })
                    }
                </div>
            }
        </GenericWindow>
    );
}

export default Main;
