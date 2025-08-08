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

export type Furniture = {
    id: string
}

export type Palettes = {[id: string]: Palette}

export type TerrainData = {[id: string]: Terrain}
export type FurnitureData = {[id: string]: Furniture}

export enum TileLayer {
    Terrain = "Terrain",
    Furniture = "Furniture",
    Monster = "Monster",
    Field = "Field"
}