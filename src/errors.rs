use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)] // Allow the use of "{:?}" format specifier
pub enum CustomError {
    IntError(ParseIntError),
    GridShape,
}

// Allow the use of "{}" format specifier
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CustomError::IntError(ref cause) => write!(f, "Parse Int Error: {}", cause),
            CustomError::GridShape => write!(f, "Grid must be of the form mxn, where m is the number of columns and n is the number of rows."),
        }
    }
}

// Allow this type to be treated like an error
impl Error for CustomError {
    fn description(&self) -> &str {
        match *self {
            CustomError::IntError(ref cause) => cause.description(),
            CustomError::GridShape => "Unknown error!",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            CustomError::IntError(ref cause) => Some(cause),
            CustomError::GridShape => None,
        }
    }
}

impl From<ParseIntError> for CustomError {
    fn from(error: ParseIntError) -> Self {
        CustomError::IntError(error)
    }
}