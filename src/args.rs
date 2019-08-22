use crate::constants::*;
use crate::models::{Grid, Interval, MetadataPosition, TimestampPosition};
use structopt::StructOpt;

pub fn application_args() -> Args {
    Args::from_args()
}

#[derive(Debug, StructOpt)]
pub struct Args {
    ///Color of the timestamp background rectangle in hexadecimal, for example AABBCC
    #[structopt(long, default_value = "000000")]
    pub background_colour: String,
    ///Alpha channel value for the captures (transparency in range [0, 255]). Defaults to 255 (opaque)
    #[structopt(long, default_value = "255")]
    pub capture_alpha: u8,
    ///do not capture frames in the last n percent of total time
    #[structopt(long, default_value = "7")]
    pub end_delay_percent: f32,
    ///Fast mode. Just make a contact sheet as fast as possible, regardless of output image quality. May mess up the terminal.
    #[structopt(long)]
    pub fast: bool,
    ///display frames on a mxn grid (for example 4x5). The special value zero (as in 2x0 or 0x5 or 0x0) is only allowed when combined with --interval or with --manual. Zero means that the component should be automatically deduced based on other arguments passed.
    #[structopt(long, default_value = "4x4")]
    pub grid: Grid,

    ///number of pixels spacing captures horizontally
    #[structopt(long, default_value = "5")]
    pub grid_horizontal_spacing: u32,

    ///number of pixels spacing captures vertically
    #[structopt(long, default_value = "5")]
    pub grid_vertical_spacing: u32,

    #[structopt(multiple = true)]
    pub filenames: Vec<String>,
    ///Capture frames at specified interval. Interval format is any string supported by `parsedatetime`. For example '5m', '3 minutes 5 seconds', '1 hour 15 min and 20 sec' etc.
    #[structopt(long)]
    pub interval: Option<Interval>,

    ///Comma-separated list of frame timestamps to use, for example 1:11:11.111,2:22:22.222
    #[structopt(long = "manual", short = "m", required = false)]
    pub manual_timestamps: Vec<String>,

    ///Color of the metadata font in hexadecimal, for example AABBCC
    #[structopt(long, default_value = "ffffff")]
    pub metadata_font_colour: String,

    ///size of the font used for metadata
    #[structopt(long, default_value = "12.0")]
    pub metadata_font_size: f32,

    ///Horizontal margin (in pixels) in the metadata header.
    #[structopt(long, default_value = "10")]
    pub metadata_horizontal_margin: u32,

    ///Margin (in pixels) in the metadata header.
    #[structopt(long, default_value = "10")]
    pub metadata_margin: u32,

    ///Position of the metadata header. Must be one of ['top', 'bottom']
    #[structopt(long)]
    pub metadata_position: Option<MetadataPosition>,

    ///Vertical margin (in pixels) in the metadata header.
    #[structopt(long, default_value = "10")]
    pub metadata_vertical_margin: u32,

    // TODO: move this to another struct 
    pub num_groups: u32,

    // TODO: num_samples logic
    #[structopt(long, help = "number of samples")]
    pub num_samples: Option<u32>,
    #[structopt(long)]
    pub num_selected: u32,
    #[structopt(long)]
    pub start_delay_percent: f32,
    #[structopt(long)]
    pub show_timestamp: bool,
    #[structopt(long)]
    pub timestamp_background_colour: String,
    #[structopt(long)]
    pub timestamp_border_colour: String,
    #[structopt(long)]
    pub timestamp_border_mode: bool,
    #[structopt(long)]
    pub timestamp_border_size: u32,
    #[structopt(long)]
    pub timestamp_font_colour: String,
    #[structopt(long)]
    pub timestamp_font_size: f32,
    #[structopt(long)]
    pub timestamp_position: TimestampPosition,
    #[structopt(long)]
    pub timestamp_horizontal_margin: u32,
    #[structopt(long)]
    pub timestamp_horizontal_padding: u32,
    #[structopt(long)]
    pub timestamp_vertical_margin: u32,
    #[structopt(long)]
    pub timestamp_vertical_padding: u32,
    #[structopt(long)]
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
            background_colour: String::from(DEFAULT_BACKGROUND_COLOUR),
            capture_alpha: DEFAULT_CAPTURE_ALPHA,
            end_delay_percent: 7.0,
            fast: false,
            grid: DEFAULT_GRID_SPACING,
            grid_horizontal_spacing: DEFAULT_GRID_HORIZONTAL_SPACING,
            grid_vertical_spacing: DEFAULT_GRID_VERTICAL_SPACING,
            interval: None,
            filenames: vec![],
            manual_timestamps: vec![],
            metadata_font_colour: String::from(DEFAULT_METADATA_FONT_COLOUR),
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
            timestamp_background_colour: String::from(DEFAULT_TIMESTAMP_BACKGROUND_COLOUR),
            timestamp_border_colour: String::from(DEFAULT_TIMESTAMP_BORDER_COLOUR),
            timestamp_border_mode: false,
            timestamp_border_size: DEFAULT_TIMESTAMP_BORDER_SIZE,
            timestamp_font_colour: String::from(DEFAULT_TIMESTAMP_FONT_COLOUR),
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