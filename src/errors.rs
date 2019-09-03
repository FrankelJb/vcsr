use rusttype::Error as FontError;
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

#[derive(Debug)]
pub enum CustomError {
    ArgumentError(ArgumentError),
    GridShape,
    IntError(ParseIntError),
    FloatError(ParseFloatError),
    Io(io::Error),
    MediaError,
    NoneError,
    RustTypeError(FontError),
    VideoStreamError,
}

// Allow the use of "{}" format specifier
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CustomError::ArgumentError(ref cause) => write!(f, "Arguments are invalid. {}", cause),
            CustomError::GridShape => write!(f, "Grid must be of the form mxn, where m is the number of columns and n is the number of rows."),
            CustomError::FloatError(ref cause) => write!(f, "Parse Float Error: {}", cause),
            CustomError::IntError(ref cause) => write!(f, "Parse Int Error: {}", cause),
            CustomError::Io(ref cause) => write!(f, "IO Error: {}", cause),
            CustomError::MediaError => write!(f, "Could not find all media attributes."),
            CustomError::NoneError => write!(f, "Could not unwrap None."),
            CustomError::RustTypeError(ref cause) => write!(f, "Rust Type Font Error: {}", cause),
            CustomError::VideoStreamError => write!(f, "The file does not contain a video stream.")
        }
    }
}

// Allow this type to be treated like an error
impl Error for CustomError {
    fn description(&self) -> &str {
        match *self {
            CustomError::ArgumentError(ref cause) => &cause.cause,
            CustomError::GridShape => "Cannot create a grid with those dimensions",
            CustomError::FloatError(ref cause) => cause.description(),
            CustomError::IntError(ref cause) => cause.description(),
            CustomError::Io(ref cause) => cause.description(),
            CustomError::MediaError => "possibly corrupt media",
            CustomError::NoneError => "unable to unwrap None",
            CustomError::RustTypeError(ref cause) => cause.description(),
            CustomError::VideoStreamError => "No video stream in file",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            CustomError::ArgumentError(ref cause) => Some(cause),
            CustomError::GridShape => None,
            CustomError::FloatError(ref cause) => Some(cause),
            CustomError::IntError(ref cause) => Some(cause),
            CustomError::Io(ref cause) => Some(cause),
            CustomError::MediaError => None,
            CustomError::NoneError => None,
            CustomError::RustTypeError(ref cause) => Some(cause),
            CustomError::VideoStreamError => None,
        }
    }
}

impl From<io::Error> for CustomError {
    fn from(error: io::Error) -> Self {
        CustomError::Io(error)
    }
}

impl From<ParseFloatError> for CustomError {
    fn from(error: ParseFloatError) -> Self {
        CustomError::FloatError(error)
    }
}

impl From<ParseIntError> for CustomError {
    fn from(error: ParseIntError) -> Self {
        CustomError::IntError(error)
    }
}

impl From<FontError> for CustomError {
    fn from(error: FontError) -> Self {
        CustomError::RustTypeError(error)
    }
}
