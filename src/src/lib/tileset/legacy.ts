export type TileNew = {
    file: string;
    "//"?: [number, number];
    sprite_width?: number;
    sprite_height?: number;
    sprite_offset_x?: number;
    sprite_offset_y?: number;
};
export type TileInfo = {
    pixelscale: number;
    width: number;
    height: number;
    zlevel_height: number;
    iso: boolean;
    retract_dist_min: number;
    retract_dist_max: number;
};
export type SpritesheetConfig = {
    'tiles-new': TileNew[];
    tile_info: TileInfo[];
};

export enum LegacyTilesetCommand {
    DownloadSpritesheet = "download_spritesheet",
    GetInfoOfCurrentTileset = "get_info_of_current_tileset"
}