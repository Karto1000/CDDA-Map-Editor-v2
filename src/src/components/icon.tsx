//===================================================================
// Import References 
//===================================================================
import React from "react";

//===================================================================
// Constant Variables Definitions
//===================================================================

//===================================================================
// Export Type Definitions
//===================================================================

//===================================================================
// Local Type Definitions
//===================================================================
type Props = {
    name: IconName,
    width?: number,
    height?: number
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
        width = 32,
        height = 32
    }: Props
): React.JSX.Element {
    return <img width={width} height={height} src={`/icons/${name}.png`} alt={name}/>
}

//===================================================================
// Exports
//===================================================================