use crate::args::Args;
use crate::constants::*;
use crate::errors::CustomError;
use crate::models::{
    Dimensions, Frame, Grid, MediaAttributes, MediaCapture, MediaInfo, MetadataPosition,
    TimestampPosition,
};

use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use imageproc::{drawing::draw_text_mut, rect::Rect};
use indicatif::ProgressBar;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rayon::prelude::*;
use rusttype::{point, Font, FontCollection, Point, PositionedGlyph, Scale};
use std::{env, fs::File, io::prelude::*, path::Path};
use textwrap::wrap;

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

pub fn timestamp_generator(media_attributes: &MediaAttributes, args: &Args) -> Vec<String> {
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
            time += capture_interval;
            time
        })
        .map(|ts| MediaInfo::pretty_duration(ts, false, true))
        .collect()
}

pub fn select_sharpest_images(
    media_attributes: &MediaAttributes,
    media_capture: &MediaCapture,
    args: &Args,
    bar: &ProgressBar,
) -> Result<(Vec<Frame>, Vec<Frame>), CustomError> {
    let desired_size = grid_desired_size(
        &args.grid,
        &media_attributes.dimensions,
        Some(args.vcs_width),
        Some(args.grid_horizontal_spacing),
    );

    let timestamps = if args.manual_timestamps.len() > 0 {
        args.manual_timestamps.clone()
    } else {
        timestamp_generator(media_attributes, &args)
    };

    let do_capture = |task_number: usize,
                      ts_tuple: (f32, String),
                      width: u32,
                      height: u32,
                      suffix: &str,
                      args: &Args|
     -> Result<Frame, CustomError> {
        bar.set_message(&format!("Creating capture {}", task_number));
        bar.inc(1);
        let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(7).collect();
        let mut dir = env::temp_dir();
        let filename = format!("tmp{}{}", rand_string, suffix);
        dir.push(&filename);
        let full_path = dir.to_string_lossy().into_owned();
        media_capture.make_capture(&ts_tuple.1, width, height, Some(&full_path))?;
        let mut blurriness = 1.0;
        let mut avg_colour = 0.0;
        if !args.fast {
            blurriness = MediaCapture::compute_blurrines(&full_path)?;
            avg_colour = MediaCapture::compute_avg_colour(&full_path)?;
        }
        Ok(Frame {
            filename: full_path,
            blurriness: blurriness,
            timestamp: ts_tuple.0,
            avg_colour: avg_colour,
        })
    };

    let blurs: Result<Vec<Frame>, CustomError> = timestamps
        .into_par_iter()
        .enumerate()
        .map(|(i, ts)| {
            do_capture(
                i,
                (MediaInfo::pretty_to_seconds(&ts)?, ts),
                desired_size.x,
                desired_size.y,
                if args.fast { ".jpg" } else { ".png" },
                args,
            )
        })
        .collect();
    let mut time_sorted = blurs?;
    &time_sorted.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

    let num_groups = args.num_groups.unwrap();
    let mut selected_items: Vec<Frame> = vec![];
    if num_groups > 1 {
        let group_size = 1.max(time_sorted.len() as u32 / num_groups);
        for chunk in time_sorted.chunks_mut(group_size as usize) {
            chunk.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
            if let Some(c) = chunk.last() {
                selected_items.push(c.clone());
            }
        }
    } else {
        selected_items = time_sorted.clone();
    };

    let selected_items = select_colour_variety(&mut selected_items, num_groups);
    Ok((selected_items, time_sorted))
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
    media_info_filename: &str,
    metadata_font: &Font,
    metadata_font_size: f32,
    header_margin: u32,
    width: u32,
    text: Option<&str>,
) -> usize {
    let text = match text {
        Some(text) => text,
        None => media_info_filename,
    };

    let max_width = width - 2 * header_margin;
    let scale = Scale::uniform(metadata_font_size);

    let v_metrics = metadata_font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    let mut max_length = 0;
    for i in 0..text.chars().count() + 1 {
        if let Some(text_chunk) = text.get(0..i) {
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
    }
    max_length
}

pub fn prepare_metadata_text_lines(
    media_attributes: &MediaAttributes,
    dimensions: &Dimensions,
    header_font: &Font,
    header_font_size: f32,
    header_margin: u32,
    width: u32,
) -> Vec<String> {
    // TODO: template maybe
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
        let mut remaining_chars = line;
        while remaining_chars.len() > 0 {
            let max_metadata_line_length = max_line_length(
                &media_attributes.filename,
                &header_font,
                header_font_size,
                header_margin,
                width,
                Some(line),
            );
            let wraps = wrap(remaining_chars, max_metadata_line_length);
            header_lines.push(String::from(wraps[0].clone()));
            remaining_chars = &remaining_chars[wraps[0].len()..];
        }
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

pub fn load_font<'a>(font_path_str: &str) -> Result<Font<'a>, CustomError> {
    let font_path = Path::new(font_path_str);
    if font_path.exists() {
        let mut file = File::open(font_path).unwrap();
        let mut data = Vec::new();
        let _ = file.read_to_end(&mut data);
        FontCollection::from_bytes(data)
            .unwrap()
            .into_font()
            .map_err(|e| CustomError::RustTypeError(e))
    } else {
        Err(CustomError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("file does not found {}", font_path_str),
        )))
    }
}

pub fn draw_metadata<'a>(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    args: &Args,
    header_line_height: u32,
    header_lines: &Vec<String>,
    header_font_colour: Rgba<u8>,
    header_font_size: f32,
    header_font: &'a Font<'a>,
) -> u32 {
    let mut h = args.grid_vertical_spacing;
    let scale = Scale::uniform(header_font_size);
    for line in header_lines {
        // Give the text a shadow because I just learned how
        // to do this.

        let text_size = get_text_size(&header_font, scale, line);
        let mut shadow = RgbaImage::from_pixel(
            text_size.0,
            text_size.1,
            decode_hex(&args.metadata_background_colour),
        );
        draw_text_mut(
            &mut shadow,
            Rgba([0, 0, 0, 255]),
            0,
            0,
            scale,
            &header_font,
            line,
        );

        let blur = image::imageops::blur(&shadow, 1.0);

        image::imageops::replace(img, &blur, args.metadata_horizontal_margin + 2, h + 2);
        draw_text_mut(
            img,
            header_font_colour,
            args.metadata_horizontal_margin,
            h,
            scale,
            &header_font,
            &line,
        );
        h += header_line_height;
    }
    h
}

/// Creates a video contact sheet with the media information in a header
/// and the selected frames arranged on a mxn grid with optional
/// timestamps
pub fn compose_contact_sheet(
    media_attributes: &MediaAttributes,
    frames: &mut Vec<Frame>,
    args: &Args,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, CustomError> {
    let dimensions = &media_attributes.dimensions;
    let desired_size = grid_desired_size(
        &args.grid,
        &dimensions,
        Some(args.vcs_width),
        Some(args.grid_horizontal_spacing),
    );
    let width = args.grid.x * (desired_size.x + args.grid_horizontal_spacing)
        + args.grid_horizontal_spacing;
    let height =
        args.grid.y * (desired_size.y + args.grid_vertical_spacing) + args.grid_vertical_spacing;

    let header_font = match &args.metadata_font {
        Some(font_path_str) => load_font(&font_path_str)?,
        None => {
            let data = include_bytes!("../resources/Roboto-Bold.ttf").to_vec();
            FontCollection::from_bytes(data)
                .unwrap()
                .into_font()
                .map_err(|e| CustomError::RustTypeError(e))?
        }
    };
    let timestamp_font = match &args.timestamp_font {
        Some(font_path_str) => load_font(&font_path_str)?,
        None => {
            let data = include_bytes!("../resources/Roboto-Regular.ttf").to_vec();
            FontCollection::from_bytes(data)
                .unwrap()
                .into_font()
                .map_err(|e| CustomError::RustTypeError(e))?
        }
    };
    let timestamp_font_scale = Scale::uniform(args.timestamp_font_size);
    let timestamp_border_colour = decode_hex(&args.timestamp_border_colour);

    let header_lines = prepare_metadata_text_lines(
        &media_attributes,
        &dimensions,
        &header_font,
        args.metadata_font_size,
        args.metadata_horizontal_margin,
        width,
    );

    let line_spacing_coefficient = 1.2;
    let header_line_height = (args.metadata_font_size * line_spacing_coefficient) as u32;
    let mut header_height =
        2 * args.metadata_margin + header_lines.len() as u32 * header_line_height;

    if let MetadataPosition::Hidden = args.metadata_position {
        header_height = 0;
    }

    let final_image_width = width;
    let final_image_height = height + header_height;

    let hex_background = decode_hex(&args.background_colour);
    let mut image = RgbaImage::from_pixel(final_image_width, final_image_height, hex_background);

    let mut metadata_image = RgbaImage::from_pixel(
        final_image_width,
        header_height,
        decode_hex(&args.metadata_background_colour),
    );

    let mut y = 0;

    if let MetadataPosition::Top = args.metadata_position {
        y = header_height;
    }

    draw_metadata(
        &mut metadata_image,
        &args,
        header_line_height,
        &header_lines,
        decode_hex(&args.metadata_font_colour),
        args.metadata_font_size,
        &header_font,
    );

    let mut x = args.grid_horizontal_spacing;
    y += args.grid_vertical_spacing;

    let shadow_width = 10;
    let mut rect = RgbaImage::from_pixel(
        desired_size.x + shadow_width,
        desired_size.y + shadow_width,
        hex_background,
    );
    let black_pixel = Rgba([0, 0, 0, 0]);
    imageproc::drawing::draw_filled_rect_mut(
        &mut rect,
        Rect::at(shadow_width as i32 / 2, shadow_width as i32 / 2)
            .of_size(desired_size.x, desired_size.y),
        black_pixel,
    );
    let mut blurred = image::imageops::blur(&mut rect, 3.0);
    frames.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
    for (i, frame) in frames.iter().enumerate() {
        let mut f = image::open(&Path::new(&frame.filename)).unwrap().to_rgba();
        putalpha(&mut f, args.capture_alpha);

        if !args.no_shadow {
            image::imageops::replace(&mut image, &mut blurred, x, y);
        }
        image::imageops::replace(&mut image, &mut f, x, y);

        if args.show_timestamp {
            let timestamp_time = MediaInfo::pretty_duration(frame.timestamp, true, false);
            let _timestamp_duration =
                MediaInfo::pretty_duration(media_attributes.duration_seconds, true, true);
            let _parsed_time = MediaInfo::parse_duration(frame.timestamp);
            let _parsed_duraton = MediaInfo::parse_duration(media_attributes.duration_seconds);

            // TODO: Handlebar
            let timestamp_text = format!("{}", timestamp_time);
            let text_size = get_text_size(
                &timestamp_font,
                Scale::uniform(args.timestamp_font_size),
                &timestamp_text,
            );
            let rectangle_hpadding = args.timestamp_horizontal_padding;
            let rectangle_vpadding = args.timestamp_vertical_padding;

            let (upper_left, size) = compute_timestamp_position(
                args,
                x,
                y,
                text_size,
                &desired_size,
                rectangle_hpadding,
                rectangle_vpadding,
            );

            if !args.timestamp_border_mode {
                let timestamp_border_colour = decode_hex(&args.timestamp_border_colour);
                draw_filled_rounded_rect_mut(
                    &mut image,
                    Rect::at(upper_left.x as i32, upper_left.y as i32).of_size(size.x, size.y),
                    timestamp_border_colour,
                    args.timestamp_border_radius,
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
        x += desired_size.x + args.grid_horizontal_spacing;

        // update y position
        if (i as u32 + 1) % args.grid.x == 0 {
            y += desired_size.y + args.grid_vertical_spacing;
        }

        // update x position
        if (i as u32 + 1) % args.grid.x == 0 {
            x = args.grid_horizontal_spacing;
        }
    }

    match args.metadata_position {
        MetadataPosition::Top => {
            image::imageops::replace(&mut image, &mut metadata_image, 0, 0);
        }
        MetadataPosition::Bottom => {
            y += args.grid_vertical_spacing;
            image::imageops::replace(&mut image, &mut metadata_image, 0, y);
        }
        MetadataPosition::Hidden => {
            info!("Metadata hidden");
        }
    }
    Ok(image)
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
            .map(|g| {
                if let Some(bb) = g.pixel_bounding_box() {
                    bb.min.x
                } else {
                    0
                }
            })
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| {
                if let Some(bb) = g.pixel_bounding_box() {
                    bb.max.x
                } else {
                    0
                }
            })
            .unwrap();
        (max_x - min_x) as u32
    };
    (glyphs_width, glyphs_height)
}

/// Draws a rectangle with corners rounded to radius.
/// Panics if 2 x radius is greater than width or height
pub fn draw_filled_rounded_rect_mut<I>(image: &mut I, rect: Rect, colour: I::Pixel, radius: f32)
where
    I: GenericImage,
    I::Pixel: 'static,
{
    if rect.width() < 2 * radius as u32 || rect.height() < 2 * radius as u32 {
        panic!("Radius cannot be greater than width or height");
    }
    let mut i = 0.0;
    let mut j = 0.0;
    let float_left = rect.left() as f32;
    let float_right = rect.right() as f32;
    let float_top = rect.top() as f32;
    let float_bottom = rect.bottom() as f32;
    let float_width = rect.width() as f32;
    let float_height = rect.height() as f32;

    while i < float_width / 2.0 && j < float_height / 2.0 {
        // draw top from left to right
        imageproc::drawing::draw_line_segment_mut(
            image,
            (float_left + radius, float_top + j),
            (float_right - radius, float_top + j),
            colour,
        );
        // draw right from top to bottom
        imageproc::drawing::draw_line_segment_mut(
            image,
            (float_right - i, float_top + radius),
            (float_right - i, float_bottom - radius),
            colour,
        );
        // draw bottom from right to right
        imageproc::drawing::draw_line_segment_mut(
            image,
            (float_left + radius, float_bottom - j),
            (float_right - radius, float_bottom - j),
            colour,
        );
        // draw left from top to bottom
        imageproc::drawing::draw_line_segment_mut(
            image,
            (float_left + i, float_top + radius),
            (float_left + i, float_bottom - radius),
            colour,
        );

        j += 1.0;
        i += 1.0;
    }

    let radius = radius as i32;

    imageproc::drawing::draw_filled_circle_mut(
        image,
        (
            rect.left() + rect.width() as i32 / 2,
            rect.top() + rect.height() as i32 / 2,
        ),
        1,
        colour,
    );
    imageproc::drawing::draw_filled_circle_mut(
        image,
        (rect.left() + radius, rect.top() + radius),
        radius,
        colour,
    );
    imageproc::drawing::draw_filled_circle_mut(
        image,
        (rect.right() - radius, rect.top() + radius),
        radius,
        colour,
    );
    imageproc::drawing::draw_filled_circle_mut(
        image,
        (rect.left() + radius, rect.bottom() - radius),
        radius,
        colour,
    );
    imageproc::drawing::draw_filled_circle_mut(
        image,
        (rect.right() - radius, rect.bottom() - radius),
        radius,
        colour,
    );
}
