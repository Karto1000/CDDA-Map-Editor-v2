export type CDDAIdentifier = string;

export enum MapDataEvent {
    OpenedMap = "opened_map",
    PlaceSprites = "place_sprites",
    ItemData = "item_data"
}

export type CellData = {
    item: string,
    computers: string,
    signs: string
    // TODO: Add other fields
}

export type ItemDataEvent = {
    [coordinates: string]: CellData[]
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

export enum MapDataSendCommand {
    CreateProject = "create_project",
    OpenProject = "open_project",
    CloseProject = "close_project",
    GetCurrentProjectData = "get_current_project_data",
    SaveCurrentProject = "save_current_project",
    GetProjectCellData = "get_project_cell_data"
}