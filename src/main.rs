#![allow(dead_code)]

extern crate env_logger;
extern crate image;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;

mod constants;
mod models;
mod operations;

use std::env;
use std::path::Path;

fn main() {
    // TODO: Check that ffprobe is installed
    // println!("{}", models::MediaInfo::human_readable_size(2854871.0));

    env::set_var("RUST_LOG", "vcsi=debug,info");
    env_logger::init();
;

    let args: Vec<String> = env::args().collect();
    // TODO: Handle results to main
    let ffprobe = models::MediaInfo::probe_media(&Path::new(&args[1])).unwrap();
    let mut media_info = models::MediaInfo {
        ffprobe: ffprobe,
        ..Default::default()
    };
    media_info.compute_display_resolution();
    media_info.compute_format();
    // info!("duration: {}", media_info.duration);
    media_info.parse_attributes();
    // info!("media_info: {:?}", media_info);
    let media_capture =
        models::MediaCapture::new(args[1].to_string(), None, None, Some("key".to_string()));
    media_capture.make_capture(
        "00:02:45".to_string(),
        10,
        10,
        // media_info.display_width.unwrap() / 10,
        // media_info.display_height.unwrap() / 10,
        None,
    );
    models::MediaCapture::compute_avg_colour("out.jpg".to_string());

    models::MediaCapture::compute_blurrines("out.jpg".to_string());

    info!(
        "avg9x: {}",
        models::MediaCapture::avg9x(
            vec![
                20.0, 19.0, 18.0, 17.0, 16.0, 15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0, 7.0,
                6.0, 5.0, 4.0, 3.0, 2.0, 1.0,
            ],
            None
        )
    )
    // let args: models::Args = Default::default();
    // info!("{:?}", operations::timestamp_generator(&media_info, &args));
}
