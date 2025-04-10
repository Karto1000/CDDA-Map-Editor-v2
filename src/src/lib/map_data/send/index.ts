export enum MapChangeEventKind {
    Place = "Place",
    Delete = "Delete"
}

export type PlaceSpriteCommand = {
    position: string
    index: number
}

export type PlaceSpritesCommand = {
    positions: string[]
    indexes: number[]
    sprite_layers: number[]
}


export type DeleteCommand = string;

export type MapChangeEvent = {
    kind: {
        [key in MapChangeEventKind]: PlaceSpriteCommand
    }
}

export enum MapDataSendCommand {
    Place = "place",
    CreateMap = "create_map",
    OpenMap = "open_map",
    CloseMap = "close_map",
    GetCurrentMapData = "get_current_map_data",
    SaveCurrentMap = "save_current_map"
}