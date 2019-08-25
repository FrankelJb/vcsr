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
mod errors;
mod models;
mod operations;

use std::ffi::OsStr;
use std::path::Path;
use std::{env, error::Error};
use walkdir::{DirEntry, WalkDir};

// impl Termination for () {
//     fn report(self) -> i32 {
//         ExitCode::SUCCESS.report()
//     }
// }

// impl<E: fmt::Debug> Termination for Result<(), E> {
//     fn report(self) -> i32 {
//         match self {
//             Ok(()) => ().report(),
//             Err(err) => {
//                 eprintln!("Error: {:?}", err);
//                 ExitCode::FAILURE.report()
//             }
//         }
//     }
// }

fn main() {
    // TODO: Check that ffprobe is installed
    // println!("{}", models::MediaInfo::human_readable_size(2854871.0));

    env::set_var("RUST_LOG", "vcsi=debug,info,warn");
    env_logger::init();

    let args = args::application_args();
    let mut walker: WalkDir;

    for path in &args.filenames {
        if args.recursive {
            walker = WalkDir::new(path);
        } else {
            walker = WalkDir::new(path).max_depth(1);
        }
        for entry in walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension() != None)
            .filter(|e| {
                let extension = e.path().extension().and_then(OsStr::to_str).unwrap();
                if args.exclude_extensions.contains(&String::from(extension)) {
                    warn!("Excluded extension {}. Skipping.", extension);
                    false
                } else {
                    true
                }
            })
        {
            let mut current_args = args.clone();
            process_file(entry, &mut current_args);
        }
    }
}

fn process_file(dir_entry: DirEntry, args: &mut args::Args) -> Result<(), String> {
    let file_name_str = dir_entry.file_name().to_str().unwrap();

    if args.verbose {
        info!("Considering {}", file_name_str);
    }

    if !dir_entry.path().exists() {
        if args.ignore_errors {
            info!("File does not exist, skipping {}: ", file_name_str);
            return Ok(());
        } else {
            return Err(format!("File does not exist: {}", file_name_str));
        }
    }

    let output_path = match &args.output_path {
        None => format!("{}.{}", file_name_str, &args.image_format),
        Some(output_path) => {
            if Path::new(output_path).is_dir() {
                Path::new(output_path)
                    .join(dir_entry.path().file_stem().unwrap())
                    .to_string_lossy()
                    .into_owned()
            } else {
                output_path.to_string()
            }
        }
    };

    if args.no_overwrite {
        if Path::new(&output_path).exists() {
            info!("contact sheet already exists, skipping {}", output_path);
            return Ok(());
        }
    }

    info!("Processing {:?}", dir_entry.path());

    if args.interval.is_some() && !args.manual_timestamps.is_empty() {
        return Err("Cannot use --interval and --manual at the same time.".to_string());
    }

    if let Some(delay_percent) = &args.delay_percent {
        args.start_delay_percent = *delay_percent;
        args.end_delay_percent = *delay_percent;
    }

    args.num_groups = 5;

    let _media_info = models::MediaInfo::new(dir_entry.path(), false);
    // // TODO: Handle results to main
    // let ffprobe =
    //     models::MediaInfo::probe_media(&Path::new(&args.filenames.first().unwrap())).unwrap();
    // let mut media_info = models::MediaInfo {
    //     ffprobe: ffprobe,
    //     ..Default::default()
    // };
    // media_info.compute_display_resolution();
    // media_info.compute_format();
    // // info!("duration: {}", media_info.duration);
    // media_info.parse_attributes();
    // // info!("media_info: {:?}", media_info);
    // let media_capture = models::MediaCapture::new(
    //     args.filenames.first().unwrap().to_string(),
    //     None,
    //     None,
    //     None,
    // );
    // media_capture.make_capture(
    //     "00:02:45".to_string(),
    //     media_info.display_width.unwrap() / 3,
    //     media_info.display_height.unwrap() / 3,
    //     None,
    // );
    // models::MediaCapture::compute_avg_colour("out.jpg");

    // debug!(
    //     "blurinness is {}",
    //     models::MediaCapture::compute_blurrines("out.jpg")
    // );

    // info!("{:?}", operations::timestamp_generator(&media_info, &args));
    // let font = operations::load_font(&args, None, &constants::DEFAULT_TIMESTAMP_FONT);
    // info!(
    //     "{:?}",
    //     operations::prepare_metadata_text_lines(&media_info, &font, 10, 1499)
    // );

    // let mut selected_frames =
    //     operations::select_sharpest_images(&media_info, &media_capture, &args);
    // operations::compose_contact_sheet(media_info, &mut selected_frames, &args);
    Ok(())
}
