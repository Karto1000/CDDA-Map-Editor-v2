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
