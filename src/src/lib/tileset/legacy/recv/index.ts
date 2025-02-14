type TileNew = {
    file: string;
    "//"?: string;
    sprite_width?: number;
    sprite_height?: number;
    sprite_offset_x?: number;
    sprite_offset_y?: number;
};

type TileInfo = {
    pixelscale: number;
    width: number;
    height: number;
    zlevel_height: number;
    iso: boolean;
    retract_dist_min: number;
    retract_dist_max: number;
};

type SpritesheetConfig = {
    'tiles-new': TileNew[];
    tile_info: TileInfo[];
};