use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ModelError {
    #[error("io error while reading data: {0}")]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    String(#[from] StringError),
    #[error("referenced data to {data} is out of bounds at {offset}")]
    OutOfBounds { data: &'static str, offset: usize },
    #[error("Trying to read past the end of the file")]
    Eof(usize),
}

#[derive(Debug, Error)]
pub enum StringError {
    #[error(transparent)]
    NonUTF8(#[from] std::str::Utf8Error),
    #[error("String is not null-terminated")]
    NotNullTerminated,
}
