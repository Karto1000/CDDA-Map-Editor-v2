export enum MapChangeEventKind {
    Place = "Place",
    Delete = "Delete"
}

export type PlaceCommand = {
    position: string
    character: string
}

export type DeleteCommand = string;

export type MapChangeEvent = {
    kind: {
        [key in MapChangeEventKind]: PlaceCommand
    }
}

export enum MapDataSendCommand {
    Place = "place",
    CreateMap = "create_map",
    OpenMap = "open_map",
    CloseMap = "close_map",
    GetCurrentMapData = "get_current_map_data"
}