import {Tab, TabTypeKind} from "../hooks/useTabs.js";
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
}

export interface LocalEventsMap {
    [LocalEvent.CHANGED_THEME]: { theme: Theme}
    [LocalEvent.ADD_LOCAL_TAB]: Tab,
    [LocalEvent.REMOVE_LOCAL_TAB]: { name: string }
    [LocalEvent.OPEN_LOCAL_TAB]: { name: string }
    [LocalEvent.CLOSE_LOCAL_TAB]: { name: string }
    [LocalEvent.TILESET_LOADED]: Tilesheets
    [LocalEvent.CHANGE_THEME_REQUEST]: { theme: Theme }
}

export class ChangedThemeEvent extends CustomEvent<LocalEventsMap[LocalEvent.CHANGED_THEME]> {}
export class AddLocalTabEvent extends CustomEvent<LocalEventsMap[LocalEvent.ADD_LOCAL_TAB]> {}
export class RemoveLocalTabEvent extends CustomEvent<LocalEventsMap[LocalEvent.REMOVE_LOCAL_TAB]> {}
export class OpenLocalTabEvent extends CustomEvent<LocalEventsMap[LocalEvent.OPEN_LOCAL_TAB]> {}
export class CloseLocalTabEvent extends CustomEvent<LocalEventsMap[LocalEvent.CLOSE_LOCAL_TAB]> {}
export class TilesetLoadedEvent extends CustomEvent<LocalEventsMap[LocalEvent.TILESET_LOADED]> {}
export class ChangeThemeRequestEvent extends CustomEvent<LocalEventsMap[LocalEvent.CHANGE_THEME_REQUEST]> {}