use crate::cdda_data::Distribution;
use crate::util::CDDAIdentifier;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
pub struct RegionIdentifier(pub String);

#[derive(Debug, Clone, Deserialize)]
pub struct RegionTerrainAndFurniture {
    pub terrain: HashMap<RegionIdentifier, HashMap<CDDAIdentifier, i32>>,
    pub furniture: HashMap<RegionIdentifier, HashMap<CDDAIdentifier, i32>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OvermapTerrainAlias {
    pub om_terrain: String,
    pub om_terrain_match_type: String,
    pub alias: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OvermapLakeSettings {
    pub noise_threshold_lake: f32,
    pub lake_size_min: i32,
    pub lake_depth: i32,
    pub shore_extendable_overmap_terrain: Vec<String>,
    pub shore_extendable_overmap_terrain_aliases: Vec<OvermapTerrainAlias>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OvermapOceanSettings {
    pub noise_threshold_ocean: f32,
    pub ocean_size_min: i32,
    pub ocean_depth: i32,
    pub ocean_start_north: i32,
    pub ocean_start_east: i32,
    pub ocean_start_west: i32,
    pub ocean_start_south: i32,
    pub sandy_beach_width: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OvermapRavineSettings {
    pub num_ravines: i32,
    pub ravine_width: i32,
    pub ravine_range: i32,
    pub ravine_depth: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OvermapForestSettings {
    pub noise_threshold_forest: f32,
    pub noise_threshold_forest_thick: f32,
    pub noise_threshold_swamp_adjacent_water: f32,
    pub noise_threshold_swamp_isolated: f32,
    pub river_floodplain_buffer_distance_min: i32,
    pub river_floodplain_buffer_distance_max: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OvermapConnectionSettings {
    pub intra_city_road_connection: String,
    pub inter_city_road_connection: String,
    pub trail_connection: String,
    pub sewer_connection: String,
    pub subway_connection: String,
    pub rail_connection: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForestTrailSettings {
    pub chance: i32,
    pub border_point_chance: i32,
    pub minimum_forest_size: i32,
    pub random_point_min: i32,
    pub random_point_max: i32,
    pub random_point_size_scalar: i32,
    pub trailhead_chance: i32,
    pub trailhead_road_distance: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapExtras {
    pub forest: HashMap<String, i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CitySettings {
    pub shop_radius: i32,
    pub shop_sigma: i32,
    pub park_radius: i32,
    pub park_sigma: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherSettings {
    pub base_temperature: f32,
    pub base_humidity: f32,
    pub base_pressure: f32,
    pub base_wind: f32,
    pub base_wind_distrib_peaks: i32,
    pub base_wind_season_variation: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OvermapFeatureFlagSettings {
    pub clear_blacklist: bool,
    pub blacklist: Vec<String>,
    pub clear_whitelist: bool,
    pub whitelist: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CDDARegionSettings {
    pub id: CDDAIdentifier,
    pub default_oter: Vec<String>,
    pub default_groundcover: Vec<Distribution>,
    pub region_terrain_and_furniture: RegionTerrainAndFurniture,
    pub river_scale: f32,
}
