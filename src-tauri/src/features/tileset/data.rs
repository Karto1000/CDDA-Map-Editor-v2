use cdda_lib::types::MeabyVec;
use serde::{Deserialize, Serialize};

pub(super) type MeabyAnimated<T> = MeabyVec<T>;

pub(super) const FALLBACK_TILE_ROW_SIZE: usize = 16;
pub(super) const FALLBACK_TILE_MAPPING: &'static [(&'static str, u32)] = &[
    // Ignore some textures at the start and end of each color
    (" ", 32),
    ("!", 33),
    ("\"", 34),
    ("#", 35),
    ("$", 36),
    ("%", 37),
    ("&", 38),
    ("(", 40),
    (")", 41),
    ("*", 42),
    ("+", 43),
    (",", 44),
    ("-", 45),
    (".", 46),
    ("/", 47),
    ("0", 48),
    ("1", 49),
    ("2", 50),
    ("3", 51),
    ("4", 52),
    ("5", 53),
    ("6", 54),
    ("7", 55),
    ("8", 56),
    ("9", 57),
    (":", 58),
    (";", 59),
    ("<", 60),
    ("=", 61),
    ("?", 62),
    ("@", 63),
    ("A", 64),
    ("B", 65),
    ("C", 66),
    ("D", 67),
    ("E", 68),
    ("F", 69),
    ("G", 70),
    ("H", 71),
    ("I", 72),
    ("J", 73),
    ("K", 74),
    ("L", 75),
    ("M", 76),
    ("N", 77),
    ("O", 78),
    ("P", 79),
    ("Q", 80),
    ("R", 81),
    ("S", 82),
    ("T", 83),
    ("U", 84),
    ("V", 85),
    ("W", 86),
    ("X", 87),
    ("Y", 88),
    ("Z", 89),
    ("[", 90),
    (r"\", 91),
    ("]", 92),
    ("^", 93),
    ("_", 94),
    ("`", 95),
    ("a", 96),
    ("b", 97),
    ("c", 98),
    ("d", 99),
    ("e", 100),
    ("f", 101),
    ("g", 102),
    ("h", 103),
    ("i", 104),
    ("j", 105),
    ("k", 106),
    ("l", 107),
    ("m", 108),
    ("n", 109),
    ("o", 110),
    ("p", 111),
    ("q", 112),
    ("r", 113),
    ("s", 114),
    ("t", 115),
    ("u", 116),
    ("v", 117),
    ("w", 118),
    ("x", 119),
    ("y", 120),
    ("z", 121),
    ("{", 122),
    ("}", 124),
    ("~", 125),
    ("|", 178),
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
