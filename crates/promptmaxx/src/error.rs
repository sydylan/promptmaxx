use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("Prompt already exists")]
    Duplicate,
}

pub type Result<T> = std::result::Result<T, Error>;
