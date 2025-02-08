export type EditorConfig = {
    cdda_path?: string
    selected_tileset?: string
    theme: string
}

export type EditorData = {
    config: EditorConfig
    available_tilesets: string[]
}