import React, {useEffect, useState, JSX} from "react";
import "./generic-window.scss"
import "../index.scss"
import {getCurrentWindow} from "@tauri-apps/api/window";
import {emitTo, listen} from "@tauri-apps/api/event";
import {Theme} from "../shared/hooks/useTheme.js";
import Icon, {IconName} from "../shared/components/icon.js";

export type GenericWindowProps = {
    title: string,
    children: React.ReactNode
    hasCloseButton?: boolean
    onCloseClicked?: () => Promise<void>
}

export const WINDOW_CLOSED = "window-closed"
export const THEME_CHANGED = "theme-changed"

export default function GenericWindow(
    {
        title,
        children,
        hasCloseButton = true,
        onCloseClicked = async () => {}
    }: GenericWindowProps
) {
    const search = new URLSearchParams(window.location.search)
    const [localTheme, setLocalTheme] = useState<Theme>(search.get("theme") as Theme);


    useEffect(() => {
        // Listen for theme change
        const unlisten = listen<{ theme: Theme }>(THEME_CHANGED, e => {
                console.log("Received theme change event: ", e.payload)
                setLocalTheme(e.payload.theme)
            }
        )

        return () => {
            unlisten.then(f => f())
        }
    }, [])

    async function onCloseClick() {
        await onCloseClicked();

        const window = getCurrentWindow();

        await emitTo(window.label, WINDOW_CLOSED)

        await window.close();
    }

    return (
        <div className={`${localTheme}-theme generic-window`}>
            <div data-tauri-drag-region className={`header`}>
                <h2>{title}</h2>
                {
                    hasCloseButton &&
                    <button className={"close-button"} onClick={onCloseClick}>
                        <Icon name={IconName.CloseSmall}/>
                    </button>
                }
            </div>
            <div className={"window-body"}>
                {children}
            </div>
        </div>
    )
}