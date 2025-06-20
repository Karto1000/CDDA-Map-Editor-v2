import React, {useEffect, useState} from "react";
import {emit, emitTo} from "@tauri-apps/api/event";
import {useTauriEvent} from "./useTauriEvent.js";
import {TauriEvent} from "../../tauri/events/types.js";

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

export function useTheme(): [Theme, React.Dispatch<React.SetStateAction<Theme>>] {
    const [theme, setTheme] = useState<Theme>(Theme.Dark);

    function changeThemeHandler(data: {theme: Theme}) {
        setTheme(data.theme)

        console.log("sending theme change event: ", data.theme, "")

        emit(
            TauriEvent.CHANGED_THEME,
            data.theme
        )
    }

    useTauriEvent(
        TauriEvent.CHANGE_THEME_REQUEST,
        changeThemeHandler,
        [theme]
    )

    useEffect(() => {
        const localTheme = localStorage.getItem("theme");

        if (!localTheme) {
            localStorage.setItem("theme", Theme.Dark.toString());
            return;
        }

        setTheme(localTheme as Theme)
        emit(
            TauriEvent.CHANGED_THEME,
            {theme: localTheme as Theme}
        )
    }, []);

    return [theme, setTheme];
}