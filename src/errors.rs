// use rusttype::Error as FontError;
use image::ImageError;
use serde_json::error as serde_json_error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VcsrError {
    #[error("Arguments are invalid: `{0}`")]
    ArgumentError(String),
    #[error("Input colours are invalid: `{0}`")]
    ColourError(String),
    #[error("Grid must be of the form mxn, where m is the number of columns and n is the number of rows.")]
    GridShape,
    #[error(transparent)]
    ImageError(#[from] ImageError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    IntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    FloatError(#[from] std::num::ParseFloatError),
    #[error("Could not find all media attributes")]
    MediaError,
    #[error("Cannot unwrap none")]
    NoneError,
    #[error("Rust Type Font Error")]
    RustTypeError,
    #[error("Stream Error")]
    StreamError(#[from] serde_json_error::Error),
    #[error("Invalid timestamps: `{0}`")]
    TimestampError(String),
    #[error("The file does not contain a video stream.")]
    VideoStreamError,
}
