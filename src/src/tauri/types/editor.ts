export enum ProjectTypeKind {
    MapEditor = "MapEditor",
    LiveViewer = "LiveViewer"
}

type ProjectType =
    | { type: ProjectTypeKind.MapEditor; data: ProjectSaveState }
    | { type: ProjectTypeKind.LiveViewer; data: LiveViewerData };

interface LiveViewerData {
    path: string;
    om_terrain: string;
}

type ProjectSaveState =
    | { state: "Unsaved" }
    | { state: "Saved"; path: string };

export enum KeybindAction {
    NewProject = "NewProject",
    OpenProject = "OpenProject",
    SaveProject = "SaveProject",
    CloseTab = "CloseTab",
    CloseAllTabs = "CloseAllTabs",
    ImportMap = "ImportMap",
    ExportMap = "ExportMap",
    OpenSettings = "OpenSettings",
    Undo = "Undo",
    Redo = "Redo",
    Copy = "Copy",
    Paste = "Paste",
    Draw = "Draw",
    Fill = "Fill",
    Erase = "Erase",
    ReloadMap = "ReloadMap",
}

export type Keybind = {
    action: KeybindAction | null,
    key: string,
    withCtrl?: boolean,
    withShift?: boolean,
    withAlt?: boolean,
    isGlobal?: boolean,
}

export function getKeybindingText(kb: Keybind): string {
    return `${kb.withCtrl ? "Ctrl+" : ""}${kb.withAlt ? "Alt+" : ""}${kb.withShift ? "Shift+" : ""}${kb.key}`
}

export type ProgramConfig = {
    cdda_path?: string
    selected_tileset?: string
    theme: string
    config_path: string,
    keybinds: Keybind[]
}

export type ProgramData = {
    config: ProgramConfig
    available_tilesets: string[] | null,
    openable_projects: { [name: string]: { path: string } },
    recent_projects: { [name: string]: { path: string } },
    opened_project: number | null
}