//===================================================================
// Import References
//===================================================================
import React, {useContext} from "react";
import {Theme} from "../hooks/useTheme.ts";
import {ThemeContext} from "../../app.js";

//===================================================================
// Constant Variables Definitions
//===================================================================

//===================================================================
// Export Type Definitions
//===================================================================

//===================================================================
// Local Type Definitions
//===================================================================
export type IconProps = {
    name: IconName,
    width?: number,
    height?: number,
    rotation?: number
    pointerEvents?: string
}

//===================================================================
// Class Definitions
//===================================================================
export enum IconName {
    AddSmall = "add-small",
    PenMedium = "brush-medium",
    ChevronDownSmall = "chevron-down-small",
    ChevronUpSmall = "chevron-up-small",
    CloseSmall = "close-small",
    CursorSmall = "cursor-small",
    DeleteSmall = "delete-small",
    ExportSmall = "export-small",
    HideSmall = "hide-small",
    ImportSmall = "import-small",
    OpenSmall = "open-small",
    PaintBucketSmall = "paint-bucket-small",
    SaveSmall = "save-small",
    SettingsSmall = "settings-small",
    ShapesSmall = "shapes-small",
    WindowedSmall = "windowed-small",
    EyeMedium = "eye-medium",
    CheckmarkMedium = "checkmark-medium",
    QuestionSmall = "question-small",
    ReloadMedium = "reload-medium",
    ErrorMedium = "error-medium",
    InfoMedium = "info-medium"
}

//===================================================================
// Function Definitions
//===================================================================

//===================================================================
// Component Definition
//===================================================================
export default function Icon(
    {
        name,
        width = 14,
        height = 14,
        rotation = 0,
        pointerEvents = "auto"
    }: IconProps
): React.JSX.Element {
    const {theme} = useContext(ThemeContext);

    const imgFilter = theme === Theme.Dark ? "invert(0%)" : "invert(100%)";
    return <img
        style={{rotate: `${rotation}deg`, filter: imgFilter, pointerEvents}}
        width={width}
        className={"icon"}
        height={height}
        src={`/icons/${name}.png`}
        alt={name}
    />
}

//===================================================================
// Exports
//===================================================================