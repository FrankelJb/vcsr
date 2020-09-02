#![allow(dead_code)]

extern crate clap;
extern crate console;
extern crate dirs;
extern crate exitcode;
extern crate image;
#[macro_use]
extern crate log;
extern crate num_cpus;
extern crate palette;
extern crate rand;
extern crate rayon;
extern crate serde;
extern crate serde_json;
extern crate textwrap;

pub mod args;
mod constants;
pub mod errors;
pub mod models;
mod operations;

use std::{
    io,
    path::{Path, PathBuf},
    str::FromStr,
};
use walkdir::DirEntry;

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

pub fn process_file(
    dir_entry: &DirEntry,
    args: &mut args::Args,
) -> Result<PathBuf, errors::VcsrError> {
    let file_name_str = dir_entry.file_name().to_str().unwrap();

    if args.verbose {
        info!("Considering {}", file_name_str);
    }

    if !dir_entry.path().exists() {
        if args.ignore_errors {
            info!("File does not exist, skipping {}: ", file_name_str);
            return Ok(dir_entry.path().to_path_buf());
        } else {
            return Err(std::io::Error::new(io::ErrorKind::NotFound, "file does not exist").into());
        }
    }

    let output_path = match &args.output_path {
        None => {
            let mut full_path = dir_entry.path().to_path_buf().into_os_string();
            full_path.push(format!(".{}", args.image_format));
            PathBuf::from(full_path)
        }
        Some(output_path) => {
            if Path::new(output_path).is_dir() {
                let mut full_path = Path::new(output_path)
                    .join(dir_entry.file_name())
                    .into_os_string();
                full_path.push(format!(".{}", args.image_format));
                PathBuf::from(full_path)
            } else {
                Path::new(output_path).to_path_buf()
            }
        }
    };

    if args.no_overwrite {
        if Path::new(&output_path).exists() {
            info!(
                "contact sheet already exists, skipping {}",
                &output_path.to_string_lossy().to_owned().to_owned()
            );
            return Ok(output_path);
        }
    }

    if args.interval.is_some() && !args.manual_timestamps.is_empty() {
        return Err(errors::VcsrError::ArgumentError(
            "Cannot use --interval and --manual at the same time.".to_string(),
        ));
    }

    if args.vcs_width != constants::DEFAULT_CONTACT_SHEET_WIDTH && args.actual_size {
        return Err(errors::VcsrError::ArgumentError(
            "Cannot use --width and --actual-size at the same time.".to_string(),
        ));
    }

    if let Some(delay_percent) = &args.delay_percent {
        args.start_delay_percent = *delay_percent;
        args.end_delay_percent = *delay_percent;
    }

    args.num_groups = Some(5);

    let media_info = models::MediaInfo::new(dir_entry.path(), false)?;
    let media_attributes = media_info
        .media_attributes
        .ok_or_else(|| errors::VcsrError::MediaError)?;
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
        return Err(errors::VcsrError::ArgumentError(
            "Row or column of size zero is only supported with --interval or --manual.".to_string(),
        ));
    }

    if let Some(interval) = &args.interval {
        let total_delay = operations::total_delay_seconds(&media_attributes, &args);
        let selected_duration = media_attributes.duration_seconds - total_delay;
        let num_samples = Some((selected_duration / interval.as_secs() as f32) as u32);
        args.num_samples = num_samples;
        args.num_selected = num_samples;
        args.num_groups = num_samples;
    }

    // manual frame selection
    if !args.manual_timestamps.is_empty() {
        args.manual_timestamps = args
            .manual_timestamps
            .clone()
            .into_iter()
            .filter(|ts| {
                models::MediaInfo::pretty_to_seconds(ts).unwrap()
                    < media_attributes.duration_seconds
            })
            .collect();
        if args.manual_timestamps.is_empty() {
            return Err(errors::VcsrError::TimestampError(String::from(
                "no manual timestamps less than input duration.",
            )));
        }
        let mframes_size = Some(args.manual_timestamps.len() as u32);
        args.num_samples = mframes_size;
        args.num_selected = mframes_size;
        args.num_groups = mframes_size;
    }

    if args.interval.is_some() || !args.manual_timestamps.is_empty() {
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

    args.num_selected = Some(args.grid.x * args.grid.y);

    if let Some(grid_spacing) = args.grid_spacing {
        args.grid_horizontal_spacing = grid_spacing;
        args.grid_vertical_spacing = grid_spacing;
    }

    if args.actual_size {
        let x = args.grid.x;
        args.vcs_width = x * media_attributes.dimensions.display_width.unwrap()
            + (x - 1) * args.grid_horizontal_spacing;
    }

    let (mut selected_frames, temp_frames) =
        operations::select_sharpest_images(&media_attributes, &media_capture, &args)?;

    let image = operations::compose_contact_sheet(&media_attributes, &mut selected_frames, &args)?;

    image.save(&output_path)?;

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

    for frame in temp_frames {
        std::fs::remove_file(frame.filename)?;
    }

    Ok(output_path)
}

pub fn grid_from_str(s: &str) -> Result<models::Grid, errors::VcsrError> {
    models::Grid::from_str(s)
}
