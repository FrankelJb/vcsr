use rusttype::Error as FontError;
use serde_json::error as serde_json_error;
use std::error::Error;
use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug, Clone)]
pub struct ArgumentError {
    pub cause: String,
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid arguments")
    }
}

// This is important for other errors to wrap this one.
impl Error for ArgumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug, Clone)]
pub struct ColourError {
    pub cause: String,
}

impl fmt::Display for ColourError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "colour invalid")
    }
}

// This is important for other errors to wrap this one.
impl Error for ColourError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug)]
pub enum VcsrError {
    ArgumentError(ArgumentError),
    ColourError(ColourError),
    GridShape,
    IntError(ParseIntError),
    FloatError(ParseFloatError),
    Io(io::Error),
    MediaError,
    NoneError,
    RustTypeError(FontError),
    StreamError(serde_json_error::Error),
    TimestampError(ArgumentError),
    VideoStreamError,
}

// Allow the use of "{}" format specifier
impl fmt::Display for VcsrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VcsrError::ArgumentError(ref cause) => write!(f, "Arguments are invalid: {}", cause),
            VcsrError::ColourError(ref cause) => write!(f, "Input colour was invalid: {}", cause),            VcsrError::GridShape => write!(f, "Grid must be of the form mxn, where m is the number of columns and n is the number of rows."),
            VcsrError::FloatError(ref cause) => write!(f, "Parse Float Error: {}", cause),
            VcsrError::IntError(ref cause) => write!(f, "Parse Int Error: {}", cause),
            VcsrError::Io(ref cause) => write!(f, "IO Error: {}", cause),
            VcsrError::MediaError => write!(f, "Could not find all media attributes."),
            VcsrError::NoneError => write!(f, "Could not unwrap None."),
            VcsrError::RustTypeError(ref cause) => write!(f, "Rust Type Font Error: {}", cause),
            VcsrError::StreamError(ref cause) => write!(f, "Stream Error: {}", cause),
            VcsrError::TimestampError(ref cause) => write!(f, "Invalid timestamps: {}", cause.cause),
            VcsrError::VideoStreamError => write!(f, "The file does not contain a video stream.")
        }
    }
}

// Allow this type to be treated like an error
impl Error for VcsrError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            VcsrError::ArgumentError(ref cause) => Some(cause),
            VcsrError::ColourError(ref cause) => Some(cause),
            VcsrError::GridShape => None,
            VcsrError::FloatError(ref cause) => Some(cause),
            VcsrError::IntError(ref cause) => Some(cause),
            VcsrError::Io(ref cause) => Some(cause),
            VcsrError::MediaError => None,
            VcsrError::NoneError => None,
            VcsrError::RustTypeError(ref cause) => Some(cause),
            VcsrError::StreamError(ref cause) => Some(cause),
            VcsrError::TimestampError(ref cause) => Some(cause),
            VcsrError::VideoStreamError => None,
        }
    }
}

impl From<io::Error> for VcsrError {
    fn from(error: io::Error) -> Self {
        VcsrError::Io(error)
    }
}

impl From<ParseFloatError> for VcsrError {
    fn from(error: ParseFloatError) -> Self {
        VcsrError::FloatError(error)
    }
}

impl From<ParseIntError> for VcsrError {
    fn from(error: ParseIntError) -> Self {
        VcsrError::IntError(error)
    }
}

impl From<FontError> for VcsrError {
    fn from(error: FontError) -> Self {
        VcsrError::RustTypeError(error)
    }
}
