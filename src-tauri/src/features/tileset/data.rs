use crate::features::tileset::legacy_tileset::FallbackSpriteIndex;
use cdda_lib::types::MeabyVec;
use serde::{Deserialize, Serialize};

pub(super) type MeabyAnimated<T> = MeabyVec<T>;

pub(super) type AnimationFrames<T> = Vec<T>;

pub(super) const FALLBACK_TILE_ROW_SIZE: usize = 16;
pub(super) const FALLBACK_TILE_MAPPING: &'static [(
    &'static str,
    FallbackSpriteIndex,
)] = &[
    // Ignore some textures at the start and end of each color
    (" ", FallbackSpriteIndex(32)),
    ("!", FallbackSpriteIndex(33)),
    ("\"", FallbackSpriteIndex(34)),
    ("#", FallbackSpriteIndex(35)),
    ("$", FallbackSpriteIndex(36)),
    ("%", FallbackSpriteIndex(37)),
    ("&", FallbackSpriteIndex(38)),
    ("(", FallbackSpriteIndex(40)),
    (")", FallbackSpriteIndex(41)),
    ("*", FallbackSpriteIndex(42)),
    ("+", FallbackSpriteIndex(43)),
    (",", FallbackSpriteIndex(44)),
    ("-", FallbackSpriteIndex(45)),
    (".", FallbackSpriteIndex(46)),
    ("/", FallbackSpriteIndex(47)),
    ("0", FallbackSpriteIndex(48)),
    ("1", FallbackSpriteIndex(49)),
    ("2", FallbackSpriteIndex(50)),
    ("3", FallbackSpriteIndex(51)),
    ("4", FallbackSpriteIndex(52)),
    ("5", FallbackSpriteIndex(53)),
    ("6", FallbackSpriteIndex(54)),
    ("7", FallbackSpriteIndex(55)),
    ("8", FallbackSpriteIndex(56)),
    ("9", FallbackSpriteIndex(57)),
    (":", FallbackSpriteIndex(58)),
    (";", FallbackSpriteIndex(59)),
    ("<", FallbackSpriteIndex(60)),
    ("=", FallbackSpriteIndex(61)),
    ("?", FallbackSpriteIndex(62)),
    ("@", FallbackSpriteIndex(63)),
    ("A", FallbackSpriteIndex(64)),
    ("B", FallbackSpriteIndex(65)),
    ("C", FallbackSpriteIndex(66)),
    ("D", FallbackSpriteIndex(67)),
    ("E", FallbackSpriteIndex(68)),
    ("F", FallbackSpriteIndex(69)),
    ("G", FallbackSpriteIndex(70)),
    ("H", FallbackSpriteIndex(71)),
    ("I", FallbackSpriteIndex(72)),
    ("J", FallbackSpriteIndex(73)),
    ("K", FallbackSpriteIndex(74)),
    ("L", FallbackSpriteIndex(75)),
    ("M", FallbackSpriteIndex(76)),
    ("N", FallbackSpriteIndex(77)),
    ("O", FallbackSpriteIndex(78)),
    ("P", FallbackSpriteIndex(79)),
    ("Q", FallbackSpriteIndex(80)),
    ("R", FallbackSpriteIndex(81)),
    ("S", FallbackSpriteIndex(82)),
    ("T", FallbackSpriteIndex(83)),
    ("U", FallbackSpriteIndex(84)),
    ("V", FallbackSpriteIndex(85)),
    ("W", FallbackSpriteIndex(86)),
    ("X", FallbackSpriteIndex(87)),
    ("Y", FallbackSpriteIndex(88)),
    ("Z", FallbackSpriteIndex(89)),
    ("[", FallbackSpriteIndex(90)),
    (r"\", FallbackSpriteIndex(91)),
    ("]", FallbackSpriteIndex(92)),
    ("^", FallbackSpriteIndex(93)),
    ("_", FallbackSpriteIndex(94)),
    ("`", FallbackSpriteIndex(95)),
    ("a", FallbackSpriteIndex(96)),
    ("b", FallbackSpriteIndex(97)),
    ("c", FallbackSpriteIndex(98)),
    ("d", FallbackSpriteIndex(99)),
    ("e", FallbackSpriteIndex(100)),
    ("f", FallbackSpriteIndex(101)),
    ("g", FallbackSpriteIndex(102)),
    ("h", FallbackSpriteIndex(103)),
    ("i", FallbackSpriteIndex(104)),
    ("j", FallbackSpriteIndex(105)),
    ("k", FallbackSpriteIndex(106)),
    ("l", FallbackSpriteIndex(107)),
    ("m", FallbackSpriteIndex(108)),
    ("n", FallbackSpriteIndex(109)),
    ("o", FallbackSpriteIndex(110)),
    ("p", FallbackSpriteIndex(111)),
    ("q", FallbackSpriteIndex(112)),
    ("r", FallbackSpriteIndex(113)),
    ("s", FallbackSpriteIndex(114)),
    ("t", FallbackSpriteIndex(115)),
    ("u", FallbackSpriteIndex(116)),
    ("v", FallbackSpriteIndex(117)),
    ("w", FallbackSpriteIndex(118)),
    ("x", FallbackSpriteIndex(119)),
    ("y", FallbackSpriteIndex(120)),
    ("z", FallbackSpriteIndex(121)),
    ("{", FallbackSpriteIndex(122)),
    ("}", FallbackSpriteIndex(124)),
    ("~", FallbackSpriteIndex(125)),
    ("|", FallbackSpriteIndex(178)),
];

#[derive(Debug, Clone, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub(super) enum AdditionalTileType {
    // TODO: Is this what is meant with intersection?
    #[serde(alias = "center", alias = "intersection")]
    Center,

    #[serde(rename = "corner")]
    Corner,

    #[serde(rename = "t_connection")]
    TConnection,

    #[serde(rename = "edge")]
    Edge,

    #[serde(alias = "end_piece", alias = "end")]
    EndPiece,

    #[serde(rename = "unconnected")]
    Unconnected,

    #[serde(rename = "broken")]
    Broken,

    #[serde(rename = "open")]
    Open,

    // ???
    // BrownLikeBears -> tile_config.json -> Line 5688
    #[serde(rename = "h")]
    H,
}
