use crate::constants::*;
use crate::models::{Grid, Interval, MetadataPosition, TimestampPosition};

pub fn application_args() -> Args {
    Args::default()
}

pub struct Args {
    pub background_colour: &'static str,
    pub capture_alpha: u8,
    pub end_delay_percent: f32,
    pub fast: bool,
    pub grid: Grid,
    pub grid_horizontal_spacing: u32,
    pub grid_vertical_spacing: u32,
    pub input_path: &'static str,
    pub interval: Option<Interval>,
    pub manual_timestamps: Option<Vec<String>>,
    pub metadata_font_colour: &'static str,
    pub metadata_font_size: f32,
    pub metadata_horizontal_margin: u32,
    pub metadata_margin: u32,
    pub metadata_position: Option<MetadataPosition>,
    pub metadata_vertical_margin: u32,
    pub num_groups: u32,
    pub num_samples: Option<u32>,
    pub num_selected: u32,
    pub start_delay_percent: f32,
    pub show_timestamp: bool,
    pub timestamp_background_colour: &'static str,
    pub timestamp_border_colour: &'static str,
    pub timestamp_border_mode: bool,
    pub timestamp_border_size: u32,
    pub timestamp_font_colour: &'static str,
    pub timestamp_font_size: f32,
    pub timestamp_position: TimestampPosition,
    pub timestamp_horizontal_margin: u32,
    pub timestamp_horizontal_padding: u32,
    pub timestamp_vertical_margin: u32,
    pub timestamp_vertical_padding: u32,
    pub vcs_width: u32,
}

impl Args {
    fn num_samples(grid: Grid) -> Option<u32> {
        Some(grid.x * grid.y)
    }
}

impl Default for Args {
    fn default() -> Self {
        Args {
            background_colour: DEFAULT_BACKGROUND_COLOUR,
            capture_alpha: DEFAULT_CAPTURE_ALPHA,
            end_delay_percent: 7.0,
            fast: false,
            grid: DEFAULT_GRID_SPACING,
            grid_horizontal_spacing: DEFAULT_GRID_HORIZONTAL_SPACING,
            grid_vertical_spacing: DEFAULT_GRID_VERTICAL_SPACING,
            interval: None,
            input_path: "",
            manual_timestamps: None,
            metadata_font_colour: DEFAULT_METADATA_FONT_COLOUR,
            metadata_font_size: DEFAULT_METADATA_FONT_SIZE,
            metadata_horizontal_margin: DEFAULT_METADATA_HORIZONTAL_MARGIN,
            metadata_margin: DEFAULT_METADATA_MARGIN,
            metadata_position: Some(MetadataPosition::Top),
            metadata_vertical_margin: DEFAULT_METADATA_VERTICAL_MARGIN,
            // TODO: Change this to the right thing
            num_groups: 16,
            num_samples: Args::num_samples(DEFAULT_GRID_SPACING),
            num_selected: DEFAULT_GRID_SPACING.x * DEFAULT_GRID_SPACING.y,
            start_delay_percent: 7.0,
            show_timestamp: true,
            timestamp_background_colour: DEFAULT_TIMESTAMP_BACKGROUND_COLOUR,
            timestamp_border_colour: DEFAULT_TIMESTAMP_BORDER_COLOUR,
            timestamp_border_mode: false,
            timestamp_border_size: DEFAULT_TIMESTAMP_BORDER_SIZE,
            timestamp_font_colour: DEFAULT_TIMESTAMP_FONT_COLOUR,
            timestamp_font_size: DEFAULT_TIMESTAMP_FONT_SIZE,
            timestamp_position: TimestampPosition::SE,
            timestamp_horizontal_margin: DEFAULT_TIMESTAMP_HORIZONTAL_MARGIN,
            timestamp_horizontal_padding: DEFAULT_TIMESTAMP_HORIZONTAL_PADDING,
            timestamp_vertical_margin: DEFAULT_TIMESTAMP_VERTICAL_MARGIN,
            timestamp_vertical_padding: DEFAULT_TIMESTAMP_VERTICAL_PADDING,
            vcs_width: DEFAULT_CONTACT_SHEET_WIDTH,
        }
    }
}