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

export type TilesheetCDDAId = {
    id: string,
    prefix: string,
    postfix: string
}

export type MappedCDDAId = {
    tilesheet_id: TilesheetCDDAId
}

export type CellData = {
    [zLevel: number]: {
        [position: string]: {
            terrain?: MappedCDDAId,
            furniture?: MappedCDDAId,
            monster?: MappedCDDAId,
            field?: MappedCDDAId
        }
    }
}

// export type CellData = {
//     [coords: string]: {
//         terrain: string | null,
//         furniture: {
//             selectedSign: DisplaySign | null,
//             selectedFurniture: string | null,
//             selectedComputer: unknown | null,
//             selectedGaspump: string | null
//         },
//         itemGroups: DisplayItemGroup[] | null
//     }
// }

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