export type SlimTilesheet = {
    range: [number, number] | null
    objectURL: string
}

export type SlimTilesheets = {
    tilesheets: { [name: string]: SlimTilesheet }
    fallback: SlimTilesheet
}