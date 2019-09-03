#![allow(dead_code)]

extern crate clap;
extern crate console;
extern crate env_logger;
extern crate exitcode;
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

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    ffi::OsStr,
    io,
    path::Path,
    process::{Command, Stdio},
    thread,
    {env, error::Error},
};
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
    env::set_var("RUST_LOG", "vcsr=debug,info,warn");
    env_logger::builder().default_format_timestamp(false).init();

    let multi = MultiProgress::new();
    let bar_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-");

    match Command::new("ffmpeg")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
    {
        Ok(_) => info!("ffmpeg installed. Continuing."),
        Err(_) => {
            eprintln!("ffmpeg not installed. Exiting.");
            std::process::exit(exitcode::SOFTWARE)
        }
    };
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
            let bar = multi.add(ProgressBar::hidden());
            bar.set_style(bar_style.clone());
            let mut current_args = args.clone();
            let _ = thread::spawn(move || match process_file(entry, &mut current_args, &bar) {
                Ok(file_name) => {
                    bar.finish_with_message(&format!(
                        "contact sheet succesfully created at {}",
                        file_name
                    ));
                }
                Err(err) => {
                    error!("Error: {:?}", err.description());
                    std::process::exit(-1);
                }
            });
        }
    }
    multi.join().unwrap();
    std::process::exit(exitcode::OK);
}

fn process_file(
    dir_entry: DirEntry,
    args: &mut args::Args,
    bar: &ProgressBar,
) -> Result<String, errors::CustomError> {
    let file_name_str = dir_entry.file_name().to_str().unwrap();

    if args.verbose {
        info!("Considering {}", file_name_str);
    }

    if !dir_entry.path().exists() {
        if args.ignore_errors {
            info!("File does not exist, skipping {}: ", file_name_str);
            return Ok(String::from(file_name_str));
        } else {
            return Err(errors::CustomError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "file does not exist",
            )));
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
            info!("contact sheet already exists, skipping {}", &output_path);
            return Ok(output_path);
        }
    }

    info!("Processing {:?}", dir_entry.path());

    if args.interval.is_some() && !args.manual_timestamps.is_empty() {
        return Err(errors::CustomError::ArgumentError(errors::ArgumentError {
            cause: "Cannot use --interval and --manual at the same time.".to_string(),
        }));
    }

    if args.vcs_width != constants::DEFAULT_CONTACT_SHEET_WIDTH && args.actual_size {
        return Err(errors::CustomError::ArgumentError(errors::ArgumentError {
            cause: "Cannot use --width and --actual-size at the same time.".to_string(),
        }));
    }

    if let Some(delay_percent) = &args.delay_percent {
        args.start_delay_percent = *delay_percent;
        args.end_delay_percent = *delay_percent;
    }

    args.num_groups = Some(5);

    let media_info = models::MediaInfo::new(dir_entry.path(), false)?;
    let media_attributes = media_info
        .media_attributes
        .ok_or_else(|| errors::CustomError::MediaError)?;
    let media_capture = models::MediaCapture::new(
        dir_entry.path().to_string_lossy().to_owned().to_string(),
        args.accurate,
        args.accurate_delay_seconds,
        args.frame_type.clone(),
    );

    if args.metadata_margin != constants::DEFAULT_METADATA_MARGIN {
        args.metadata_horizontal_margin = args.metadata_margin;
        args.metadata_vertical_margin = args.metadata_margin;
    }

    if args.interval.is_none()
        && args.manual_timestamps.is_empty()
        && (args.grid.x == 0 || args.grid.y == 0)
    {
        return Err(errors::CustomError::ArgumentError(errors::ArgumentError {
            cause: "Row or column of size zero is only supported with --interval or --manual."
                .to_string(),
        }));
    }

    if let Some(interval) = &args.interval {
        let total_delay = operations::total_delay_seconds(&media_attributes, &args);
        let selected_duration = media_attributes.duration_seconds - total_delay;
        let num_samples =
            Some((selected_duration as f32 / interval.total_seconds().floor()) as u32);
        args.num_samples = num_samples;
        args.num_selected = num_samples;
        args.num_groups = num_samples;
    }

    // manual frame selection
    if !args.manual_timestamps.is_empty() {
        let mframes_size = Some(args.manual_timestamps.len() as u32);
        args.num_samples = mframes_size;
        args.num_selected = mframes_size;
        args.num_groups = mframes_size;
    }

    if args.interval.is_some() && !args.manual_timestamps.is_empty() {
        let square_side = (args.num_samples.unwrap() as f32).sqrt().ceil() as u32;

        if args.grid == constants::DEFAULT_GRID_SIZE || (args.grid.x == 0 && args.grid.y == 0) {
            args.grid = models::Grid {
                x: square_side,
                y: square_side,
            };
        } else if args.grid.x == 0 {
            // y is fixed
            args.grid = models::Grid {
                x: args.num_samples.unwrap() / args.grid.y,
                y: args.grid.y,
            };
        } else if args.grid.y == 0 {
            // x is fixed
            args.grid = models::Grid {
                x: args.grid.y,
                y: args.num_samples.unwrap() / args.grid.x,
            };
        }
    }

    args.num_selected = Some(args.grid.x * args.grid.y);
    bar.set_length(args.num_selected.unwrap() as u64 + 4);

    if args.num_samples.is_none() {
        args.num_samples = args.num_selected;
    }
    if args.num_groups.is_none() {
        args.num_groups = args.num_selected;
    }

    // make sure num_selected isn't too large
    if args.interval.is_none() && args.manual_timestamps.is_empty() {
        if args.num_selected.unwrap() > args.num_groups.unwrap() {
            args.num_groups = args.num_selected;
        }

        if args.num_selected.unwrap() > args.num_samples.unwrap() {
            args.num_samples = args.num_selected;
        }

        // make sure num_samples is large enough
        if args.num_samples.unwrap() < args.num_selected.unwrap()
            || args.num_samples.unwrap() < args.num_groups.unwrap()
        {
            args.num_samples = args.num_selected;
            args.num_groups = args.num_selected;
        }
    }

    if let Some(grid_spacing) = args.grid_spacing {
        args.grid_horizontal_spacing = grid_spacing;
        args.grid_vertical_spacing = grid_spacing;
    }

    bar.inc(1);

    let (mut selected_frames, temp_frames) =
        operations::select_sharpest_images(&media_attributes, &media_capture, &args, &bar)?;

    bar.set_message("Composing contact sheet");
    bar.inc(1);

    let image = operations::compose_contact_sheet(&media_attributes, &mut selected_frames, &args)?;

    bar.set_message("Saving image");
    image.save(&output_path)?;
    bar.inc(1);

    if let Some(thumbnail_output_path) = &args.thumbnail_output_path {
        if !Path::new(thumbnail_output_path).exists() {
            std::fs::create_dir_all(thumbnail_output_path)?;
        }
        info!("Copying thumbnails to {}", thumbnail_output_path);
        for (i, frame) in selected_frames.iter().enumerate() {
            let thumbnail_file_extension = Path::new(&frame.filename).extension().unwrap();
            let thumbnail_filename = format!(
                "{}.{:0>4}.{}",
                dir_entry.path().file_stem().unwrap().to_str().unwrap(),
                i,
                thumbnail_file_extension.to_str().unwrap()
            );
            let thumbnail_destination = Path::new(thumbnail_output_path).join(thumbnail_filename);
            std::fs::copy(&frame.filename, thumbnail_destination)?;
        }
    }

    bar.set_message("Cleaning up temporary files");
    for frame in temp_frames {
        std::fs::remove_file(frame.filename)?;
    }
    bar.inc(1);

    Ok(output_path)
}
