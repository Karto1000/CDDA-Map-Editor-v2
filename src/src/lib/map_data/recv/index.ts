export type Identifier = string;
export type MeabyVec<T> = { Vec: T[] } | { Single: T };
export type MeabyWeighted<T> = { MultiWeighted: { weight: number, sprite: T } } | { NotWeighted: T };
export type ConnectionType =
    | "center"
    | "corner"
    | "t_connection"
    | "edge"
    | "end_piece"
    | "broken"
    | "unconnected"
    | "open";
export type AdditionalTile = {
    id: ConnectionType;
    fg?: MeabyVec<MeabyWeighted<MeabyVec<number>>>;
    bg?: MeabyVec<MeabyWeighted<MeabyVec<number>>>;
};
export type SpritesheetTile = {
    id: MeabyVec<Identifier>;
    fg?: MeabyVec<MeabyWeighted<number>>;
    bg?: MeabyVec<MeabyWeighted<number>>;
    rotates?: boolean;
    animated?: boolean;
    multitile?: boolean;
    additional_tiles?: AdditionalTile[];
};
export type TilesetInfo = {
    pixelscale: number;
    width: number;
    height: number;
    zlevel_height: number;
    iso: boolean;
    retract_dist_min: number;
    retract_dist_max: number;
};
export type TilesetTiles = {
    file: string;
    spritesheet_dimensions: [number, number],
    sprite_width: number
    sprite_height: number
    tiles: SpritesheetTile[];
};
export type TilesetConfig = {
    tile_info: TilesetInfo[];
    "tiles-new": TilesetTiles[];
};

export type PlaceTerrainEvent = {
    position: string
    identifier: string
}