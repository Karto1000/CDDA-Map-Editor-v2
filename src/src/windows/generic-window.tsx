import React, {useEffect, useState} from "react";
import "./generic-window.scss"
import "../index.scss"
import {Theme} from "../hooks/useTheme.js";
import Icon, {IconName} from "../components/icon.js";
import {getCurrentWindow} from "@tauri-apps/api/window";
import {listen} from "@tauri-apps/api/event";

export type GenericWindowProps = {
    title: string,
    children: React.ReactNode[] | React.ReactNode,
}

export default function GenericWindow(props: GenericWindowProps) {
    const search = new URLSearchParams(window.location.search)
    const [localTheme, setLocalTheme] = useState<Theme>(search.get("theme") as Theme);


    useEffect(() => {
        // Listen for theme change
        const unlisten = listen<{ theme: Theme }>("theme-changed", e => {
                console.log("Received theme change event: ", e.payload)
                setLocalTheme(e.payload.theme)
            }
        )

        return () => {
            unlisten.then(f => f())
        }
    }, [])

    async function onCloseClick() {
        const window = getCurrentWindow();
        await window.close();
    }

    return (
        <div className={`${localTheme}-theme generic-window`}>
            <div data-tauri-drag-region className={`header`}>
                <h2>{props.title}</h2>
                <button className={"close-button"} onClick={onCloseClick}>
                    <Icon name={IconName.CloseSmall}/>
                </button>
            </div>
            <div className={"window-body"}>
                {props.children}
            </div>
        </div>
    )
}