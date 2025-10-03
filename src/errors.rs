use magic_crypt::MagicCryptError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot read user input.")]
    ReadError,
    #[error("Could not verify.")]
    VerificationError,
    #[error("Could not decrypt.")]
    BadDecryption(#[from] MagicCryptError),
    #[error("Could not find a password for the place {0}.")]
    NoPassword(String),
    #[error("Error performing database action.")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Ran into an unexpected issue: {0}")]
    Other(String),
}
