use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault Error: {0}")]
    DecryptionError(&'static str),
    #[error("Initialization: {0}")]
    InitializationError(&'static str),
    #[error("Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Input error: {0}")]
    InputError(#[from] inquire::InquireError),

    #[error("Time error: {0}")]
    TimeError(#[from] std::time::SystemTimeError),

    #[error("{0}")]
    Other(&'static str),
}
