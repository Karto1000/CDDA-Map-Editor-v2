import React, {useEffect, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {WindowLabel} from "../lib.js";
import {emitTo, once} from "@tauri-apps/api/event";
import {MapEditorData, Project} from "../../tauri/types/editor.js";

function Main() {
    const [project, setProject] = useState<Project<MapEditorData>>(null)

    useEffect(() => {
        (async () => {
            await once<Project<MapEditorData>>("initial-data", p => {
                setProject(p.payload)
            })

            await emitTo(WindowLabel.MapInfo, "window-ready")
        })()
    }, []);

    return (
        <GenericWindow title={"Map Info"}>
            <p>{project?.name}</p>
        </GenericWindow>
    );
}

export default Main;
