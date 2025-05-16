export type CDDAIdentifier = string;

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

export enum DisplayItemGroupType {
    Single = "Single",
    Collection = "Collection",
    Distribution = "Distribution"
}