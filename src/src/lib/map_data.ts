export type CDDAIdentifier = string;

export enum MapDataEvent {
    PlaceSprites = "place_sprites",
    UpdateLiveViewer = "update_live_viewer",
}

export type DisplayItemGroup = {
    type: DisplayItemGroupType.Single,
    item: CDDAIdentifier
    probability: number
} | {
    type: DisplayItemGroupType.Collection
    name: string
    items: DisplayItemGroup[]
    probability: number
} | {
    type: DisplayItemGroupType.Distribution
    name: string
    items: DisplayItemGroup[]
    probability: number
}

export type DisplaySign = {
    signage: string
    snippet: string
}

export type CellData = {
    [coords: string]: { item_groups: DisplayItemGroup[], signs: DisplaySign }
}

export type StaticSprite = {
    position: string
    index: number
    layer: number
    rotate_deg: number
    z: number
}

export type AnimatedSprite = {
    position: string
    indices: number[],
    layer: number
    rotate_deg: number
    z: number,
}

export type FallbackSprite = {
    position: string,
    index: number
    z: number
}

export type PlaceSpritesEvent = {
    static_sprites: StaticSprite[]
    animated_sprites: AnimatedSprite[]
    fallback_sprites: FallbackSprite[]
}

export enum DisplayItemGroupType {
    Single = "Single",
    Collection = "Collection",
    Distribution = "Distribution"
}


export enum MapDataSendCommand {
    CreateProject = "create_project",
    OpenProject = "open_project",
    CloseProject = "close_project",
    GetCurrentProjectData = "get_current_project_data",
    SaveCurrentProject = "save_current_project",
    GetProjectCellData = "get_project_cell_data",
    GetSprites = "get_sprites",
    ReloadProject = "reload_project"
}