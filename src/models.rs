#![allow(dead_code)]

use crate::constants::*;
use image;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{fmt, str};

use rustfft::{num_complex::Complex, num_traits::Zero, FFTplanner};

#[derive(Clone, Debug, Default)]
pub struct Grid {
    pub x: u32,
    pub y: u32,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub avg_colour: f32,
    pub blurriness: f32,
    pub filename: String,
    pub timestamp: f32,
}

#[derive(Clone, Debug, Default)]
pub struct MediaInfo {
    pub ffprobe: Ffprobe,
    pub audio_codec: Option<String>,
    pub audio_codec_long: Option<String>,
    pub audio_bit_rate: Option<u32>,
    pub audio_sample_rate: Option<u32>,
    pub display_aspect_ratio: Option<String>,
    pub display_height: Option<u32>,
    pub display_width: Option<u32>,
    pub duration: String,
    pub duration_seconds: f32,
    pub filename: String,
    pub frame_rate: u32,
    pub sample_aspect_ratio: Option<String>,
    pub sample_height: Option<u32>,
    pub sample_width: Option<u32>,
    pub size_bytes: i32,
    pub size: String,
    pub video_codec: Option<String>,
    pub video_codec_long: Option<String>,
}

impl MediaInfo {
    pub fn probe_media(path: &Path) -> Result<Ffprobe, io::Error> {
        if path.exists() {
            let output = Command::new("ffprobe")
                // .arg("-v")
                // .arg("quiet")
                .arg("-print_format")
                .arg("json")
                .arg("-show_format")
                .arg("-show_streams")
                .arg(path)
                .output()?;

            if let Ok(stdout) = str::from_utf8(&output.stdout) {
                let r = serde_json::from_str::<Ffprobe>(stdout);
                match r {
                    Ok(_) => println!(""),
                    Err(err) => error!("{}", err),
                };
                let v: Ffprobe = serde_json::from_str(stdout).unwrap();
                Ok(v)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "ffprobe crashed unexpectedly",
                ))
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "cannot find requested file",
            ))
        }
    }

    pub fn human_readable_size(mut num: f64) -> String {
        let suffix = "B";
        let mut size = format!("{:.1} {}{}", num, "Yi", suffix);
        for unit in vec!["", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi"].iter() {
            if num.abs() < 1024.0 {
                size = format!("{:3.1} {}{}", num, unit, suffix);
                break;
            }
            num = num / 1024.0;
        }
        size
    }

    pub fn find_video_stream(&self) -> Option<&Stream> {
        let streams = &self.ffprobe.streams;
        streams.iter().find(|stream| match stream {
            Stream::VideoStream(_) => true,
            Stream::AudioStream(_) => false,
        })
    }

    pub fn find_audio_stream(&self) -> Option<&Stream> {
        let streams = &self.ffprobe.streams;
        streams.iter().find(|stream| match stream {
            Stream::VideoStream(_) => false,
            Stream::AudioStream(_) => true,
        })
    }

    pub fn compute_display_resolution(&mut self) {
        let video_stream = self.find_video_stream().unwrap().clone();
        if let Stream::VideoStream(video_stream) = video_stream {
            self.sample_width = video_stream.width;
            self.sample_height = video_stream.height;
            if let Some(rotation) = video_stream.tags.rotate {
                // Swap width and height
                if rotation == 90 {
                    std::mem::swap(&mut self.sample_width, &mut self.sample_height)
                }
            }

            let sample_aspect_ratio = video_stream.sample_aspect_ratio;
            debug!("sample_aspect_ratio {}", sample_aspect_ratio);
            if sample_aspect_ratio == "1:1" {
                self.display_width = self.sample_width;
                self.display_height = self.sample_height;
            } else {
                let mut sample_split = sample_aspect_ratio.split(":").into_iter();
                let sw = sample_split
                    .next()
                    .unwrap()
                    .to_string()
                    .parse::<u32>()
                    .unwrap();
                let sh = sample_split
                    .next()
                    .unwrap()
                    .to_string()
                    .parse::<u32>()
                    .unwrap();

                let new_sample_width = self.sample_width.unwrap() * sw / sh;
                self.display_width = Some(new_sample_width);
                self.display_height = self.sample_height;
            }

            if let Some(option_display_width) = self.display_width {
                if option_display_width == 0 {
                    self.display_width = self.sample_width;
                }
            }
            if let Some(option_display_height) = self.display_height {
                if option_display_height == 0 {
                    self.display_height = self.sample_height;
                }
            }
            debug!("self.display_width {:?}", self.display_width);
            debug!("self.display_height {:?}", self.display_height);
        }
    }

    // Compute duration, size and retrieve filename
    pub fn compute_format(&mut self) {
        let video_stream = self.find_video_stream().unwrap();
        if let Stream::VideoStream(video_stream) = video_stream {
            let duration = match &video_stream.duration {
                Some(duration) => duration,
                None => &self.ffprobe.format.duration,
            }
            .to_string();
            let duration_seconds = duration.parse::<f32>().unwrap();
            self.duration_seconds = duration_seconds;
            self.duration =
                MediaInfo::pretty_duration(duration.parse::<f32>().unwrap(), true, true);

            self.filename = Path::new(&self.ffprobe.format.filename)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            self.size_bytes = self.ffprobe.format.size.parse().unwrap();
            self.size =
                MediaInfo::human_readable_size(self.ffprobe.format.size.parse::<f64>().unwrap());
        }
    }

    // Converts seconds to a human readable time format
    pub fn pretty_duration(seconds: f32, show_centis: bool, show_millis: bool) -> String {
        let hours = (seconds / 3600.0).floor();
        let remaining_seconds = seconds - 3600.0 * hours;

        let minutes = (remaining_seconds / 60.0).floor();
        let remaining_seconds = remaining_seconds - 60.0 * minutes;
        let mut duration = "".to_string();

        if hours > 0.0 {
            duration = format!("{}:", hours);
        }

        duration = format!(
            "{}{:0>2}:{:0>2}",
            duration,
            minutes,
            remaining_seconds.floor()
        );

        if show_centis || show_millis {
            let mut coeff = 100.0;
            let mut digits = 2;
            if show_millis {
                coeff = 1000.0;
                digits = 3;
            }

            let centis = ((remaining_seconds - remaining_seconds.floor()) * coeff).floor();
            duration = format!("{}.{:0>digits$}", duration, centis, digits = digits);
        }

        duration
    }

    pub fn pretty_to_seconds(pretty_duration: String) -> f32 {
        // TODO: Handle this result
        let millis_split: Vec<&str> = pretty_duration.split(".").collect();
        let mut millis = 0.0;
        let left;
        if millis_split.len() == 2 {
            millis = millis_split[1].parse().unwrap();
            left = millis_split[0].to_string();
        } else {
            left = pretty_duration;
        }
        let left_split: Vec<&str> = left.split(":").collect();
        let hours;
        let minutes;
        let seconds;
        if left_split.len() < 3 {
            hours = 0.0;
            minutes = left_split[0].parse::<f32>().unwrap();
            seconds = left_split[1].parse::<f32>().unwrap();
        } else {
            hours = left_split[0].parse::<f32>().unwrap();
            minutes = left_split[1].parse::<f32>().unwrap();
            seconds = left_split[2].parse::<f32>().unwrap();
        }
        (millis / 1000.0) + seconds + minutes * 60.0 + hours * 3600.0
    }

    pub fn parse_duration(seconds: f32) -> Time {
        let hours = (seconds / 3600.0).floor();
        let remaining_seconds = seconds - 3600.0 * hours;

        let minutes = (remaining_seconds / 60.0).floor();
        let remaining_seconds = remaining_seconds - 60.0 * minutes;
        let seconds = remaining_seconds.floor();

        let millis = ((remaining_seconds - remaining_seconds.floor()) * 1000.0).floor();
        let centis = ((remaining_seconds - remaining_seconds.floor()) * 100.0).floor();

        Time {
            hours,
            minutes,
            seconds,
            centis,
            millis,
        }
    }

    pub fn desired_size(&self, width: Option<u32>) -> Grid {
        let new_width = match width {
            Some(w) => w,
            None => DEFAULT_CONTACT_SHEET_WIDTH,
        };
        let ratio = new_width as f64 / f64::from(self.display_width.unwrap());
        let desired_height = (self.display_height.unwrap() as f64 * ratio).floor();
        Grid {
            x: new_width,
            y: desired_height as u32,
        }
    }

    // Parse multiple media attributes
    pub fn parse_attributes(&mut self) {
        // video
        let video_stream = self.find_video_stream().unwrap().clone();
        if let Stream::VideoStream(video_stream) = video_stream {
            self.video_codec = video_stream.codec_name;
            self.video_codec_long = video_stream.codec_long_name;
            self.sample_aspect_ratio = Some(video_stream.sample_aspect_ratio);
            self.display_aspect_ratio = video_stream.display_aspect_ratio;
            if let Some(avg_frame_rate) = video_stream.avg_frame_rate {
                let splits: Vec<&str> = avg_frame_rate.split("/").collect();
                let frame_rate: u32;
                if splits.len() == 2 {
                    frame_rate =
                        (splits[0]).parse::<u32>().unwrap() / splits[1].parse::<u32>().unwrap();
                } else {
                    frame_rate = avg_frame_rate.parse::<u32>().unwrap();
                }

                self.frame_rate = frame_rate;
            }
        }
        if let Some(audio_stream) = self.find_audio_stream() {
            if let Stream::AudioStream(audio_stream) = audio_stream.clone() {
                self.audio_codec = Some(audio_stream.codec_name);
                self.audio_codec_long = audio_stream.codec_long_name;
                self.audio_sample_rate = Some(audio_stream.sample_rate.unwrap().parse().unwrap());
                self.audio_bit_rate = Some(audio_stream.bit_rate.unwrap().parse().unwrap());
            }
        }
    }
}

pub struct MediaCapture {
    path: String,
    accurate: bool,
    skip_delay_seconds: f32,
    frame_type: Option<String>,
}

impl MediaCapture {
    pub fn new(
        path: String,
        accurate: Option<bool>,
        skip_delay_seconds: Option<f32>,
        frame_type: Option<String>,
    ) -> MediaCapture {
        let accurate = match accurate {
            Some(a) => a,
            None => false,
        };
        let skip_delay_seconds = match skip_delay_seconds {
            Some(s) => s,
            None => DEFAULT_ACCURATE_DELAY_SECONDS,
        };
        MediaCapture {
            path: path,
            accurate: accurate,
            skip_delay_seconds: skip_delay_seconds,
            frame_type: frame_type,
        }
    }

    pub fn make_capture(&self, time: String, width: u32, height: u32, out_path: Option<String>) {
        let skip_delay = MediaInfo::pretty_duration(self.skip_delay_seconds, false, true);
        let out_path = match out_path {
            Some(o) => o,
            None => "out.jpg".to_string(),
        };

        let mut select_args = match &self.frame_type {
            Some(frame_type) => {
                if frame_type == "key" {
                    vec!["-vf".to_string(), "select=key".to_string()]
                } else {
                    vec![
                        "-vf".to_string(),
                        format!("\'select=eq(frame_type\\,{})\'", frame_type).to_string(),
                    ]
                }
            }
            None => Vec::new(),
        };

        let time_seconds = MediaInfo::pretty_to_seconds(time.to_owned());
        let skip_time_seconds = time_seconds - self.skip_delay_seconds;
        let skip_time = MediaInfo::pretty_duration(skip_time_seconds, false, true);
        // FIXME: These ss need to be in the correct order
        let mut args = if !self.accurate {
            // || skip_time_seconds < 0.0 {
            vec!["-ss".to_string(), time]
        } else {
            vec!["-ss".to_string(), skip_time, "-ss".to_string(), skip_delay]
        };

        args.append(&mut vec!["-i".to_string(), self.path.to_string()]);
        let width_x_height = format!("{}x{}", width, height);
        // args.append(&mut time_parts);
        args.append(&mut vec![
            "-vframes".to_string(),
            "1".to_string(),
            "-s".to_string(),
            width_x_height,
        ]);
        args.append(&mut select_args);
        args.append(&mut vec!["-y".to_string(), out_path]);

        Command::new("ffmpeg")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(args)
            .output()
            .expect("Handle this");
    }

    pub fn compute_avg_colour(image_path: &str) -> f32 {
        //TODO: Result
        if Path::new(image_path).exists() {
            let image = image::open(image_path).unwrap().to_rgba();
            let rgbs: (f32, f32, f32) =
                image
                    .enumerate_pixels()
                    .fold((0.0, 0.0, 0.0), |acc, (_, _, p)| match p {
                        image::Rgba { data: rgba } => (
                            acc.0 + rgba[0] as f32,
                            acc.1 + rgba[1] as f32,
                            acc.2 + rgba[2] as f32,
                        ),
                    });
            let size = image.width() as f32 * image.height() as f32;
            (rgbs.0 / size + rgbs.1 / size + rgbs.2 / size) / 3.0
        } else {
            error!("image_path doesn't exist {}", image_path);
            0.0
        }
    }

    pub fn compute_blurrines(image_path: &str) -> f32 {
        // TODO: Handle this result rather than return 0.0
        if Path::new(image_path).exists() {
            let f = std::fs::File::open(image_path).unwrap();
            drop(f);

            let image = image::open(image_path).unwrap().to_luma();
            let mut input: Vec<Complex<f32>> = image
                .enumerate_pixels()
                .map(|(_, _, p)| match p {
                    image::Luma { data: g } => Complex {
                        re: g[0] as f32,
                        im: 0.0,
                    },
                })
                .collect();

            let mut output: Vec<Complex<f32>> = vec![Zero::zero(); input.len()];
            let mut planner = FFTplanner::new(false);
            let fft = planner.plan_fft(input.len());
            fft.process(&mut input, &mut output);

            let mut collected: Vec<f32> = output
                .into_iter()
                .map(|c| match c {
                    Complex { re, im: _ } => (re).abs(),
                })
                .collect();
            collected.sort_by(|a, b| b.partial_cmp(&a).unwrap());
            collected.dedup();
            let max_freq = MediaCapture::avg9x(collected, None);
            if max_freq > 0.0 {
                return 1.0 / max_freq;
            } else {
                return 1.0;
            }
        }
        0.0
    }

    pub fn avg9x(matrix: Vec<f32>, percentage: Option<f32>) -> f32 {
        let percentage = match percentage {
            Some(percentage) => percentage,
            None => 0.05,
        };

        let length = (percentage * matrix.len() as f32).floor() as usize;
        let matrix_subset = &matrix[0..length];
        if length % 2 == 0 {
            (matrix_subset[length / 2 - 1] + matrix_subset[length / 2]) / 2.0
        } else {
            matrix_subset[(length - 1)] / 2.0
        }
    }

    fn max_req(matrix: Vec<f32>) -> f32 {
        *matrix.first().unwrap()
    }
}

#[derive(Debug)]
pub struct Time {
    hours: f32,
    minutes: f32,
    seconds: f32,
    centis: f32,
    millis: f32,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct Disposition {
    attached_pic: u32,
    clean_effects: u32,
    comment: u32,
    default: u32,
    dub: u32,
    forced: u32,
    hearing_impaired: u32,
    karaoke: u32,
    lyrics: u32,
    original: u32,
    timed_thumbnails: u32,
    visual_impaired: u32,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct StreamTags {
    creation_time: Option<String>,
    duration: Option<String>,
    handler_name: Option<String>,
    language: Option<String>,
    rotate: Option<u32>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GenericStream {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Stream {
    VideoStream(VideoStream),
    AudioStream(AudioStream),
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct VideoStream {
    avg_frame_rate: Option<String>,
    bit_rate: Option<String>,
    bits_per_raw_sample: Option<String>,
    codec_long_name: Option<String>,
    codec_name: Option<String>,
    codec_tag: Option<String>,
    codec_tag_string: Option<String>,
    codec_time_base: Option<String>,
    codec_type: String,
    coded_height: Option<i32>,
    coded_width: Option<i32>,
    color_primaries: Option<String>,
    color_range: Option<String>,
    colr_space: Option<String>,
    color_transfer: Option<String>,
    chroma_location: Option<String>,
    display_aspect_ratio: Option<String>,
    display_height: Option<u32>,
    display_width: Option<u32>,
    disposition: Disposition,
    duration_ts: Option<i32>,
    duration: Option<String>,
    has_b_frames: i32,
    height: Option<u32>,
    index: i32,
    is_avc: Option<String>,
    level: Option<i32>,
    nal_length_size: Option<String>,
    nb_frames: Option<String>,
    pix_fmt: Option<String>,
    profile: String,
    r_frame_rate: String,
    refs: Option<i32>,
    #[serde(default = "default_sample_aspect_ratio")]
    sample_aspect_ratio: String,
    start_pts: Option<i32>,
    start_time: Option<String>,
    tags: StreamTags,
    time_base: Option<String>,
    width: Option<u32>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AudioStream {
    avg_frame_rate: Option<String>,
    bit_rate: Option<String>,
    bits_per_sample: Option<i32>,
    channel_layout: String,
    channels: i32,
    codec_long_name: Option<String>,
    codec_name: String,
    codec_tag: String,
    codec_tag_string: String,
    codec_time_base: String,
    codec_type: String,
    disposition: Disposition,
    dmix_mode: Option<String>,
    duration: Option<String>,
    duration_ts: Option<i32>,
    index: i32,
    loro_cmixlev: Option<String>,
    loro_surmixlev: Option<String>,
    ltrt_cmixlev: Option<String>,
    ltrt_surmixlev: Option<String>,
    max_bit_rate: Option<String>,
    nb_frames: Option<String>,
    profile: Option<String>,
    r_frame_rate: Option<String>,
    sample_fmt: Option<String>,
    sample_rate: Option<String>,
    side_data_list: Option<Vec<SideDataType>>,
    start_pts: Option<i32>,
    start_time: Option<String>,
    tags: StreamTags,
    time_base: Option<String>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct SideDataType {
    side_data_type: String,
}

fn default_sample_aspect_ratio() -> String {
    "1:1".to_string()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Format {
    bit_rate: String,
    pub duration: String,
    filename: String,
    format_long_name: String,
    format_name: String,
    nb_programs: i32,
    nb_streams: i32,
    probe_score: i32,
    size: String,
    start_time: String,
    tags: FormatTags,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct FormatTags {
    creation_time: Option<String>,
    compatible_brands: Option<String>,
    encoder: Option<String>,
    major_brand: Option<String>,
    minor_version: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Ffprobe {
    streams: Vec<Stream>,
    pub format: Format,
}

pub struct Args {
    pub background_colour: &'static str,
    pub capture_alpha: u8,
    pub end_delay_percent: f32,
    pub fast: bool,
    pub grid: Grid,
    pub grid_horizontal_spacing: u32,
    pub grid_vertical_spacing: u32,
    pub input_path: String,
    pub interval: Option<Interval>,
    pub manual_timestamps: Option<Vec<String>>,
    pub metadata_font_colour: &'static str,
    pub metadata_font_size: u32,
    pub metadata_horizontal_margin: u32,
    pub metadata_margin: u32,
    pub metadata_position: Option<MetadataPosition>,
    pub metadata_vertical_margin: u32,
    pub num_groups: u32,
    pub num_samples: Option<u32>,
    pub num_selected: u32,
    pub start_delay_percent: f32,
    pub timestamp_position: TimestampPosition,
    pub timestamp_horizontal_margin: u32,
    pub timestamp_vertical_margin: u32,
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
            input_path: "".to_string(),
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
            timestamp_position: TimestampPosition::SE,
            timestamp_horizontal_margin: DEFAULT_TIMESTAMP_HORIZONTAL_MARGIN,
            timestamp_vertical_margin: DEFAULT_TIMESTAMP_HORIZONTAL_MARGIN,
            vcs_width: DEFAULT_CONTACT_SHEET_WIDTH,
        }
    }
}

pub struct Interval {
    pub interval: String,
}

impl Interval {
    pub fn total_seconds(&self) -> f32 {
        1.0
    }
}

pub enum TimestampPosition {
    North,
    South,
    East,
    West,
    NE,
    NW,
    SE,
    SW,
    Center,
}

pub enum MetadataPosition {
    Top,
    Bottom,
}
