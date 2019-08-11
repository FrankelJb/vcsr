#![allow(dead_code)]

extern crate serde;
extern crate serde_json;

mod models;

fn main() {
    // TODO: Check that ffprobe is installed
    // println!("{}", models::MediaInfo::human_readable_size(2854871.0));
    let ffprobe = models::MediaInfo::probe_media(
        [
            "/",
            "home",
            "beans",
            "Downloads",
            "bbb_sunflower_2160p_60fps_normal.mp4",
        ]
        .iter()
        .collect(),
    );
    if let Some(ffprobe) = ffprobe {
        println!("{:?}", ffprobe);
        let media_info = models::MediaInfo {ffprobe: ffprobe};
        let stream = media_info.compute_display_resolution();
        println!("{:?}", stream);
    }
}
