use hyper;
use std::error;
use std::fmt;
use std::convert::From;
use http::uri::InvalidUri;
use http::header::ToStrError;
use url::ParseError;

#[derive(Debug)]
pub enum YCError {
    Hyper(hyper::Error),
    InvalidResponse(String),
    DataNotFound(String),
    InvalidUri(InvalidUri),
    ToStrError(ToStrError),
    UrlParseError(ParseError),
    InvalidData,
}

impl fmt::Display for YCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YCError::Hyper(err) => write!(f, "Hyper error: {}", err),
            YCError::InvalidResponse(err) => write!(f, "Invalid response: {}", err),
            YCError::DataNotFound(err) => write!(f, "Data not found: {}", err),
            YCError::InvalidUri(err) => write!(f, "invalide uri: {}", err),
            YCError::ToStrError(err) => write!(f, "to string error: {}", err),
            YCError::UrlParseError(err) => write!(f, "url parse error: {}", err),
            YCError::InvalidData => write!(f, "invalid data"),
        }
    }
}

impl error::Error for YCError {
    fn description(&self) -> &str {
        match self {
            YCError::Hyper(err) => err.description(),
            YCError::InvalidResponse(err) => err,
            YCError::DataNotFound(err) => err,
            YCError::InvalidUri(err) => err.description(),
            YCError::ToStrError(err) => err.description(),
            YCError::UrlParseError(err) => err.description(),
            YCError::InvalidData => "invalid data",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            YCError::Hyper(err) => Some(err),
            YCError::InvalidUri(err) => Some(err),
            YCError::ToStrError(err) => Some(err),
            YCError::UrlParseError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ToStrError> for YCError {
    fn from(err: ToStrError) -> YCError {
        YCError::ToStrError(err)
    }
}

impl From<InvalidUri> for YCError {
    fn from(err: InvalidUri) -> YCError {
        YCError::InvalidUri(err)
    }
}

impl From<ParseError> for YCError {
    fn from(err: ParseError) -> YCError {
        YCError::UrlParseError(err)
    }
}

impl From<hyper::Error> for YCError {
    fn from(err: hyper::Error) -> YCError {
        YCError::Hyper(err)
    }
}