use std::time::Duration;

use crate::{
    constants::*,
    models::{Grid, MetadataPosition, TimestampPosition},
};
use clap::Parser;
use humantime::DurationError;

pub fn application_args() -> Args {
    Args::parse()
}

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[arg(skip)]
    pub num_groups: Option<u64>,
    #[arg(skip)]
    pub num_selected: Option<u64>,
    /// Make accurate captures. This capture mode is way slower than the default one but it helps when capturing frames from HEVC videos.
    #[arg(long, short)]
    pub accurate: bool,

    /// Fast skip to N seconds before capture time, then do accurate capture (decodes N seconds of video before each capture). This is used with accurate capture mode only.
    #[arg(long, short = 'A', required = false, default_value = "1.0")]
    pub accurate_delay_seconds: f32,

    ///Make thumbnails of actual size. In other words, thumbnails will have the actual 1:1 size of the video resolution.
    #[arg(long, short = 'S')]
    pub actual_size: bool,

    /// Color of the timestamp background rectangle in hexadecimal, for example AABBCC
    #[arg(long, default_value = "39897eff", required = false)]
    pub background_colour: String,

    /// Alpha channel value for the captures (transparency in range [0, 255]). Defaults to 255 (opaque)
    #[arg(long, default_value = "255", required = false)]
    pub capture_alpha: u8,

    /// do not capture frames in the first and last n percent of total time
    #[arg(long)]
    pub delay_percent: Option<f32>,

    /// do not capture frames in the last n percent of total time
    #[arg(long, default_value = "7", required = false)]
    pub end_delay_percent: f32,

    /// Do not process files that end with the given extensions.
    #[arg(long, default_values_t = vec![("jpg").to_string()])]
    pub exclude_extensions: Vec<String>,

    /// Fast mode. Just make a contact sheet as fast as possible, regardless of output image quality. May mess up the terminal.
    #[arg(long)]
    pub fast: bool,

    /// Frame type passed to ffmpeg 'select=eq(pict_type,FRAME_TYPE)' filter. Should be one of ('I', 'B', 'P') or the special type 'key' which will use the 'select=key' filter instead.
    #[arg(long)]
    pub frame_type: Option<String>,

    /// display frames on a mxn grid (for example 4x5). The special value zero (as in 2x0 or 0x5 or 0x0) is only allowed when combined with --interval or with --manual. Zero means that the component should be automatically deduced based on other arguments passed.
    #[arg(long, short = 'g', default_value = "4x4", required = false)]
    pub grid: Grid,

    /// number of pixels spacing captures both vertically and horizontally
    #[arg(long)]
    pub grid_spacing: Option<u64>,

    /// number of pixels spacing captures horizontally
    #[arg(long, default_value = "15", required = false)]
    pub grid_horizontal_spacing: u64,

    /// number of pixels spacing captures vertically
    #[arg(long, default_value = "15", required = false)]
    pub grid_vertical_spacing: u64,

    /// Output image format. Can be any format supported by image-rs. For example 'png' or 'jpg'.
    #[arg(long = "format", short = 'f', default_value = "jpg", required = false)]
    pub image_format: String,

    /// Ignore any error encountered while processing files recursively and continue to the next file.
    #[arg(long)]
    pub ignore_errors: bool,

    /// Capture frames at specified interval. Interval format is any string supported by `humantime`. For example '5m', '3 minutes 5 seconds', '1 hour 15 min and 20 sec' etc.
    #[arg(long, short, value_parser = parse_humantime_duration)]
    pub interval: Option<Duration>,

    /// Space-separated list of frame timestamps to use, for example 1:11:11.111 2:22:22.222
    #[arg(long = "manual", short = 'm', required = false)]
    pub manual_timestamps: Vec<String>,

    /// Color of the metadata background in hexadecimal, for example AABBCC
    #[arg(long, default_value = "39897eff", required = false)]
    pub metadata_background_colour: String,

    /// Path to TTF font used for metadata
    #[arg(long)]
    pub metadata_font: Option<String>,

    /// Color of the metadata font in hexadecimal, for example AABBCC
    #[arg(long, default_value = "ffffff00", required = false)]
    pub metadata_font_colour: String,

    /// size of the font used for metadata
    #[arg(long, default_value = "32", required = false)]
    pub metadata_font_size: f32,

    /// Horizontal margin (in pixels) in the metadata header.
    #[arg(long, default_value = "15", required = false)]
    pub metadata_horizontal_margin: u64,

    /// Margin (in pixels) in the metadata header.
    #[arg(long, default_value = "15", required = false)]
    pub metadata_margin: u64,

    /// Position of the metadata header.
    #[arg(long, required = false, default_value = "top", value_enum)]
    pub metadata_position: MetadataPosition,

    /// Vertical margin (in pixels) in the metadata header.
    #[arg(long, default_value = "15", required = false)]
    pub metadata_vertical_margin: u64,

    /// Do not overwrite output file if it already exists, simply ignore this file and continue processing other unprocessed files.
    #[arg(long)]
    pub no_overwrite: bool,

    /// save to output file
    #[arg(long = "output", short = 'o')]
    pub output_path: Option<String>,

    /// Process every file in the specified directory recursively
    #[arg(long, short)]
    pub recursive: bool,

    ///number of samples
    #[arg(long, short = 's')]
    pub num_samples: Option<u64>,

    /// show dropshadow on frames
    #[arg(long)]
    pub no_shadow: bool,

    /// do not capture frames in the first n percent of total time
    #[arg(long, default_value = "7", required = false)]
    pub start_delay_percent: f32,

    /// display timestamp for each frame
    #[arg(long, short = 't')]
    pub show_timestamp: bool,

    /// Save thumbnail files to the specified output directory. If set, the thumbnail files will not be deleted after successful creation of the contact sheet.
    #[arg(long, short = 'O')]
    pub thumbnail_output_path: Option<String>,

    /// Color of the timestamp background rectangle in hexadecimal, for example AABBCC
    #[arg(long, default_value = "000000aa", required = false)]
    pub timestamp_background_colour: String,

    /// Color of the timestamp border in hexadecimal, for example AABBCC
    #[arg(long, default_value = "000000", required = false)]
    pub timestamp_border_colour: String,

    /// Draw timestamp text with a border instead of the default rectangle.
    #[arg(long)]
    pub timestamp_border_mode: bool,

    /// Draw timestamp text with a border instead of the default rectangle.
    #[arg(long, default_value = "1.0")]
    pub timestamp_border_radius: f32,

    /// Size of the timestamp border in pixels (used only with --timestamp-border-mode).
    #[arg(long, default_value = "1", required = false)]
    pub timestamp_border_size: u32,

    /// Path to TTF font used for timestamps
    #[arg(long)]
    pub timestamp_font: Option<String>,

    /// Color of the timestamp font in hexadecimal, for example AABBCC
    #[arg(long, default_value = "ffffff", required = false)]
    pub timestamp_font_colour: String,

    /// size of the font used for timestamps
    #[arg(long, default_value = "12", required = false)]
    pub timestamp_font_size: f32,

    /// Timestamp position.
    #[arg(
        long,
        short = 'T',
        ignore_case = true,
        default_value = "se",
        required = false
    )]
    #[arg(value_enum)]
    pub timestamp_position: TimestampPosition,

    //// Horizontal margin (in pixels) for timestamps.
    #[arg(long, default_value = "5", required = false)]
    pub timestamp_horizontal_margin: u64,

    /// Horizontal padding (in pixels) for timestamps.
    #[arg(long, default_value = "3", required = false)]
    pub timestamp_horizontal_padding: u64,

    /// Vertical margin (in pixels) for timestamps.
    #[arg(long, default_value = "5", required = false)]
    pub timestamp_vertical_margin: u64,

    ///V ertical padding (in pixels) for timestamps.
    #[arg(long, default_value = "1", required = false)]
    pub timestamp_vertical_padding: u64,

    /// width of the generated contact sheet
    #[arg(long = "width", short = 'w', default_value = "1500", required = false)]
    pub vcs_width: u64,

    /// log to stdout as well as to the log file.
    #[arg(long, short)]
    pub verbose: bool,

    /// files to be processed. Clap requires this to be the last argument
    #[arg(required = true)]
    pub filenames: Vec<String>,
}

impl Args {
    fn num_samples(grid: Grid) -> Option<u64> {
        Some(grid.x * grid.y)
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            num_groups: None,
            num_selected: None,
            accurate: false,
            accurate_delay_seconds: DEFAULT_ACCURATE_DELAY_SECONDS,
            background_colour: String::from(DEFAULT_BACKGROUND_COLOUR),
            actual_size: false,
            capture_alpha: DEFAULT_CAPTURE_ALPHA,
            delay_percent: DEFAULT_DELAY_PERCENT,
            end_delay_percent: DEFAULT_END_DELAY_PERCENT,
            exclude_extensions: vec![
                String::from("jpg"),
                String::from("txt"),
                String::from("srt"),
                String::from("png"),
            ],
            fast: false,
            frame_type: DEFAULT_FRAME_TYPE,
            filenames: vec![],
            grid: DEFAULT_GRID_SIZE,
            grid_spacing: DEFAULT_GRID_SPACING,
            grid_horizontal_spacing: DEFAULT_GRID_HORIZONTAL_SPACING,
            grid_vertical_spacing: DEFAULT_GRID_VERTICAL_SPACING,
            image_format: String::from(DEFAULT_IMAGE_FORMAT),
            ignore_errors: false,
            interval: DEFAULT_INTERVAL,
            manual_timestamps: vec![],
            metadata_background_colour: String::from(DEFAULT_BACKGROUND_COLOUR),
            metadata_font: DEFAULT_METADATA_FONT,
            metadata_font_colour: String::from(DEFAULT_METADATA_FONT_COLOUR),
            metadata_font_size: DEFAULT_METADATA_FONT_SIZE,
            metadata_horizontal_margin: DEFAULT_METADATA_HORIZONTAL_MARGIN,
            metadata_margin: DEFAULT_METADATA_MARGIN,
            metadata_position: DEFAULT_METADATA_POSITION,
            metadata_vertical_margin: DEFAULT_METADATA_VERTICAL_MARGIN,
            no_overwrite: false,
            output_path: None,
            recursive: false,
            num_samples: None,
            no_shadow: false,
            start_delay_percent: DEFAULT_START_DELAY_PERCENT,
            show_timestamp: true,
            thumbnail_output_path: None,
            timestamp_background_colour: String::from(DEFAULT_TIMESTAMP_BACKGROUND_COLOUR),
            timestamp_border_colour: String::from(DEFAULT_TIMESTAMP_BORDER_COLOUR),
            timestamp_border_mode: false,
            timestamp_border_radius: 1.0,
            timestamp_border_size: 1,
            timestamp_font: None,
            timestamp_font_colour: String::from(DEFAULT_TIMESTAMP_FONT_COLOUR),
            timestamp_font_size: DEFAULT_TIMESTAMP_FONT_SIZE,
            timestamp_position: DEFAULT_TIMESTAMP_POSITION,
            timestamp_horizontal_margin: DEFAULT_TIMESTAMP_HORIZONTAL_MARGIN,
            timestamp_horizontal_padding: DEFAULT_TIMESTAMP_HORIZONTAL_PADDING,
            timestamp_vertical_margin: DEFAULT_TIMESTAMP_VERTICAL_MARGIN,
            timestamp_vertical_padding: DEFAULT_TIMESTAMP_VERTICAL_PADDING,
            vcs_width: DEFAULT_CONTACT_SHEET_WIDTH,
            verbose: false,
        }
    }
}

fn parse_humantime_duration(src: &str) -> Result<Duration, DurationError> {
    Ok(src.parse::<humantime::Duration>()?.into())
}
