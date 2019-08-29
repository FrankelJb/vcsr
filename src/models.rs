use crate::constants::*;
use crate::errors::CustomError;
use clap::arg_enum;
use image;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{fmt, str, str::FromStr};
use structopt::StructOpt;

use rustfft::{num_complex::Complex, num_traits::Zero, FFTplanner};

#[derive(Clone, Debug, Default)]
pub struct Grid {
    pub x: u32,
    pub y: u32,
}

impl Eq for Grid {}

impl PartialEq for Grid {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

impl FromStr for Grid {
    type Err = CustomError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mxn: Vec<&str> = s.split("x").collect();
        if mxn.len() > 2 {
            Err(CustomError::GridShape)
        } else {
            let y_fromstr = mxn[1].parse::<u32>()?;
            let x_fromstr = mxn[0].parse::<u32>()?;
            Ok(Grid {
                x: x_fromstr,
                y: y_fromstr,
            })
        }
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
    pub media_attributes: Option<MediaAttributes>,
}

#[derive(Clone, Debug, Default)]
pub struct MediaAttributes {
    pub dimensions: Dimensions,
    pub audio_codec: Option<String>,
    pub audio_codec_long: Option<String>,
    pub audio_bit_rate: Option<u32>,
    pub audio_sample_rate: Option<u32>,
    pub display_aspect_ratio: Option<String>,
    pub duration: String,
    pub duration_seconds: f32,
    pub filename: String,
    pub frame_rate: u32,
    pub sample_aspect_ratio: Option<String>,
    pub size_bytes: f64,
    pub size: String,
    pub video_codec: Option<String>,
    pub video_codec_long: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Dimensions {
    pub display_height: Option<u32>,
    pub display_width: Option<u32>,
    pub sample_height: Option<u32>,
    pub sample_width: Option<u32>,
}

impl MediaInfo {
    pub fn new(path: &Path, _verbose: bool) -> Result<MediaInfo, CustomError> {
        let ffprobe = Self::probe_media(path)?;
        let media_attributes = Self::create_media_attributes(&ffprobe)?;
        Ok(MediaInfo {
            ffprobe: ffprobe,
            media_attributes: Some(media_attributes),
        })
    }

    pub fn probe_media(path: &Path) -> Result<Ffprobe, CustomError> {
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
                Err(CustomError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "ffprobe crashed unexpectedly",
                )))
            }
        } else {
            Err(CustomError::Io(io::Error::new(
                io::ErrorKind::Other,
                "cannot find requested file",
            )))
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

    pub fn find_video_stream(ffprobe: &Ffprobe) -> Option<&Stream> {
        ffprobe.streams.iter().find(|stream| match stream {
            Stream::VideoStream(_) => true,
            Stream::AudioStream(_) => false,
        })
    }

    pub fn find_audio_stream(ffprobe: &Ffprobe) -> Option<&Stream> {
        ffprobe.streams.iter().find(|stream| match stream {
            Stream::VideoStream(_) => false,
            Stream::AudioStream(_) => true,
        })
    }

    pub fn compute_display_resolution(ffprobe: &Ffprobe) -> Result<Dimensions, CustomError> {
        let video_stream = Self::find_video_stream(ffprobe).unwrap().clone();
        if let Stream::VideoStream(video_stream) = video_stream {
            let mut display_height: Option<u32>;
            let mut display_width: Option<u32>;
            let mut sample_height: Option<u32>;
            let mut sample_width: Option<u32>;
            sample_width = video_stream.width;
            sample_height = video_stream.height;
            if let Some(rotation) = video_stream.tags.rotate {
                // Swap width and height
                if rotation == 90 {
                    std::mem::swap(&mut sample_width, &mut sample_height);
                }
            }

            let sample_aspect_ratio = video_stream.sample_aspect_ratio;
            if sample_aspect_ratio == "1:1" {
                display_width = sample_width;
                display_height = sample_height;
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

                let new_sample_width = sample_width.unwrap() * sw / sh;
                display_width = Some(new_sample_width);
                display_height = sample_height;
            }

            if let Some(option_display_width) = display_width {
                if option_display_width == 0 {
                    display_width = sample_width;
                }
            }
            if let Some(option_display_height) = display_height {
                if option_display_height == 0 {
                    display_height = sample_height;
                }
            }
            return Ok(Dimensions {
                display_height: display_height,
                display_width: display_width,
                sample_height: sample_height,
                sample_width: sample_width,
            });
        }
        Err(CustomError::VideoStreamError)
    }

    pub fn compute_duration(ffprobe: &Ffprobe) -> Option<(f32, String)> {
        let video_stream = Self::find_video_stream(ffprobe).unwrap();
        if let Stream::VideoStream(video_stream) = video_stream {
            let duration = match &video_stream.duration {
                Some(duration) => duration,
                None => &ffprobe.format.duration,
            }
            .to_string();
            let duration_seconds = duration.parse::<f32>().unwrap();
            let duration = MediaInfo::pretty_duration(duration.parse::<f32>().unwrap(), true, true);
            return Some((duration_seconds, duration));
        }
        None
    }

    // Compute duration, size and retrieve filename
    pub fn compute_filename(ffprobe: &Ffprobe) -> String {
        Path::new(&ffprobe.format.filename)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    pub fn compute_size(ffprobe: &Ffprobe) -> Result<(f64, String), Box<Error>> {
        let size_bytes = ffprobe.format.size.parse::<f64>()?;
        let size = MediaInfo::human_readable_size(size_bytes);
        Ok((size_bytes, size))
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

    pub fn pretty_to_seconds(pretty_duration: &str) -> Result<f32, CustomError> {
        // TODO: Handle this result
        let millis_split: Vec<&str> = pretty_duration.split(".").collect();
        let mut millis = 0.0;
        let left;
        if millis_split.len() == 2 {
            millis = millis_split[1].parse()?;
            left = millis_split[0];
        } else {
            left = pretty_duration;
        }
        let left_split: Vec<&str> = left.split(":").collect();
        let hours;
        let minutes;
        let seconds;
        if left_split.len() < 3 {
            hours = 0.0;
            minutes = left_split[0].parse::<f32>()?;
            seconds = left_split[1].parse::<f32>()?;
        } else {
            hours = left_split[0].parse::<f32>()?;
            minutes = left_split[1].parse::<f32>()?;
            seconds = left_split[2].parse::<f32>()?;
        }
        Ok((millis / 1000.0) + seconds + minutes * 60.0 + hours * 3600.0)
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

    pub fn desired_size(dimensions: &Dimensions, width: Option<u32>) -> Grid {
        let new_width = match width {
            Some(w) => w,
            None => DEFAULT_CONTACT_SHEET_WIDTH,
        };
        let ratio = new_width as f64 / f64::from(dimensions.display_width.unwrap());
        let desired_height = (dimensions.display_height.unwrap() as f64 * ratio).floor();
        Grid {
            x: new_width,
            y: desired_height as u32,
        }
    }

    // Parse multiple media attributes
    pub fn create_media_attributes(ffprobe: &Ffprobe) -> Result<MediaAttributes, CustomError> {
        let dimensions = Self::compute_display_resolution(&ffprobe)?;
        let (duration_seconds, duration) = Self::compute_duration(&ffprobe).unwrap();
        let filename = Self::compute_filename(&ffprobe);
        let (size_bytes, size) = Self::compute_size(&ffprobe).unwrap();
        let mut video_codec = None;
        let mut video_codec_long = None;
        let mut sample_aspect_ratio = None;
        let mut display_aspect_ratio = None;
        let mut frame_rate = 0;

        // video
        let video_stream = Self::find_video_stream(&ffprobe).unwrap().clone();
        if let Stream::VideoStream(video_stream) = video_stream {
            video_codec = video_stream.codec_name;
            video_codec_long = video_stream.codec_long_name;
            sample_aspect_ratio = Some(video_stream.sample_aspect_ratio);
            display_aspect_ratio = video_stream.display_aspect_ratio;
            if let Some(avg_frame_rate) = video_stream.avg_frame_rate {
                let splits: Vec<&str> = avg_frame_rate.split("/").collect();
                if splits.len() == 2 {
                    frame_rate =
                        (splits[0]).parse::<u32>().unwrap() / splits[1].parse::<u32>().unwrap();
                } else {
                    frame_rate = avg_frame_rate.parse::<u32>().unwrap();
                }

                frame_rate = frame_rate;
            }
        }
        let mut audio_codec = None;
        let mut audio_codec_long = None;
        let mut audio_sample_rate = None;
        let mut audio_bit_rate = None;
        if let Some(audio_stream) = Self::find_audio_stream(&ffprobe) {
            if let Stream::AudioStream(audio_stream) = audio_stream.clone() {
                audio_codec = Some(audio_stream.codec_name);
                audio_codec_long = audio_stream.codec_long_name;
                audio_sample_rate = Some(audio_stream.sample_rate.unwrap().parse().unwrap());
                audio_bit_rate = Some(audio_stream.bit_rate.unwrap().parse().unwrap());
            }
        }
        Ok(MediaAttributes {
            audio_codec: audio_codec,
            audio_codec_long: audio_codec_long,
            audio_sample_rate: audio_sample_rate,
            audio_bit_rate: audio_bit_rate,
            dimensions: dimensions,
            display_aspect_ratio: display_aspect_ratio,
            duration: duration,
            duration_seconds: duration_seconds,
            filename: filename,
            frame_rate: frame_rate,
            sample_aspect_ratio: sample_aspect_ratio,
            size: size,
            size_bytes: size_bytes,
            video_codec: video_codec,
            video_codec_long: video_codec_long,
        })
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
        accurate: bool,
        skip_delay_seconds: Option<f32>,
        frame_type: Option<String>,
    ) -> MediaCapture {
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

    /// Capture a frame at given time with given width and height
    /// using ffmpeg.
    pub fn make_capture(
        &self,
        time: &str,
        width: u32,
        height: u32,
        out_path: Option<&str>,
    ) -> Result<(), CustomError> {
        let skip_delay = MediaInfo::pretty_duration(self.skip_delay_seconds, false, true);
        let out_path = match out_path {
            Some(o) => o,
            None => "out.jpg",
        };


        let mut select_args = match &self.frame_type {
            Some(frame_type) => {
                if frame_type == "key" {
                    vec!["-vf", "select=key"]
                } else {
                    let frame_type_string = format!("\'select=eq(frame_type\\,{})\'", frame_type);
                    vec!["-vf", &frame_type_string]
                }
            }
            None => Vec::new(),
        };

        let time_seconds = MediaInfo::pretty_to_seconds(time)?;
        let skip_time_seconds = time_seconds - self.skip_delay_seconds;
        let skip_time = MediaInfo::pretty_duration(skip_time_seconds, false, true);
        // FIXME: These ss need to be in the correct order
        let mut args = if !self.accurate {
            // || skip_time_seconds < 0.0 {
            vec!["-ss", time]
        } else {
            vec!["-ss", &skip_time, "-ss", &skip_delay]
        };

        args.append(&mut vec!["-i", &self.path]);
        let width_x_height = format!("{}x{}", width, height);
        // args.append(&mut time_parts);
        args.append(&mut vec!["-vframes", "1", "-s", &width_x_height]);
        args.append(&mut select_args);
        args.append(&mut vec!["-y", out_path]);

        Command::new("ffmpeg")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(args)
            .output()?;
        Ok(())
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

#[derive(Clone, Debug, StructOpt)]
pub struct Interval {
    #[structopt(long = "interval")]
    pub interval: String,
}

impl Interval {
    pub fn total_seconds(&self) -> f32 {
        1.0
    }
}

impl FromStr for Interval {
    type Err = CustomError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Interval {
            interval: String::from(s),
        })
    }
}

arg_enum! {
    #[derive(Clone, Debug, StructOpt)]
    pub enum MetadataPosition {
        Top,
        Bottom,
        Hidden
    }
}

arg_enum! {
    #[derive(Clone, Debug, StructOpt)]
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
}
