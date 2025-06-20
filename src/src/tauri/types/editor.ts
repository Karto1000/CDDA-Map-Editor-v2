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

export type EditorConfig = {
    cdda_path?: string
    selected_tileset?: string
    theme: string
}
export type EditorData = {
    config: EditorConfig
    openable_projects: string[],
    available_tilesets: string[] | null,
    recent_projects: {name: string, path: string}[],
    opened_project: number | null
}