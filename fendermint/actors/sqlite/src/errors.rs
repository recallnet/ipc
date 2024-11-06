use fil_actors_runtime::ActorError;
use rusqlite::Error as SQLiteError;
use sqlite_vfs::RegisterError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    Passthrough(String),
    SQLiteError(SQLiteError),
    RegisterError(RegisterError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Passthrough(s) => write!(f, "{}", s),
            Error::SQLiteError(e) => e.fmt(f),
            Error::RegisterError(e) => e.fmt(f),
        }
    }
}

impl From<SQLiteError> for Error {
    fn from(value: SQLiteError) -> Self {
        Error::SQLiteError(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Passthrough(value)
    }
}

impl From<RegisterError> for Error {
    fn from(value: RegisterError) -> Self {
        Error::RegisterError(value)
    }
}

impl From<Error> for ActorError {
    fn from(value: Error) -> Self {
        match value {
            Error::Passthrough(s) => ActorError::illegal_state(s),
            Error::SQLiteError(e) => ActorError::illegal_state(e.to_string()),
            Error::RegisterError(e) => ActorError::illegal_state(e.to_string()),
        }
    }
}
