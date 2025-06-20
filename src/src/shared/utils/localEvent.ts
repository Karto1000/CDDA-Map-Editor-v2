import {Tab} from "../hooks/useTabs.js";
import {Theme} from "../hooks/useTheme.js";
import {Tilesheets} from "../../features/sprites/tilesheets.js";

export enum LocalEvent {
    CHANGED_THEME = "change-theme",
    CHANGE_THEME_REQUEST = "change-theme-request",
    ADD_LOCAL_TAB = "add-local-tab",
    REMOVE_LOCAL_TAB = "remove-local-tab",
    OPEN_LOCAL_TAB = "open-local-tab",
    CLOSE_LOCAL_TAB = "close-local-tab",
    TILESET_LOADED = "tileset-loaded",
    CHANGE_Z_LEVEL = "change-z-level",
    CHANGE_WORLD_MOUSE_POSITION = "change-world-mouse-position",
    CHANGE_SELECTED_POSITION = "change-selected-position",
    UPDATE_VIEWER = "update-viewer",
    TOGGLE_GRID = "toggle-grid",
    OPEN_MAPGEN_INFO_WINDOW = "open-mapgen-info-window",
    OPEN_PALETTES_WINDOW = "open-palettes-window",
}

export interface LocalEventsMap {
    [LocalEvent.CHANGED_THEME]: { theme: Theme }
    [LocalEvent.ADD_LOCAL_TAB]: Tab,
    [LocalEvent.REMOVE_LOCAL_TAB]: { name: string }
    [LocalEvent.OPEN_LOCAL_TAB]: { name: string }
    [LocalEvent.CLOSE_LOCAL_TAB]: { name: string }
    [LocalEvent.TILESET_LOADED]: Tilesheets
    [LocalEvent.CHANGE_THEME_REQUEST]: { theme: Theme }
    [LocalEvent.CHANGE_Z_LEVEL]: { zLevel: number }
    [LocalEvent.CHANGE_WORLD_MOUSE_POSITION]: { position: { x: number, y: number } }
    [LocalEvent.CHANGE_SELECTED_POSITION]: { position?: { x: number, y: number } }
    [LocalEvent.UPDATE_VIEWER]: {  },
    [LocalEvent.TOGGLE_GRID]: { state: boolean },
    [LocalEvent.OPEN_MAPGEN_INFO_WINDOW]: {},
    [LocalEvent.OPEN_PALETTES_WINDOW]: {}
}

export class ChangedThemeEvent extends CustomEvent<LocalEventsMap[LocalEvent.CHANGED_THEME]> {
}

export class AddLocalTabEvent extends CustomEvent<LocalEventsMap[LocalEvent.ADD_LOCAL_TAB]> {
}

export class RemoveLocalTabEvent extends CustomEvent<LocalEventsMap[LocalEvent.REMOVE_LOCAL_TAB]> {
}

export class OpenLocalTabEvent extends CustomEvent<LocalEventsMap[LocalEvent.OPEN_LOCAL_TAB]> {
}

export class CloseLocalTabEvent extends CustomEvent<LocalEventsMap[LocalEvent.CLOSE_LOCAL_TAB]> {
}

export class TilesetLoadedEvent extends CustomEvent<LocalEventsMap[LocalEvent.TILESET_LOADED]> {
}

export class ChangeThemeRequestEvent extends CustomEvent<LocalEventsMap[LocalEvent.CHANGE_THEME_REQUEST]> {
}

export class UpdateViewerEvent extends CustomEvent<LocalEventsMap[LocalEvent.UPDATE_VIEWER]> {}
export class ToggleGridEvent extends CustomEvent<LocalEventsMap[LocalEvent.TOGGLE_GRID]> {}
export class OpenMapgenInfoWindowEvent extends CustomEvent<LocalEventsMap[LocalEvent.OPEN_MAPGEN_INFO_WINDOW]> {}
export class OpenPalettesWindowEvent extends CustomEvent<LocalEventsMap[LocalEvent.OPEN_PALETTES_WINDOW]> {}