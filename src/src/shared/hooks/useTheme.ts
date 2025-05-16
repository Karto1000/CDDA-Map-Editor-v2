import React, {MutableRefObject, useEffect, useState} from "react";
import {emitTo} from "@tauri-apps/api/event";
import {ChangedThemeEvent, ChangeThemeRequestEvent, LocalEvent} from "../utils/localEvent.js";

export const darkColors = {
    light: "#282828",
    dark: "#1E1E1E",
    darker: "#0F0F0F",
    lightBlue: "#86C2F4",
    darkBlue: "#23416E",
    darkestBlue: "#1E2942",
    selected: "#3B8BF3",
    delete: "#CA3336",
    lightDelete: "#ed4e51",
    lightest: "#FFFFFF",
    disabled: "#868585"
}

export const lightColors = {
    light: "#8d8d8d",
    dark: "#eeeeee",
    darker: "#f8f8f8",
    lightBlue: "#86C2F4",
    darkBlue: "#86C2F4",
    darkestBlue: "#1E2942",
    selected: "#3B8BF3",
    delete: "#CA3336",
    lightDelete: "#ed4e51",
    lightest: "#000000",
    disabled: "#868585",
}

export enum Theme {
    Dark = "dark",
    Light = "light"
}

export const getColorFromTheme = (theme: Theme, color: string): string => {
    if (theme === Theme.Dark) {
        return darkColors[color]
    } else if (theme === Theme.Light) {
        return lightColors[color]
    }
}

export function useTheme(eventBus: MutableRefObject<EventTarget>): [Theme, React.Dispatch<React.SetStateAction<Theme>>] {
    const [theme, setTheme] = useState<Theme>(Theme.Dark);

    useEffect(() => {
        const changeThemeHandler = (d: ChangeThemeRequestEvent) => {
            setTheme(d.detail.theme)
            console.log("sending theme change event: ", d.detail.theme, "")
            eventBus.current.dispatchEvent(
                new ChangedThemeEvent(
                    LocalEvent.CHANGED_THEME,
                    {detail: {theme: d.detail.theme}}
                )
            )
        }

        eventBus.current.addEventListener(
            LocalEvent.CHANGE_THEME_REQUEST,
            changeThemeHandler
        )

        return () => {
            eventBus.current.removeEventListener(
                LocalEvent.CHANGE_THEME_REQUEST,
                changeThemeHandler
            )
        }
    }, [theme, eventBus]);

    useEffect(() => {
        const localTheme = localStorage.getItem("theme");

        if (!localTheme) {
            localStorage.setItem("theme", Theme.Dark.toString());
            return;
        }

        setTheme(localTheme as Theme)
        eventBus.current.dispatchEvent(
            new ChangedThemeEvent(
                LocalEvent.CHANGED_THEME,
                {detail: {theme: localTheme as Theme}})
        )
    }, [eventBus]);

    useEffect(() => {
        localStorage.setItem("theme", theme.toString());
        emitTo("setting", "theme-changed", {theme: theme.toString()});
    }, [theme]);

    return [theme, setTheme];
}