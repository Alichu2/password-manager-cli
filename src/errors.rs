use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error performing database action.")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Ran into an unexpected issue: {0}")]
    Other(String),
}
