use thiserror::Error;

#[derive(Error, Debug)]
pub enum CcgenError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(String),
}
