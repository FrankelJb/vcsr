use crate::constants::*;
use crate::models::*;

pub fn grid_desired_size(
    grid: Grid,
    media_info: MediaInfo,
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
        (media_info.duration_seconds + args.start_delay_percent / 100.0).floor();
    let end_delay_seconds = (media_info.duration_seconds + args.end_delay_percent / 100.0).floor();
    start_delay_seconds + end_delay_seconds
}

pub fn timestamp_generator(media_info: &MediaInfo, args: &Args) -> Vec<(f32, String)> {
    let delay = total_delay_seconds(media_info, args);
    let capture_interval = match &args.interval {
        Some(interval) => interval.total_seconds(),
        None => (media_info.duration_seconds - delay) / (args.num_samples as f32 + 1.0),
    };

    let mut time = (media_info.duration_seconds * args.start_delay_percent / 100.0).floor();

    (0..args.num_samples)
        .into_iter()
        .map(|_| {
            time = time + capture_interval;
            (time, MediaInfo::pretty_duration(time, false, true))
        })
        .collect()
}
