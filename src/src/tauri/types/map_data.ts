export type CDDAIdentifier = string;

export type TilesheetCDDAId = {
    id: string,
    prefix: string,
    postfix: string
}

export type MappedCDDAId = {
    tilesheet_id: TilesheetCDDAId
}


export type MapData = {
    palettes: any
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