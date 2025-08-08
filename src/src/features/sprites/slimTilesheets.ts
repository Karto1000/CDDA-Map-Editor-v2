import {FallbackSheet} from "../../tauri/types/spritesheet.js";

export type SlimTilesheet = {
    range: [number, number] | null
    objectURL: string
}

export type SlimFallbackTilesheet = FallbackSheet & {objectURL: string};

export type SlimTilesheets = {
    tilesheets: { [name: string]: SlimTilesheet }
    fallback: SlimFallbackTilesheet
}