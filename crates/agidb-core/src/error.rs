//! Typed error model for agidb-core.
//!
//! Public API surfaces `Result<T, AgidbError>`. Per the constitution
//! errors are always actionable — no swallowed errors, no panics in
//! the public path.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgidbError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(String),

    #[error("transaction error: {0}")]
    Transaction(String),

    #[error("extraction failed: {0}")]
    Extraction(String),

    #[error("signature corruption at offset {offset} in {path:?}")]
    CorruptSignature { offset: u64, path: PathBuf },

    #[error("signature offset {offset} out of bounds (len {len})")]
    SignatureOutOfBounds { offset: u64, len: u64 },

    #[error("invalid query: {0}")]
    InvalidQuery(String),

    #[error("concept not found: {0}")]
    UnknownConcept(String),

    #[error("episode not found: {0}")]
    UnknownEpisode(u64),

    #[error("goal not found: {0}")]
    UnknownGoal(u64),

    #[error("belief not found: {0}")]
    UnknownBelief(u64),

    #[error("invalid goal transition: {0}")]
    InvalidGoalTransition(String),

    #[error("format version mismatch (got {got}, expected {expected})")]
    FormatVersion { got: u32, expected: u32 },

    #[error("internal invariant violated: {0}")]
    Internal(String),
}

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, AgidbError>;

// redb errors are typed but we collapse them to strings at the public
// boundary so consumers don't take a transitive redb dependency for
// error matching.
impl From<redb::DatabaseError> for AgidbError {
    fn from(e: redb::DatabaseError) -> Self {
        AgidbError::Database(e.to_string())
    }
}

impl From<redb::TransactionError> for AgidbError {
    fn from(e: redb::TransactionError) -> Self {
        AgidbError::Transaction(e.to_string())
    }
}

impl From<redb::TableError> for AgidbError {
    fn from(e: redb::TableError) -> Self {
        AgidbError::Database(e.to_string())
    }
}

impl From<redb::StorageError> for AgidbError {
    fn from(e: redb::StorageError) -> Self {
        AgidbError::Database(e.to_string())
    }
}

impl From<redb::CommitError> for AgidbError {
    fn from(e: redb::CommitError) -> Self {
        AgidbError::Transaction(e.to_string())
    }
}
