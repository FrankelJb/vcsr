use crate::models::Grid;
pub const DEFAULT_ACCURATE_DELAY_SECONDS: f32 = 1.0;
pub const DEFAULT_BACKGROUND_COLOUR: &str = "000000";
pub const DEFAULT_CONTACT_SHEET_WIDTH: u32 = 1500;
pub const DEFAULT_CAPTURE_ALPHA: u8 = 255;
pub const DEFAULT_FRAME_TYPE: Option<u8> = None;
pub const DEFAULT_GRID_HORIZONTAL_SPACING: u32 = 5;
pub const DEFAULT_GRID_VERTICAL_SPACING: u32 = DEFAULT_GRID_HORIZONTAL_SPACING;
pub const DEFAULT_GRID_SPACING: Grid = Grid { x: 4, y: 4 };
pub const DEFAULT_METADATA_MARGIN: u32 = 10;
pub const DEFAULT_METADATA_FONT_COLOUR: &str = "ffffff";
pub const DEFAULT_METADATA_FONT_SIZE: f32 = 16.0;
pub const DEFAULT_METADATA_FONT: &str =
    "/Users/jonfrankel/projects/rust/vcsi/resources/DejaVuSans.ttf";
pub const DEFAULT_METADATA_HORIZONTAL_MARGIN: u32 = DEFAULT_METADATA_MARGIN;
pub const DEFAULT_METADATA_VERTICAL_MARGIN: u32 = DEFAULT_METADATA_MARGIN;
pub const FALLBACK_FONTS: &str = "/Library/Fonts/Arial Unicode.ttf";
pub const DEFAULT_TIMESTAMP_HORIZONTAL_MARGIN: u32 = 5;
pub const DEFAULT_TIMESTAMP_HORIZONTAL_PADDING: u32 = 3;
pub const DEFAULT_TIMESTAMP_VERTICAL_MARGIN: u32 = 5;
pub const DEFAULT_TIMESTAMP_VERTICAL_PADDING: u32 = 1;
pub const DEFAULT_TIMESTAMP_FONT_SIZE: f32 = 12.0;
pub const DEFAULT_TIMESTAMP_FONT: &str =
    "/Users/jonfrankel/projects/rust/vcsi/resources/DejaVuSans.ttf";
