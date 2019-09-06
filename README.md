# Video Contact Sheets (in) Rust
A video contact sheet generator written in rust.

[![Build Status](https://travis-ci.com/FrankelJb/vcsr.svg?branch=master)](https://travis-ci.com/FrankelJb/vcsr)

## Reasons for existing
This application is a direct port of [vcsi](https://github.com/amietn/vcsi). Full credit goes to the author for the algorithm and design.

I am learning Rust and while working on another project, I required a contact sheet generator. The existing generators needed some other tools during installation and I wanted something that I could just drop in and use. The process allowed me to focus on the language's ergonomics rather than the algorithm.

## Installation
I wanted the installation process to be as simple as possible. Download the archive from the [releases](https://github.com/FrankelJb/vcsr/releases) for your platform (maybe Windows sometime in the future). Extract the binary and put it somewhere that's on your `$PATH`.

### Requirements
`ffmpeg` and `ffprobe` need to be installed.

## Usage
### Examples
The simplest usage is to run it with no extra arguments other than an input file. See the full list of arguments below for default values.
```
$ vcsr bbb_sunflower_1080p_60fps_normal.mp4
```
![Simple usage example](https://raw.githubusercontent.com/FrankelJb/vcsr/a9f8c61c37505efd043b7b43e3aff4b723e23b51/bbb_sunflower_1080p_60fps_normal.mp4.jpg)

### Arguments
```
$ vcsr -h
vcsr 0.1.0

USAGE:
    vcsr [FLAGS] [OPTIONS] <filenames>...

FLAGS:
    -a, --accurate                 Make accurate captures. This capture mode is way slower than the default one but it
                                   helps when capturing frames from HEVC videos.
    -S, --actual-size              Make thumbnails of actual size. In other words, thumbnails will have the actual 1:1
                                   size of the video resolution.
        --fast                     Fast mode. Just make a contact sheet as fast as possible, regardless of output image
                                   quality. May mess up the terminal.
    -h, --help                     Prints help information
        --ignore-errors            Ignore any error encountered while processing files recursively and continue to the
                                   next file.
        --no-overwrite             Do not overwrite output file if it already exists, simply ignore this file and
                                   continue processing other unprocessed files.
        --no-shadow                show dropshadow on frames
    -r, --recursive                Process every file in the specified directory recursively
    -t, --show-timestamp           display timestamp for each frame
        --timestamp-border-mode    Draw timestamp text with a border instead of the default rectangle.
    -V, --version                  Prints version information
    -v, --verbose                  display verbose messages

OPTIONS:
    -A, --accurate-delay-seconds <accurate-delay-seconds>
            Fast skip to N seconds before capture time, then do accurate capture (decodes N seconds of video before each
            capture). This is used with accurate capture mode only.
        --background-colour <background-colour>
            Color of the timestamp background rectangle in hexadecimal, for example AABBCC [default: ffffff00]

        --capture-alpha <capture-alpha>
            Alpha channel value for the captures (transparency in range [0, 255]). Defaults to 255 (opaque) [default:
            255]
        --delay-percent <delay-percent>
            do not capture frames in the first and last n percent of total time

        --end-delay-percent <end-delay-percent>
            do not capture frames in the last n percent of total time [default: 7]

        --exclude-extensions <exclude-extensions>...
            Do not process files that end with the given extensions.

        --frame-type <frame-type>
            Frame type passed to ffmpeg 'select=eq(pict_type,FRAME_TYPE)' filter. Should be one of ('I', 'B', 'P') or
            the special type 'key' which will use the 'select=key' filter instead.
    -g, --grid <grid>
            display frames on a mxn grid (for example 4x5). The special value zero (as in 2x0 or 0x5 or 0x0) is only
            allowed when combined with --interval or with --manual. Zero means that the component should be
            automatically deduced based on other arguments passed. [default: 4x4]
        --grid-horizontal-spacing <grid-horizontal-spacing>
            number of pixels spacing captures horizontally [default: 15]

        --grid-spacing <grid-spacing>
            number of pixels spacing captures both vertically and horizontally

        --grid-vertical-spacing <grid-vertical-spacing>
            number of pixels spacing captures vertically [default: 15]

    -f, --format <image-format>
            Output image format. Can be any format supported by image-rs. For example 'png' or 'jpg'. [default:
            jpg]
        --interval <interval>
            Capture frames at specified interval. Interval format is any string supported by `humantime`. For example
            '5m', '3 minutes 5 seconds', '1 hour 15 min and 20 sec' etc.
    -m, --manual <manual-timestamps>...
            Space-separated list of frame timestamps to use, for example 1:11:11.111 2:22:22.222

        --metadata-background-colour <metadata-background-colour>
            Color of the metadata background in hexadecimal, for example AABBCC [default: b0cd7b0a]

        --metadata-font <metadata-font>                                  Path to TTF font used for metadata
        --metadata-font-colour <metadata-font-colour>
            Color of the metadata font in hexadecimal, for example AABBCC [default: ffffff00]

        --metadata-font-size <metadata-font-size>
            size of the font used for metadata [default: 32]

        --metadata-horizontal-margin <metadata-horizontal-margin>
            Horizontal margin (in pixels) in the metadata header. [default: 15]

        --metadata-margin <metadata-margin>
            Margin (in pixels) in the metadata header. [default: 15]

        --metadata-position <metadata-position>
            Position of the metadata header. [default: top]  [possible values: Top, Bottom,
            Hidden]
        --metadata-vertical-margin <metadata-vertical-margin>
            Vertical margin (in pixels) in the metadata header. [default: 10]

    -s, --num-samples <num-samples>                                      number of samples
    -o, --output <output-path>                                           save to output file
        --start-delay-percent <start-delay-percent>
            do not capture frames in the first n percent of total time [default: 7]

    -O, --thumbnail-output-path <thumbnail-output-path>
            Save thumbnail files to the specified output directory. If set, the thumbnail files will not be deleted
            after successful creation of the contact sheet.
        --timestamp-background-colour <timestamp-background-colour>
            Color of the timestamp background rectangle in hexadecimal, for example AABBCC [default: 000000aa]

        --timestamp-border-colour <timestamp-border-colour>
            Color of the timestamp border in hexadecimal, for example AABBCC [default: 000000]

        --timestamp-border-radius <timestamp-border-radius>
            Draw timestamp text with a border instead of the default rectangle. [default: 1.0]

        --timestamp-border-size <timestamp-border-size>
            Size of the timestamp border in pixels (used only with --timestamp-border-mode). [default: 1]

        --timestamp-font <timestamp-font>                                Path to TTF font used for timestamps
        --timestamp-font-colour <timestamp-font-colour>
            Color of the timestamp font in hexadecimal, for example AABBCC [default: ffffff]

        --timestamp-font-size <timestamp-font-size>
            size of the font used for timestamps [default: 12]

        --timestamp-horizontal-margin <timestamp-horizontal-margin>       [default: 5]
        --timestamp-horizontal-padding <timestamp-horizontal-padding>
            Horizontal padding (in pixels) for timestamps. [default: 3]

    -T, --timestamp-position <timestamp-position>
            Timestamp position. [default: se]  [possible values: North, South, East,
            West, NE, NW, SE, SW, Center]
        --timestamp-vertical-margin <timestamp-vertical-margin>
            Vertical margin (in pixels) for timestamps. [default: 5]

        --timestamp-vertical-padding <timestamp-vertical-padding>
            V ertical padding (in pixels) for timestamps. [default: 1]

    -w, --width <vcs-width>
            width of the generated contact sheet [default: 1500]


ARGS:
    <filenames>...
```