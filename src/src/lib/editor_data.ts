import {Tab} from "../hooks/useTabs.js";

export enum EditorDataRecvEvent {
  EditorDataChanged = "editor_data_changed",
  TabCreated = "tab_created",
  TabClosed = "tab_closed"
}

export type EditorConfig = {
  cdda_path?: string
  selected_tileset?: string
  theme: string
}
export type EditorData = {
  config: EditorConfig
  available_tilesets: string[],
  tabs: Tab[]
}

export enum EditorDataSendCommand {
  GetEditorData = "get_editor_data",
  SaveEditorData = "save_editor_data",
  CDDAInstallationDirectoryPicked = "cdda_installation_directory_picked",
  TilesetPicked = "tileset_picked",
  CreateTab = "create_tab",
  CloseTab = "close_tab"
}