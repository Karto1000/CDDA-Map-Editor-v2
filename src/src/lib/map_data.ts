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

export enum MapDataEvent {
  OpenedMap = "opened_map",
  PlaceSprites = "place_sprites",
  ItemData = "item_data"
}

export type MapGenItem = {
  item: string,
  // TODO: Add other fields
}

export type ItemDataEvent = {
  [coordinates: string]: MapGenItem[]
}

export type PlaceSpriteCommand = {
  position: string
  index: number
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

export type PlaceSpritesEvent = {
  static_sprites: StaticSprite[]
  animated_sprites: AnimatedSprite[]
  fallback_sprites: FallbackSprite[]
}


export enum MapDataSendCommand {
  CreateProject = "create_project",
  OpenProject = "open_project",
  CloseProject = "close_project",
  GetCurrentProjectData = "get_current_map_data",
  SaveCurrentProject = "save_current_project"
}