import {useEffect, useState} from "react";

export enum Theme {
    Dark = "dark",
    Light = "light"
}

export function useTheme() {
    const [theme, setTheme] = useState<Theme>(Theme.Dark);

    useEffect(() => {
        const localTheme = localStorage.getItem("theme");

        if (!localTheme) {
            localStorage.setItem("theme", Theme.Dark.toString());
            return;
        }

        setTheme(localTheme as Theme)
    }, []);

    function setThemeWrapper(theme: Theme): void {
        localStorage.setItem("theme", theme.toString());
        setTheme(theme);
    }

    return [theme, setThemeWrapper];
}