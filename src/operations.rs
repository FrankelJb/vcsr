use crate::constants::*;
use crate::models::*;

use palette::{Lab, Srgb};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use std::env;

pub fn grid_desired_size(
    grid: &Grid,
    media_info: &MediaInfo,
    width: Option<u32>,
    horizontal_margin: Option<u32>,
) -> (u32, u32) {
    let width = match width {
        Some(width) => width,
        None => DEFAULT_CONTACT_SHEET_WIDTH,
    };

    let horizontal_margin = match horizontal_margin {
        Some(horizontal_margin) => horizontal_margin,
        None => DEFAULT_GRID_HORIZONTAL_SPACING,
    };

    let desired_width = (width - (grid.x - 1) * horizontal_margin) / grid.x;

    media_info.desired_size(Some(desired_width))
}

pub fn total_delay_seconds(media_info: &MediaInfo, args: &Args) -> f32 {
    let start_delay_seconds =
        (media_info.duration_seconds * args.start_delay_percent / 100.0).floor();
    let end_delay_seconds = (media_info.duration_seconds * args.end_delay_percent / 100.0).floor();
    start_delay_seconds + end_delay_seconds
}

pub fn timestamp_generator(media_info: &MediaInfo, args: &Args) -> Vec<(f32, String)> {
    let delay = total_delay_seconds(media_info, args);
    let capture_interval = match &args.interval {
        Some(interval) => interval.total_seconds(),
        None => (media_info.duration_seconds - delay) / (args.num_samples.unwrap() as f32 + 1.0),
    };

    let mut time = (media_info.duration_seconds * args.start_delay_percent / 100.0).floor();

    (0..args.num_samples.unwrap())
        .into_iter()
        .map(|_| {
            time = time + capture_interval;
            (time, MediaInfo::pretty_duration(time, false, true))
        })
        .collect()
}

pub fn select_sharpest_images(media_info: &MediaInfo, media_capture: &MediaCapture, args: &Args) {
    let desired_size = match &args.grid {
        Some(grid) => grid_desired_size(
            grid,
            media_info,
            Some(args.vcs_width),
            Some(args.grid_horizontal_spacing),
        ),
        // TODO: figure out the right default
        None => (4, 4),
    };

    let timestamps = match &args.manual_timestamps {
        Some(timestamps) => timestamps
            .iter()
            .map(|ts| (MediaInfo::pretty_to_seconds(ts.to_string()), ts.to_string()))
            .collect(),
        None => timestamp_generator(media_info, &args),
    };

    let do_capture = |task_number: usize,
                      ts_tuple: (f32, String),
                      width: u32,
                      height: u32,
                      suffix: &str,
                      args: &Args|
     -> Frame {
        info!(
            "Starting task {}/{}",
            task_number,
            args.num_samples.unwrap()
        );
        // TODO: lots to handle
        let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(7).collect();
        let mut dir = env::temp_dir();
        let filename = format!("tmp{}{}", rand_string, suffix);
        dir.push(&filename);
        let full_path = dir.to_string_lossy().into_owned();
        media_capture.make_capture(ts_tuple.1, width, height, Some(full_path.clone()));
        let mut blurriness = 1.0;
        let mut avg_colour: Option<Lab> = Some(Srgb::new(0.0, 0.0, 0.0).into());
        if !args.fast {
            blurriness = MediaCapture::compute_blurrines(&full_path);
            avg_colour = MediaCapture::compute_avg_colour(&full_path);
        }
        Frame {
            filename: filename,
            blurriness: blurriness,
            timestamp: ts_tuple.0,
            avg_colour: avg_colour,
        }
    };

    let mut blurs: Vec<Frame> = timestamps
        .into_par_iter()
        .enumerate()
        .map(|(i, timestamp_tuple)| {
            do_capture(
                i + 1,
                timestamp_tuple,
                desired_size.0,
                desired_size.1,
                ".jpg",
                args,
            )
        })
        .collect();
    blurs.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

    let mut selected_items: Vec<Frame> = vec![];
    if args.num_groups > 1 {
        let group_size = 1.max(blurs.len() as u32 / args.num_groups);
        for chunk in blurs.chunks_mut(group_size as usize) {
            chunk.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
            if let Some(c) = chunk.last() {
                selected_items.push(c.clone());
            }
        }
    } else {
        selected_items = blurs;
    };
    info!("{:#?}", selected_items);
}

pub fn select_colour_variety(frames: Vec<Frame>, num_selected: u32) {
    let avg_colour_sorted = frames.iter().sort_by(|a, b| a.)

}
