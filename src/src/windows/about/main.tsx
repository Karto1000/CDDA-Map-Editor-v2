import React, {useEffect, useState} from "react";
import GenericWindow from "../generic-window.js";
import "./main.scss"
import {AboutInfo, BackendResponseType, TauriCommand} from "../../tauri/events/types.js";
import {tauriBridge} from "../../tauri/events/tauriBridge.js";

function Main() {
    const [aboutInfo, setAboutInfo] = useState<AboutInfo | null>(null)

    useEffect(() => {
        (async () => {
           const response = await tauriBridge.invoke<AboutInfo, never, TauriCommand.ABOUT>(TauriCommand.ABOUT, {})

            if (response.type === BackendResponseType.Error) {
                return
            }

            setAboutInfo(response.data)
        })()
    }, []);

    return (
        <GenericWindow title={"About"}>
            <div className={"about-body"}>
                <img src={"/icons/icon.ico"} alt={"CDDA Map Editor Logo"}/>
                {
                    aboutInfo &&
                    <div>
                        <p>Version: {aboutInfo.version}</p>
                        <p>Contributors: {aboutInfo.contributors}</p>
                        <p>Description: {aboutInfo.description}</p>
                    </div>
                }
            </div>
        </GenericWindow>
    );
}

export default Main;
