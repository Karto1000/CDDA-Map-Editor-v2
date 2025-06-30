import React, {useEffect, useState} from "react";
import "./generic-window.scss"
import "../index.scss"
import {getCurrentWindow} from "@tauri-apps/api/window";
import {listen} from "@tauri-apps/api/event";
import {Theme} from "../shared/hooks/useTheme.js";
import Icon, {IconName} from "../shared/components/icon.js";

export type GenericWindowProps = {
    title: string,
    children: React.ReactNode
    hasSearch?: boolean
    hasCloseButton?: boolean
    onCloseClicked?: () => Promise<void>
    onSearchQueryChanged?: (query: string) => void
    searchQuery?: string
}

export const THEME_CHANGED = "theme-changed"

export default function GenericWindow(
    {
        title,
        children,
        hasSearch = false,
        hasCloseButton = true,
        onCloseClicked = async () => {
        },
        onSearchQueryChanged = async (query: string) => {
        },
        searchQuery
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
        await window.destroy();
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
            {
                hasSearch &&
                <div className={"search-container"}>
                    <input placeholder={"Search"} value={searchQuery}
                           onChange={e => onSearchQueryChanged(e.target.value)}/>
                </div>
            }
            <div className={"window-body"}>
                {children}
            </div>
        </div>
    )
}