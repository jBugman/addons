#[derive(Debug)]
pub(crate) enum Error {
    New(&'static str),
    IOError(std::io::Error),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}
