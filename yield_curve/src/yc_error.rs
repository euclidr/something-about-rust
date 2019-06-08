use hyper;

pub enum YCError {
    Hyper(hyper::Error),
    NotFound(String),
}