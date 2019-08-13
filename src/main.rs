#![allow(dead_code)]

extern crate env_logger;
extern crate image;
#[macro_use]
extern crate log;
#[macro_use]
extern crate palette;
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
            "bbb_sunflower_2160p_60fps_normal.mp4",
            // "TEST VIDEO  - 4K with 19.5_9 aspect ratio-5JlYIVQxUk8.mkv",
            // "Just can't eat in peace!.mp4",
        ]
        .iter()
        .collect(),
    );
    if let Some(ffprobe) = ffprobe {
        let mut media_info = models::MediaInfo {
            ffprobe: ffprobe,
            ..Default::default()
        };
        media_info.compute_display_resolution();
        media_info.compute_format();
        info!("duration: {}", media_info.duration);
        let seconds = models::MediaInfo::pretty_to_seconds(media_info.duration.to_owned());
        info!("to seconds: {}", seconds);
        info!("to Time: {:?}", models::MediaInfo::parse_duration(seconds));
        info!("desired_size: {:?}", &media_info.desired_size(None));
        media_info.parse_attributes();
        info!("media_info: {:?}", media_info);
        let media_capture = models::MediaCapture::new(
            "/home/beans/Downloads/bbb_sunflower_2160p_60fps_normal.mp4".to_string(),
            None,
            None,
            Some("key".to_string()),
        );
        media_capture.make_capture(
            "00:02:45".to_string(),
            media_info.display_width.unwrap() / 10,
            media_info.display_height.unwrap() / 10,
            None,
        );
        models::MediaCapture::compute_avg_colour("out.jpg".to_string());
    }
}
