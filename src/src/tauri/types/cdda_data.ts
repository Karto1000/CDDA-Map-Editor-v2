export enum CDDADataField {
    PALETTES = "palettes",
    TERRAIN = "terrain",
    FURNITURE = "furniture"
}

export type Palette = {
    id: string
}

export type Terrain = {
    id: string
}

export type Palettes = {[id: string]: Palette}

// TODO: I know this is not valid english but shudup
export type Terrains = {[id: string]: Terrain}