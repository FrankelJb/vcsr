use crate::models::{Grid, Interval, MetadataPosition, TimestampPosition};
use structopt::StructOpt;

pub fn application_args() -> Args {
    Args::from_args()
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    rename_all = "kebab-case"
)]
pub struct Args {
    ///Make accurate captures. This capture mode is way slower than the default one but it helps when capturing frames from HEVC videos.
    #[structopt(long, short)]
    pub accurate: bool,

    ///Fast skip to N seconds before capture time, then do accurate capture (decodes N seconds of video before each capture). This is used with accurate capture mode only.
    #[structopt(long, short = "A")]
    pub accurate_delay_seconds: Option<f32>,

    ///Color of the timestamp background rectangle in hexadecimal, for example AABBCC
    #[structopt(long, default_value = "ffffff00", required = false)]
    pub background_colour: String,

    ///Make thumbnails of actual size. In other words, thumbnails will have the actual 1:1 size of the video resolution.
    #[structopt(long, short = "S")]
    pub actual_size: bool,

    ///Alpha channel value for the captures (transparency in range [0, 255]). Defaults to 255 (opaque)
    #[structopt(long, default_value = "255", required = false)]
    pub capture_alpha: u8,

    ///do not capture frames in the first and last n percent of total time
    #[structopt(long)]
    pub delay_percent: Option<f32>,

    ///do not capture frames in the last n percent of total time
    #[structopt(long, default_value = "7", required = false)]
    pub end_delay_percent: f32,

    ///Do not process files that end with the given extensions.
    #[structopt(long)]
    pub exclude_extensions: Vec<String>,

    ///Fast mode. Just make a contact sheet as fast as possible, regardless of output image quality. May mess up the terminal.
    #[structopt(long)]
    pub fast: bool,

    ///Frame type passed to ffmpeg 'select=eq(pict_type,FRAME_TYPE)' filter. Should be one of ('I', 'B', 'P') or the special type 'key' which will use the 'select=key' filter instead.
    #[structopt(long)]
    pub frame_type: Option<String>,

    #[structopt(multiple = true, last = true)]
    pub filenames: Vec<String>,

    ///display frames on a mxn grid (for example 4x5). The special value zero (as in 2x0 or 0x5 or 0x0) is only allowed when combined with --interval or with --manual. Zero means that the component should be automatically deduced based on other arguments passed.
    #[structopt(long, short = "g", default_value = "4x4", required = false)]
    pub grid: Grid,

    ///number of pixels spacing captures both vertically and horizontally
    #[structopt(long)]
    pub grid_spacing: Option<u32>,

    ///number of pixels spacing captures horizontally
    #[structopt(long, default_value = "15", required = false)]
    pub grid_horizontal_spacing: u32,

    ///number of pixels spacing captures vertically
    #[structopt(long, default_value = "15", required = false)]
    pub grid_vertical_spacing: u32,

    ///Output image format. Can be any format supported by image-rs. For example 'png' or 'jpg'.
    #[structopt(long = "format", short = "f", default_value = "jpg", required = false)]
    pub image_format: String,

    ///Ignore any error encountered while processing files recursively and continue to the next file.
    #[structopt(long)]
    pub ignore_errors: bool,

    ///Capture frames at specified interval. Interval format is any string supported by `parsedatetime`. For example '5m', '3 minutes 5 seconds', '1 hour 15 min and 20 sec' etc.
    #[structopt(long)]
    pub interval: Option<Interval>,

    ///Comma-separated list of frame timestamps to use, for example 1:11:11.111,2:22:22.222
    #[structopt(long = "manual", short = "m", required = false)]
    pub manual_timestamps: Vec<String>,

    ///Path to TTF font used for metadata
    #[structopt(
        long,
        default_value = "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        required = false
    )]
    pub metadata_font: String,

    ///Color of the metadata font in hexadecimal, for example AABBCC
    #[structopt(long, default_value = "000000ff", required = false)]
    pub metadata_font_colour: String,
    
    ///Color of the metadata background in hexadecimal, for example AABBCC
    #[structopt(long, default_value = "b0cd7b0a", required = false)]
    pub metadata_background_colour: String,

    ///size of the font used for metadata
    #[structopt(long, default_value = "16.0", required = false)]
    pub metadata_font_size: f32,

    ///Horizontal margin (in pixels) in the metadata header.
    #[structopt(long, default_value = "15", required = false)]
    pub metadata_horizontal_margin: u32,

    ///Margin (in pixels) in the metadata header.
    #[structopt(long, default_value = "15", required = false)]
    pub metadata_margin: u32,

    ///Position of the metadata header.
    #[structopt(
        long,
        raw(
            possible_values = "&MetadataPosition::variants()",
            case_insensitive = "true",
        ),
        required = false,
        default_value = "top"
    )]
    pub metadata_position: MetadataPosition,

    ///Vertical margin (in pixels) in the metadata header.
    #[structopt(long, default_value = "10", required = false)]
    pub metadata_vertical_margin: u32,

    ///Do not overwrite output file if it already exists, simply ignore this file and continue processing other unprocessed files.
    #[structopt(long)]
    pub no_overwrite: bool,

    // TODO: move this to another struct
    #[structopt(long)]
    pub num_groups: Option<u32>,
    #[structopt(long)]
    pub num_selected: Option<u32>,

    ///save to output file
    #[structopt(long = "output", short = "o")]
    pub output_path: Option<String>,

    ///Process every file in the specified directory recursively
    #[structopt(long, short)]
    pub recursive: bool,

    ///number of samples
    #[structopt(long, short = "s", help = "number of samples")]
    pub num_samples: Option<u32>,

    ///do not capture frames in the first n percent of total time
    #[structopt(long, default_value = "7", required = false)]
    pub start_delay_percent: f32,

    ///display timestamp for each frame
    #[structopt(long, short = "t")]
    pub show_timestamp: bool,

    ///Save thumbnail files to the specified output directory. If set, the thumbnail files will not be deleted after successful creation of the contact sheet.
    #[structopt(long, short = "O")]
    pub thumbnail_output_path: Option<String>,

    ///Color of the timestamp background rectangle in hexadecimal, for example AABBCC
    #[structopt(long, default_value = "000000aa", required = false)]
    pub timestamp_background_colour: String,

    ///Color of the timestamp border in hexadecimal, for example AABBCC
    #[structopt(long, default_value = "000000", required = false)]
    pub timestamp_border_colour: String,

    ///Draw timestamp text with a border instead of the default rectangle.
    #[structopt(long)]
    pub timestamp_border_mode: bool,

    ///Size of the timestamp border in pixels (used only with --timestamp-border-mode).
    #[structopt(long, default_value = "1", required = false)]
    pub timestamp_border_size: u32,

    ///Path to TTF font used for timestamps
    #[structopt(
        long,
        default_value = "/usr/share/fonts/TTF/DejaVuSans.ttf",
        required = false
    )]
    pub timestamp_font: String,

    ///Color of the timestamp font in hexadecimal, for example AABBCC
    #[structopt(long, default_value = "ffffff", required = false)]
    pub timestamp_font_colour: String,

    ///size of the font used for timestamps
    #[structopt(long, default_value = "12", required = false)]
    pub timestamp_font_size: f32,

    ///Timestamp position.
    #[structopt(
        long,
        short = "T",
        raw(
            possible_values = "&TimestampPosition::variants()",
            case_insensitive = "true"
        ),
        default_value = "se",
        required = false
    )]
    pub timestamp_position: TimestampPosition,

    ////Horizontal margin (in pixels) for timestamps.
    #[structopt(long, default_value = "5", required = false)]
    pub timestamp_horizontal_margin: u32,

    ///Horizontal padding (in pixels) for timestamps.
    #[structopt(long, default_value = "3", required = false)]
    pub timestamp_horizontal_padding: u32,

    ///Vertical margin (in pixels) for timestamps.
    #[structopt(long, default_value = "5", required = false)]
    pub timestamp_vertical_margin: u32,

    ///Vertical padding (in pixels) for timestamps.
    #[structopt(long, default_value = "1", required = false)]
    pub timestamp_vertical_padding: u32,

    ///width of the generated contact sheet
    #[structopt(long = "width", short = "w", default_value = "1500", required = false)]
    pub vcs_width: u32,

    ///display verbose messages
    #[structopt(long, short)]
    pub verbose: bool,
}

impl Args {
    fn num_samples(grid: Grid) -> Option<u32> {
        Some(grid.x * grid.y)
    }
}
