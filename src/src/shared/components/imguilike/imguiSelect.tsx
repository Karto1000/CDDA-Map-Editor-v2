import React, {useContext} from "react";
import Select from "react-select";
import {getColorFromTheme} from "../../hooks/useTheme.js";
import {ThemeContext} from "../../../app.js";

export type ImguiSelectOption = {
    label: string,
    value: string,
}

export type ImguiSelectProps = {
    options: ImguiSelectOption[],
    onChange: (value: string) => void,
}

export function ImguiSelect(props: ImguiSelectProps) {
    const theme = useContext(ThemeContext)

    return (
        <Select
            options={props.options}
            onChange={(value: { value: string; }) => props.onChange(value.value)}
            styles={{
                container: (base) => (
                    {
                        ...base,
                        flex: 1
                    }
                ),
                control: (base, state) => (
                    {
                        ...base,
                        backgroundColor: state.isFocused ? getColorFromTheme(theme.theme, "selected") :
                            getColorFromTheme(theme.theme, "darkestBlue"),
                        "&:hover": {
                            backgroundColor: state.isFocused ? getColorFromTheme(theme.theme, "selected") :
                                getColorFromTheme(theme.theme, "darkBlue")
                        },
                        outline: "none",
                        borderRadius: 0,
                        border: "none",
                    }
                ),
                dropdownIndicator: (base) => (
                    {
                        ...base,
                        color: getColorFromTheme(theme.theme, "lightest"),
                        "&:hover": {
                            color: getColorFromTheme(theme.theme, "lightest"),
                        }
                    }
                ),
                option: (base) => (
                    {
                        ...base,
                        "&:hover": {
                            backgroundColor: getColorFromTheme(theme.theme, "darkBlue"),
                        },
                        backgroundColor: getColorFromTheme(theme.theme, "darkestBlue"),
                    }
                ),
                menu: (base) => {
                    return {
                        ...base,
                        backgroundColor: getColorFromTheme(theme.theme, "darkestBlue"),
                        borderRadius: 0,
                        border: "none",
                    }
                },
            }}
        />
    )
}