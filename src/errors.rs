use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Ran into an unexpected issue: {0}")]
    Other(String),
}
