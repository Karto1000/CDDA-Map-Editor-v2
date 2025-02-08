import {EventCallback, Event, listen} from "@tauri-apps/api/event";

export type EditorConfig = {
    cdda_path?: string
    selected_tileset?: string
    theme: string
}

export type EditorData = {
    config: EditorConfig
    available_tilesets: string[]
}

export enum EditorDataRecvEvent {
    EditorDataChanged = "editor_data_changed"
}
