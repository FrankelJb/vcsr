#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command; //, Output, Stdio};
use std::{fmt, str};

pub struct Grid {
    pub x: u32,
    pub y: u32,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

pub struct Frame {
    pub filename: String,
    pub blurriness: String,
    pub timestamp: String,
    pub avg_colour: String,
}

pub struct Colour {
    pub r: u32,
    pub g: u32,
    pub b: u32,
    pub a: u32,
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}{}", self.r, self.g, self.b, self.a)
    }
}

impl Colour {
    pub fn to_hex(self, component: u32) -> String {
        format!("{:X}", component)
    }
}

#[derive(Debug, Default)]
pub struct MediaInfo {
    pub ffprobe: Ffprobe,
}

impl MediaInfo {
    pub fn probe_media(path: PathBuf) -> Option<Ffprobe> {
        //-> Result<Output, std::io::Error> {
        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_format")
            .arg("-show_streams")
            .arg(path.into_os_string())
            .output()
            .expect("HANDLE THIS");

        if let Ok(stdout) = str::from_utf8(&output.stdout) {
            // info!("{}", stdout);
            let v: Value = serde_json::from_str(stdout).unwrap();
            info!("{:#?}", v);
            let r = serde_json::from_str::<Ffprobe>(stdout);
            match r {
                Ok(_) => println!(""),
                Err(err) => error!("{}", err),
            };
            let v: Ffprobe = serde_json::from_str(stdout).unwrap();
            Some(v)
        } else {
            None
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

    pub fn compute_display_resolution(&self) -> Stream {
        let video_stream = self.find_video_stream().unwrap().clone();
        if let Stream::VideoStream(mut video_stream) = video_stream {
            let display_width;
            let display_height;

            let mut sample_width = video_stream.width;
            let mut sample_height = video_stream.height;
            if let Some(rotation) = video_stream.tags.rotate {
                info!("Rotation is: {}", rotation);
                // Swap width and height
                if rotation == 90 {
                    std::mem::swap(&mut sample_width, &mut sample_height)
                }
            }
            if video_stream.sample_aspect_ratio == "1:1" {
                display_width = Some(sample_width);
                display_height = Some(sample_height);
            } else {
                let mut sample_split = video_stream.sample_aspect_ratio.split(":").into_iter();
                let sw = sample_split
                    .next()
                    .unwrap()
                    .to_string()
                    .parse::<i32>()
                    .unwrap();
                let sh = sample_split
                    .next()
                    .unwrap()
                    .to_string()
                    .parse::<i32>()
                    .unwrap();

                display_width = Some(sample_width * sw / sh);
                display_height = Some(sample_height);
            }
            video_stream.display_width = display_width;
            video_stream.display_height = display_height;

            if let Some(option_display_width) = display_width {
                if option_display_width == 0 {
                    video_stream.display_width = Some(sample_width);
                }
            }
            if let Some(option_display_height) = display_height {
                if option_display_height == 0 {
                    video_stream.display_height = Some(sample_height);
                }
            }
            Stream::VideoStream(video_stream)
        } else {
            video_stream
        }
    }

    // Compute duration, size and retrieve filename
    pub fn compute_format(&self) {
        let video_stream = self.find_video_stream().unwrap();
        if let Stream::VideoStream(video_stream) = video_stream {
            info!("duration {:?} or {:?}", video_stream.duration, self.ffprobe.format.duration);
            let duration = match &video_stream.duration {
                Some(duration) => duration,
                None => &self.ffprobe.format.duration,
            };
            let pretty = self.pretty_duration(duration.parse::<f32>().unwrap(), true, true);
            info!("pretty duration: {}", pretty);
        }
    }

    // Converts seconds to a human readable time format
    pub fn pretty_duration(&self, seconds: f32, show_centis: bool, show_millis: bool) -> String {
        let hours = (seconds / 3600.0).floor();
        let remaining_seconds = seconds - 3600.0 * hours;

        let minutes = (remaining_seconds / 60.0).floor();
        let remaining_seconds = seconds - 60.0 * hours;
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
    color_space: Option<String>,
    color_transfer: Option<String>,
    chroma_location: Option<String>,
    display_aspect_ratio: String,
    display_height: Option<i32>,
    display_width: Option<i32>,
    disposition: Disposition,
    duration_ts: Option<i32>,
    duration: Option<String>,
    has_b_frames: i32,
    height: i32,
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
    width: i32,
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

#[derive(Debug, Default, Serialize, Deserialize)]
struct Format {
    bit_rate: String,
    duration: String,
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

#[derive(Debug, Default, Serialize, Deserialize)]
struct FormatTags {
    creation_time: Option<String>,
    compatible_brands: Option<String>,
    encoder: Option<String>,
    major_brand: Option<String>,
    minor_version: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ffprobe {
    streams: Vec<Stream>,
    format: Format,
}
