use crate::constants::*;
use crate::models::TimestampPosition;
use crate::models::*;

use image::{ Rgb };
use imageproc::drawing::draw_text_mut;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use rusttype::{point, Font, FontCollection, Point, PositionedGlyph, Scale};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use textwrap::fill;

pub fn grid_desired_size(
    grid: &Grid,
    media_info: &MediaInfo,
    width: Option<u32>,
    horizontal_margin: Option<u32>,
) -> Grid {
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

pub fn select_sharpest_images(media_info: &MediaInfo, media_capture: &MediaCapture, args: &Args) -> Vec<Frame> {
    let desired_size =grid_desired_size(
            &args.grid,
            media_info,
            Some(args.vcs_width),
            Some(args.grid_horizontal_spacing)
        );

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
        let mut avg_colour = 0.0;
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
                desired_size.x,
                desired_size.y,
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

    let selected_items = select_colour_variety(&mut selected_items, args.num_groups);
    selected_items
}

pub fn select_colour_variety(frames: &mut Vec<Frame>, num_selected: u32) -> Vec<Frame> {
    frames.sort_by(|a, b| a.avg_colour.partial_cmp(&b.avg_colour).unwrap());
    let min_colour = frames.first().unwrap().avg_colour;
    let max_colour = frames.last().unwrap().avg_colour;
    let colour_span = max_colour - min_colour;
    let min_colour_distance = colour_span * 0.05;

    frames.sort_by(|a, b| a.blurriness.partial_cmp(&b.blurriness).unwrap());
    let mut selected_items: Vec<Frame> = vec![];
    let mut unselected_items: Vec<Frame> = vec![];

    while !frames.is_empty() {
        let frame = frames.pop().unwrap();
        if selected_items.is_empty() {
            selected_items.push(frame.clone());
        } else {
            let colour_distance = frames.iter().fold(0.0, |acc, f| {
                if frame.avg_colour - f.avg_colour < acc {
                    frame.avg_colour - f.avg_colour
                } else {
                    acc
                }
            });
            if colour_distance < min_colour_distance {
                unselected_items.push(frame.clone());
            } else {
                selected_items.push(frame.clone());
            }
        }
    }

    let missing_item_count = num_selected - selected_items.len() as u32;
    if missing_item_count > 0 {
        unselected_items.sort_by(|a, b| a.blurriness.partial_cmp(&b.blurriness).unwrap());
        selected_items.extend_from_slice(&unselected_items[0..missing_item_count as usize]);
    }

    selected_items
}

pub fn draw_metadata(
    image: &str,
    args: &Args,
    header_line_height: u32,
    header_lines: Vec<&str>,
    header_font: &Font,
    header_font_colour: Rgb<u8>,
    start_height: u32,
) -> u32 {
    let mut h = start_height + args.metadata_vertical_margin;
    let mut img = image::open(image).unwrap().to_rgb();
    for line in header_lines {
        draw_text_mut(
            &mut img,
            header_font_colour,
            args.metadata_horizontal_margin,
            h,
            Scale { x: 1.0, y: 1.0 },
            header_font,
            line,
        );
        h += header_line_height;
    }
    h
}

pub fn max_line_length(
    media_info_filename: String,
    metadata_font: Font,
    metadata_font_size: f32,
    header_margin: usize,
    width: usize,
    text: Option<&str>,
) -> usize {
    let text = match text {
        Some(text) => text.to_string(),
        None => media_info_filename,
    };

    let max_width = width - 2 * header_margin;
    let scale = Scale::uniform(metadata_font_size);

    let v_metrics = metadata_font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    let mut max_length = 0;
    for i in 0..text.len() + 1 {
        let text_chunk = text.get(0..i).unwrap();
        let glyphs: Vec<PositionedGlyph<'_>> =
            metadata_font.layout(text_chunk, scale, offset).collect();
        let text_width = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;

        max_length = i;
        if text_width > max_width {
            break;
        }
    }
    max_length
}

pub fn prepare_metadata_text_lines(
    media_info: &MediaInfo,
    header_font: Font,
    header_margin: usize,
    width: usize,
) -> Vec<String> {
    // TODO: template maybe
    // TODO: font size needs to be set elsewhere
    let mut header_lines = vec![];
    let template = format!(
        r#"{filename}
        File size: {size}
        Duration: {duration}
        Dimensions: {sample_width}x{sample_height}"#,
        filename = media_info.filename,
        size = media_info.size,
        duration = media_info.duration,
        sample_width = media_info.display_width.unwrap(),
        sample_height = media_info.display_height.unwrap()
    );

    let template_lines = template
        .split("\n")
        .map(|s| if s.len() > 0 { s.trim() } else { s });
    for line in template_lines {
        let max_metadata_line_length = max_line_length(
            media_info.filename.clone(),
            header_font.clone(),
            16.0,
            header_margin,
            width,
            Some(line),
        );
        println!("max_metadata_line_length {}", max_metadata_line_length);
        header_lines.push(fill(line, max_metadata_line_length));
    }
    header_lines
}

pub fn compute_timestamp_position(
    args: &Args,
    w: u32,
    h: u32,
    text_size: (u32, u32),
    desired_size: (u32, u32),
    rectangle_hpadding: u32,
    rectangle_vpadding: u32,
) -> (Point<u32>, Point<u32>) {
    let x_offset = match args.timestamp_position {
        TimestampPosition::West | TimestampPosition::NW | TimestampPosition::SW => {
            args.timestamp_horizontal_margin
        }
        TimestampPosition::North | TimestampPosition::Center | TimestampPosition::South => {
            (desired_size.0 / 2) - (text_size.0 / 2) - rectangle_hpadding
        }
        _ => {
            desired_size.0 - text_size.0 - args.timestamp_horizontal_margin - 2 * rectangle_hpadding
        }
    };

    let y_offset = match args.timestamp_position {
        TimestampPosition::NW | TimestampPosition::North | TimestampPosition::NE => {
            args.timestamp_vertical_margin
        }
        TimestampPosition::West | TimestampPosition::Center | TimestampPosition::East => {
            (desired_size.1 / 2) - (text_size.1 / 2) - rectangle_vpadding
        }
        _ => desired_size.1 - text_size.1 - args.timestamp_vertical_margin - 2 * rectangle_vpadding,
    };

    let upper_left = point(w + x_offset, h + y_offset);
    let bottom_right = point(
        upper_left.x + text_size.0 + 2 * rectangle_hpadding,
        upper_left.y + text_size.1 + 2 * rectangle_vpadding,
    );

    (upper_left, bottom_right)
}

pub fn load_font<'a>(_args: &'a Args, font_path: Option<&str>, default_font_path: &str) -> Font<'a> {
    // TODO: default font can be included in repo
    let fonts = font_path.unwrap_or(default_font_path);
    let font_path = Path::new(&fonts);
    if font_path.exists() {
        let mut file = File::open(font_path).unwrap();
        let mut data = Vec::new();
        let _ = file.read_to_end(&mut data);
        FontCollection::from_bytes(data)
            .unwrap()
            .into_font()
            .unwrap()
    } else {
        panic!("Cannot load font: {}", fonts);
    }
}

pub fn compose_contact_sheet(media_info: MediaInfo, frames: Vec<Frame>, args: &Args) {

    let desired_size = grid_desired_size(&args.grid, &media_info, Some(args.vcs_width), Some(args.grid_horizontal_spacing));
}