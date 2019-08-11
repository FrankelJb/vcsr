#![allow(dead_code)]

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;

mod models;

use std::env;

fn main() {
    // TODO: Check that ffprobe is installed
    // println!("{}", models::MediaInfo::human_readable_size(2854871.0));
    env::set_var("RUST_LOG", "vcsi=info");
    env_logger::init();
    let ffprobe = models::MediaInfo::probe_media(
        [
            "/",
            "home",
            "beans",
            "Downloads",
            // "bbb_sunflower_2160p_60fps_normal.mp4",
            "TEST VIDEO  - 4K with 19.5_9 aspect ratio-5JlYIVQxUk8.mkv",
        ]
        .iter()
        .collect(),
    );
    if let Some(ffprobe) = ffprobe {
        let media_info = models::MediaInfo { ffprobe: ffprobe };
        let stream = media_info.compute_display_resolution();
        media_info.compute_format();
    }
}
