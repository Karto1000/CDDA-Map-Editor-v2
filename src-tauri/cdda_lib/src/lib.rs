use std::string::ToString;

pub mod types;

pub const NULL_TERRAIN: &'static str = "t_null";
pub const NULL_FURNITURE: &'static str = "f_null";
pub const NULL_NESTED: &'static str = "null";
pub const NULL_FIELD: &'static str = "fd_null";
pub const NULL_TRAP: &'static str = "tr_null";
pub const DEFAULT_MAP_WIDTH: usize = 24;
pub const DEFAULT_MAP_HEIGHT: usize = 24;
pub const DEFAULT_CELL_CHARACTER: char = ' ';
pub const DEFAULT_EMPTY_CHAR_ROW: &'static str = "                        ";
pub const DEFAULT_MAP_ROWS: [&'static str; 24] = [DEFAULT_EMPTY_CHAR_ROW; 24];
