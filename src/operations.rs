use crate::args::Args;
use crate::constants::*;
use crate::models::TimestampPosition;
use crate::models::*;

use image::{ImageBuffer, ImageRgba8, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
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
    dimensions: &Dimensions,
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

    MediaInfo::desired_size(dimensions, Some(desired_width))
}

pub fn total_delay_seconds(media_attributes: &MediaAttributes, args: &Args) -> f32 {
    let start_delay_seconds =
        (media_attributes.duration_seconds * args.start_delay_percent / 100.0).floor();
    let end_delay_seconds =
        (media_attributes.duration_seconds * args.end_delay_percent / 100.0).floor();
    start_delay_seconds + end_delay_seconds
}

pub fn timestamp_generator(media_attributes: &MediaAttributes, args: &Args) -> Vec<(f32, String)> {
    let delay = total_delay_seconds(media_attributes, args);
    let capture_interval = match &args.interval {
        Some(interval) => interval.total_seconds(),
        None => {
            (media_attributes.duration_seconds - delay) / (args.num_samples.unwrap() as f32 + 1.0)
        }
    };

    let mut time = (media_attributes.duration_seconds * args.start_delay_percent / 100.0).floor();

    (0..args.num_samples.unwrap())
        .into_iter()
        .map(|_| {
            time = time + capture_interval;
            (time, MediaInfo::pretty_duration(time, false, true))
        })
        .collect()
}

pub fn select_sharpest_images(
    media_attributes: &MediaAttributes,
    media_capture: &MediaCapture,
    args: &Args,
) -> (Vec<Frame>, Vec<Frame>) {
    let desired_size = grid_desired_size(
        &args.grid,
        &media_attributes.dimensions,
        Some(args.vcs_width),
        Some(args.grid_horizontal_spacing),
    );

    let timestamps = if args.manual_timestamps.len() > 0 {
        args.manual_timestamps
            .iter()
            .map(|ts| (MediaInfo::pretty_to_seconds(ts.to_string()), ts.to_string()))
            .collect()
    } else {
        timestamp_generator(media_attributes, &args)
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
            filename: full_path,
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
    &blurs.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

    let num_groups = args.num_groups.unwrap();
    let mut selected_items: Vec<Frame> = vec![];
    if num_groups > 1 {
        let group_size = 1.max(blurs.len() as u32 / num_groups);
        for chunk in blurs.chunks_mut(group_size as usize) {
            chunk.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
            if let Some(c) = chunk.last() {
                selected_items.push(c.clone());
            }
        }
    } else {
        selected_items = blurs.clone();
    };

    let selected_items = select_colour_variety(&mut selected_items, num_groups);
    (selected_items, blurs)
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

pub fn max_line_length(
    media_info_filename: String,
    metadata_font: Font,
    metadata_font_size: f32,
    header_margin: u32,
    width: u32,
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
            .ceil() as u32;

        max_length = i;
        if text_width > max_width {
            break;
        }
    }
    max_length
}

pub fn prepare_metadata_text_lines(
    media_attributes: &MediaAttributes,
    dimensions: &Dimensions,
    header_font: &Font,
    header_margin: u32,
    width: u32,
) -> Vec<String> {
    // TODO: template maybe
    // TODO: font size needs to be set elsewhere
    let mut header_lines = vec![];
    let template = format!(
        r#"{filename}
        File size: {size}
        Duration: {duration}
        Dimensions: {sample_width}x{sample_height}"#,
        filename = media_attributes.filename,
        size = media_attributes.size,
        duration = media_attributes.duration,
        sample_width = dimensions.display_width.unwrap(),
        sample_height = dimensions.display_height.unwrap()
    );

    let template_lines = template
        .split("\n")
        .map(|s| if s.len() > 0 { s.trim() } else { s });
    for line in template_lines {
        let max_metadata_line_length = max_line_length(
            media_attributes.filename.clone(),
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
    desired_size: &Grid,
    rectangle_hpadding: u32,
    rectangle_vpadding: u32,
) -> (Point<u32>, Point<u32>) {
    let x_offset = match args.timestamp_position {
        TimestampPosition::West | TimestampPosition::NW | TimestampPosition::SW => {
            args.timestamp_horizontal_margin
        }
        TimestampPosition::North | TimestampPosition::Center | TimestampPosition::South => {
            (desired_size.x / 2) - (text_size.0 / 2) - rectangle_hpadding
        }
        _ => {
            desired_size.x - text_size.0 - args.timestamp_horizontal_margin - 2 * rectangle_hpadding
        }
    };

    let y_offset = match args.timestamp_position {
        TimestampPosition::NW | TimestampPosition::North | TimestampPosition::NE => {
            args.timestamp_vertical_margin
        }
        TimestampPosition::West | TimestampPosition::Center | TimestampPosition::East => {
            (desired_size.y / 2) - (text_size.1 / 2) - rectangle_vpadding
        }
        _ => desired_size.y - text_size.1 - args.timestamp_vertical_margin - 2 * rectangle_vpadding,
    };

    let upper_left = point(w + x_offset, h + y_offset);
    let size = point(
        text_size.0 + 2 * rectangle_hpadding,
        text_size.1 + 2 * rectangle_vpadding,
    );

    (upper_left, size)
}

pub fn load_font<'a>(
    _args: &'a Args,
    font_path: Option<&str>,
    default_font_path: &str,
) -> Font<'a> {
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

pub fn compose_contact_sheet(
    media_attributes: &MediaAttributes,
    frames: &mut Vec<Frame>,
    args: &Args,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let dimensions = &media_attributes.dimensions;
    let desired_size = grid_desired_size(
        &args.grid,
        &dimensions,
        Some(args.vcs_width),
        Some(args.grid_horizontal_spacing),
    );
    let width = args.grid.x * (desired_size.x + args.grid_horizontal_spacing)
        - args.grid_horizontal_spacing;
    let height =
        args.grid.y * (desired_size.y + args.grid_vertical_spacing) - args.grid_vertical_spacing;

    let header_font = load_font(args, None, &DEFAULT_METADATA_FONT);
    let timestamp_font = load_font(args, None, &DEFAULT_TIMESTAMP_FONT);
    let timestamp_font_scale = Scale::uniform(args.timestamp_font_size);
    let timestamp_border_colour = decode_hex(&args.timestamp_border_colour);

    let header_lines = prepare_metadata_text_lines(
        &media_attributes,
        &dimensions,
        &header_font,
        args.metadata_horizontal_margin,
        width,
    );

    let line_spacing_coefficient = 1.2;
    let header_line_height = (args.metadata_font_size * line_spacing_coefficient) as u32;
    let mut header_height =
        2 * args.metadata_margin + header_lines.len() as u32 + header_line_height;

    if args.metadata_position.is_none() {
        header_height = 0;
    }

    let final_image_width = width;
    let final_image_height = height + header_height;

    let hex_background = decode_hex(&args.background_colour);
    let mut image = RgbaImage::from_pixel(final_image_width, final_image_height, hex_background);

    let mut h = 0;

    if let Some(MetadataPosition::Top) = args.metadata_position {
        h = args.metadata_vertical_margin;
        for line in header_lines {
            draw_text_mut(
                &mut image,
                decode_hex(&args.metadata_font_colour),
                args.metadata_horizontal_margin,
                h,
                Scale { x: 16.0, y: 16.0 },
                &header_font,
                &line,
            );
            h += header_line_height;
        }
    }

    let mut w = 0;
    frames.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
    for (i, frame) in frames.iter().enumerate() {
        let mut f = image::open(&Path::new(&frame.filename)).unwrap().to_rgba();
        putalpha(&mut f, args.capture_alpha);
        image::imageops::replace(&mut image, &mut f, w, h);

        if args.show_timestamp {
            let timestamp_time = MediaInfo::pretty_duration(frame.timestamp, true, false);
            let _timestamp_duration =
                MediaInfo::pretty_duration(media_attributes.duration_seconds, true, true);
            let _parsed_time = MediaInfo::parse_duration(frame.timestamp);
            let _parsed_duraton = MediaInfo::parse_duration(media_attributes.duration_seconds);

            // TODO: Handlebar
            let timestamp_text = format!("{time}", time = timestamp_time);
            let text_size = get_text_size(
                &timestamp_font,
                Scale::uniform(args.timestamp_font_size),
                &timestamp_text,
            );
            let rectangle_hpadding = args.timestamp_horizontal_margin;
            let rectangle_vpadding = args.timestamp_vertical_margin;

            let (upper_left, size) = compute_timestamp_position(
                args,
                w,
                h,
                text_size,
                &desired_size,
                rectangle_hpadding,
                rectangle_vpadding,
            );

            if !args.timestamp_border_mode {
                let timestamp_border_colour = decode_hex(&args.timestamp_border_colour);
                draw_filled_rect_mut(
                    &mut image,
                    Rect::at(upper_left.x as i32, upper_left.y as i32).of_size(size.x, size.y),
                    timestamp_border_colour,
                );
            } else {
                let offset_factor = args.timestamp_border_size;
                let offsets: Vec<(i32, i32)> = vec![
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ];
                let mut final_offsets: Vec<(i32, i32)> = vec![];
                for offset_counter in 1..offset_factor + 1 {
                    for x in &offsets {
                        final_offsets
                            .push((x.0 * offset_counter as i32, x.1 * offset_counter as i32));
                    }
                }
                for offset in final_offsets {
                    draw_text_mut(
                        &mut image,
                        timestamp_border_colour,
                        (upper_left.x as i32 + rectangle_hpadding as i32 + offset.0) as u32,
                        (upper_left.y as i32 + rectangle_vpadding as i32 + offset.1) as u32,
                        timestamp_font_scale,
                        &timestamp_font,
                        &timestamp_text,
                    );
                }
            }
            let timestamp_font_colour = decode_hex(&args.timestamp_font_colour);
            draw_text_mut(
                &mut image,
                timestamp_font_colour,
                upper_left.x + rectangle_hpadding,
                upper_left.y + rectangle_vpadding,
                timestamp_font_scale,
                &timestamp_font,
                &timestamp_text,
            );
        };

        // update x position for next frame
        w += desired_size.x + args.grid_horizontal_spacing;

        // update y position
        if (i as u32 + 1) % args.grid.x == 0 {
            h += desired_size.y + args.grid_vertical_spacing;
        }

        // update x position
        if (i as u32 + 1) % args.grid.x == 0 {
            w = 0;
        }
    }

    image
}

fn save_image(image: ImageBuffer<Rgba<u8>, Vec<u8>>, output_path: &str) -> std::io::Result<()> {
    ImageRgba8(image).to_rgb().save(output_path)?;
    // image.save(output_path)?;
    Ok(())
}

fn decode_hex(s: &str) -> Rgba<u8> {
    if s.len() % 2 != 0 {
        panic!("cannot decode odd length colours");
    } else {
        let mut hex_vec: Vec<u8> = (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect();
        let mut array = [0u8; 4];
        if hex_vec.len() == 3 {
            hex_vec.push(255u8);
        }
        array.copy_from_slice(&hex_vec);
        Rgba(array)
    }
}

fn putalpha(image: &mut RgbaImage, alpha: u8) {
    for (_, _, pixel) in image.enumerate_pixels_mut() {
        match pixel {
            image::Rgba { data: rgba } => {
                (*pixel = image::Rgba([rgba[0], rgba[1], rgba[2], alpha]))
            }
        }
    }
}

fn get_text_size(font: &Font, scale: Scale, text: &str) -> (u32, u32) {
    let v_metrics = font.v_metrics(scale);

    let glyphs: Vec<_> = font.layout(text, scale, Point { x: 0.0, y: 0.0 }).collect();

    let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as u32
    };
    (glyphs_width, glyphs_height)
}
