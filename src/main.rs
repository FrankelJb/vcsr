#![allow(dead_code)]

extern crate clap;
extern crate env_logger;
extern crate image;
#[macro_use]
extern crate log;
extern crate palette;
extern crate rand;
extern crate rayon;
extern crate serde;
extern crate serde_json;
extern crate textwrap;

mod args;
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

    let args = args::clap_app();

    // TODO: Handle results to main
    let ffprobe = models::MediaInfo::probe_media(&Path::new(&args.input_path)).unwrap();
    let mut media_info = models::MediaInfo {
        ffprobe: ffprobe,
        ..Default::default()
    };
    media_info.compute_display_resolution();
    media_info.compute_format();
    // info!("duration: {}", media_info.duration);
    media_info.parse_attributes();
    // info!("media_info: {:?}", media_info);
    let _media_capture = models::MediaCapture::new(args.input_path.to_string(), None, None, None);
    // media_capture.make_capture(
    //     "00:02:45".to_string(),
    //     media_info.display_width.unwrap() / 3,
    //     media_info.display_height.unwrap() / 3,
    //     None,
    // );
    models::MediaCapture::compute_avg_colour("out.jpg");

    debug!(
        "blurinness is {}",
        models::MediaCapture::compute_blurrines("out.jpg")
    );

    info!("{:?}", operations::timestamp_generator(&media_info, &args));
    let font = operations::load_font(
        &args,
        None,
        "/home/beans/projects/rust/imageproc/font/src/DejaVuSans.ttf",
    );
    info!(
        "{:?}",
        operations::prepare_metadata_text_lines(&media_info, font, 10, 1499)
    );
    // operations::select_sharpest_images(&media_info, &media_capture, &args);
}
