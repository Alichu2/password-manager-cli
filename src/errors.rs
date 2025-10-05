use magic_crypt::MagicCryptError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing number.")]
    ParsingError,
    #[error("Could not find the field {0} (line {1}).")]
    MissingField(&'static str, usize),
    #[error("Could not find the header {0}.")]
    NoHeader(String),
    #[error("Invalid headers. There needs to be at least 4 columns.")]
    BadHeaders,
    #[error("Error processing database dump.")]
    BadDump,
    #[error("Bad directory.")]
    BadDir,
    #[error("No input.")]
    EmptyInput,
    #[error("Found unexpected table `{0}`.")]
    UnexpectedTable(String),
    #[error("Could not find a valid database. Perhaps you forgot to generate it?")]
    MissingDatabase,
    #[error("Save file and key already exists. Cannot regenerate.")]
    SaveFileExists,
    #[error("Cannot create save directory.")]
    DirError,
    #[error("Cannot read user input.")]
    ReadError,
    #[error("Could not verify.")]
    VerificationError,
    #[error("Could not decrypt.")]
    BadDecryption(#[from] MagicCryptError),
    #[error("Could not find a password for the place {0}.")]
    NoPassword(String),
    #[error("Error performing database action ({0}).")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Ran into an unexpected issue: {0}")]
    Other(String),
}
