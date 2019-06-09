use hyper;
use std::error;
use std::fmt;
use http::uri::InvalidUri;

#[derive(Debug)]
pub enum YCError {
    Hyper(hyper::Error),
    InvalidResponse(String),
    DataNotFound(String),
    ParseURL(InvalidUri),
}

impl fmt::Display for YCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YCError::Hyper(err) => write!(f, "Hyper error: {}", err),
            YCError::InvalidResponse(err) => write!(f, "Invalid response: {}", err),
            YCError::DataNotFound(err) => write!(f, "Data not found: {}", err),
            YCError::ParseURL(err) => write!(f, "parse url error: {}", err),
        }
    }
}

impl error::Error for YCError {
    fn description(&self) -> &str {
        match self {
            YCError::Hyper(err) => err.description(),
            YCError::InvalidResponse(err) => err,
            YCError::DataNotFound(err) => err,
            YCError::ParseURL(err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            YCError::Hyper(err) => Some(err),
            YCError::ParseURL(err) => Some(err),
            _ => None,
        }
    }
}
