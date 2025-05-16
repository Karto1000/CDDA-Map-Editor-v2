import {AnimatedSprite, FallbackSprite, StaticSprite} from "../types/map_data.js";
import {EditorData} from "../types/editor.js";
import {TabTypeKind} from "../../shared/hooks/useTabs.js";
import {Vector2, Vector3} from "three";

export function serializedVec2ToVector2(serializedVec2: string): Vector2 {
    const parts = serializedVec2.split(",")

    const x = parseInt(parts[0])
    const y = parseInt(parts[1])

    return new Vector2(x, y)
}

export function serializedVec3ToVector3(serializedVec3: string): Vector3 {
    const parts = serializedVec3.split(",")

    const x = parseInt(parts[0])
    const y = parseInt(parts[1])
    const z = parseInt(parts[2])

    return new Vector3(x, y, z)
}

export enum BackendResponseType {
    Error,
    Success
}

export type BackendResponse<T, E> = {
    type: BackendResponseType.Error,
    error: E
} | {
    type: BackendResponseType.Success,
    data: T
}

export enum TauriCommand {
    GET_EDITOR_DATA = "get_editor_data",
    CDDA_INSTALLATION_DIRECTORY_PICKED = "cdda_installation_directory_picked",
    TILESET_PICKED = "tileset_picked",
    SAVE_EDITOR_DATA = "save_editor_data",
    GET_CURRENT_PROJECT_DATA = "get_current_project_data",
    GET_SPRITES = "get_sprites",
    RELOAD_PROJECT = "reload_project",
    OPEN_PROJECT = "open_project",
    CLOSE_PROJECT = "close_project",
    GET_PROJECT_CELL_DATA = "get_project_cell_data",
    OPEN_VIEWER = "open_viewer",
    GET_INFO_OF_CURRENT_TILESET = "get_info_of_current_tileset",
    DOWNLOAD_SPRITESHEET = "download_spritesheet",
    FRONTEND_READY = "frontend_ready",
}

export interface TauriCommandMap {
    [TauriCommand.GET_EDITOR_DATA]: {};
    [TauriCommand.CDDA_INSTALLATION_DIRECTORY_PICKED]: {
        path: string,
    };
    [TauriCommand.TILESET_PICKED]: {
        tileset: string
    };
    [TauriCommand.SAVE_EDITOR_DATA]: {};
    [TauriCommand.GET_CURRENT_PROJECT_DATA]: {};
    [TauriCommand.GET_SPRITES]: {
        name: string
    };
    [TauriCommand.RELOAD_PROJECT]: {};
    [TauriCommand.OPEN_PROJECT]: {
        name: string
    };
    [TauriCommand.CLOSE_PROJECT]: {
        name: string
    };
    [TauriCommand.GET_PROJECT_CELL_DATA]: {};
    [TauriCommand.OPEN_VIEWER]: {
        data: {
            filePath: string,
            projectName: string,
            omTerrain: {
                type: "Single",
                omTerrainId: string
            } | {
                type: "Nested",
                omTerrainIds: string[][]
            }
        }
    };
    [TauriCommand.GET_INFO_OF_CURRENT_TILESET]: {};
    [TauriCommand.DOWNLOAD_SPRITESHEET]: {
        name: string
    };
    [TauriCommand.FRONTEND_READY]: {};
}

export enum TauriEvent {
    EDITOR_DATA_CHANGED = "editor_data_changed",
    PLACE_SPRITES = "place_sprites",
    TAB_CREATED = "tab_created",
    TAB_REMOVED = "tab_removed",
    UPDATE_LIVE_VIEWER = "update_live_viewer",
    CHANGE_THEME = "change_theme"
}

export interface TauriEventMap {
    [TauriEvent.EDITOR_DATA_CHANGED]: EditorData;
    [TauriEvent.PLACE_SPRITES]: {
        static_sprites: StaticSprite[];
        animated_sprites: AnimatedSprite[];
        fallback_sprites: FallbackSprite[];
    };
    [TauriEvent.TAB_CREATED]: {
        name: string,
        tab_type: TabTypeKind,
    };
    [TauriEvent.TAB_REMOVED]: {
        name: string
    };
    [TauriEvent.UPDATE_LIVE_VIEWER]: {};
}
