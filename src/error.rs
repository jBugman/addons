#[derive(Debug)]
pub enum Error {
    New(&'static str),
    IOError(std::io::Error),
    VersionError(String),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<semver_parser::parser::Error<'_>> for Error {
    fn from(e: semver_parser::parser::Error) -> Self {
        Error::VersionError(format!("{}", e))
    }
}
