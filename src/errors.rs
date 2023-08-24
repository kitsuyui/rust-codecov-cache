/**
 * Error is an enum wrapping all possible errors.
 */
#[derive(Debug)]
pub enum Error {
    EnvError(std::env::VarError),
    CodecovClientError(codecov::errors::Error),
}

impl From<codecov::errors::Error> for Error {
    fn from(err: codecov::errors::Error) -> Error {
        match err {
            codecov::errors::Error::EnvError(e) => Error::EnvError(e),
            _ => Error::CodecovClientError(err),
        }
    }
}
