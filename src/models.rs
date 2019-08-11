#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
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
            println!("{}", stdout);
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
        streams.iter().find(|stream| stream.codec_name == "video")
    }

    pub fn find_audio_stream(&self) -> Option<&Stream> {
        let streams = &self.ffprobe.streams;
        streams.iter().find(|stream| stream.codec_name == "audio")
    }

    pub fn compute_display_resolution(&self) -> Stream {
        let mut video_stream = self.find_video_stream().unwrap().clone();
        let sample_width = video_stream.width;
        let sample_height = video_stream.height;
        let mut display_width = video_stream.display_width;
        let mut display_height = video_stream.display_height;

        if let Some(mut sample_width) = video_stream.width {
            if let Some(mut sample_height) = video_stream.height {
                if let Some(rotation) = video_stream.tags.rotate {
                    // Swap width and height
                    if rotation == 90 {
                        std::mem::swap(&mut sample_width, &mut sample_height)
                    }
                }
                if let Some(sample_aspect_ratio) = &video_stream.sample_aspect_ratio {
                    if sample_aspect_ratio == "1:1" {
                        display_width = Some(sample_width);
                        display_height = Some(sample_height);
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

                        display_width = Some(sample_width * sw / sh);
                        display_height = Some(sample_height);
                    }
                    video_stream.display_width = display_width;
                    video_stream.display_height = display_height;
                }
            }
        }
        if let Some(option_display_width) = display_width {
            if option_display_width == 0 {
                video_stream.display_width = sample_width;
            }
        }
        if let Some(option_display_height) = display_height {
            if option_display_height == 0 {
                video_stream.display_height = sample_height;
            }
        }
        video_stream
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct Disposition {
    default: u32,
    dub: u32,
    original: u32,
    comment: u32,
    lyrics: u32,
    karaoke: u32,
    forced: u32,
    hearing_impaired: u32,
    visual_impaired: u32,
    clean_effects: u32,
    attached_pic: u32,
    timed_thumbnails: u32,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct StreamTags {
    creation_time: String,
    language: String,
    handler_name: String,
    rotate: Option<u32>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Stream {
    index: u32,
    codec_name: String,
    codec_long_name: String,
    profile: Option<String>,
    codec_type: String,
    codec_time_base: String,
    codec_tag_string: String,
    codec_tag: String,
    width: Option<u32>,
    height: Option<u32>,
    display_width: Option<u32>,
    display_height: Option<u32>,
    coded_width: Option<u32>,
    coded_height: Option<u32>,
    has_b_frames: Option<u32>,
    pix_fmt: Option<String>,
    level: Option<u32>,
    chroma_location: Option<String>,
    refs: Option<u32>,
    is_avc: Option<String>,
    nal_length_size: Option<String>,
    r_frame_rate: String,
    avg_frame_rate: String,
    time_base: String,
    start_pts: u32,
    start_time: String,
    duration_ts: u32,
    duration: String,
    bit_rate: String,
    bits_per_raw_sample: Option<String>,
    nb_frames: String,
    disposition: Disposition,
    tags: StreamTags,
    sample_aspect_ratio: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Format {
    filename: String,
    nb_streams: u32,
    nb_programs: u32,
    format_name: String,
    format_long_name: String,
    start_time: String,
    duration: String,
    size: String,
    bit_rate: String,
    probe_score: u32,
    tags: FormatTags,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct FormatTags {
    major_brand: String,
    minor_version: String,
    compatible_brands: String,
    creation_time: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ffprobe {
    streams: Vec<Stream>,
    format: Format,
}
