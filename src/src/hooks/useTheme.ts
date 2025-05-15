import React, {useEffect, useState} from "react";
import {Vector3} from "three";
import {emitTo} from "@tauri-apps/api/event";

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

export function hexToRBGNormalized(hex: string): Vector3 {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    const r = parseInt(result[1], 16) / 255
    const g = parseInt(result[2], 16) / 255
    const b = parseInt(result[3], 16) / 255
    return new Vector3(r, g, b)
}

export function useTheme(): [Theme, React.Dispatch<React.SetStateAction<Theme>>] {
    const [theme, setTheme] = useState<Theme>(Theme.Dark);

    useEffect(() => {
        const localTheme = localStorage.getItem("theme");

        if (!localTheme) {
            localStorage.setItem("theme", Theme.Dark.toString());
            return;
        }

        setTheme(localTheme as Theme)
    }, []);

    useEffect(() => {
        localStorage.setItem("theme", theme.toString());
        emitTo("setting", "theme-changed", {theme: theme.toString()});
    }, [theme]);

    return [theme, setTheme];
}