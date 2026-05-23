//! Typed errors for layer-2 extraction.
//!
//! Converts to [`agidb_core::AgidbError::Extraction`] at the agidb-core
//! boundary so the engine sees a single error surface and callers above
//! the crate only need to handle `AgidbError`.

use agidb_core::AgidbError;

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    /// ONNX model failed to load from disk.
    #[error("model load: {0}")]
    ModelLoad(String),

    /// HuggingFace download or SHA verify failed.
    #[error("model download: {0}")]
    ModelDownload(String),

    /// ORT session.run() returned an error.
    #[error("ort inference: {0}")]
    Inference(String),

    /// Tokenizer failed to encode input.
    #[error("tokenize: {0}")]
    Tokenize(String),

    /// std::io error during model file IO.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// SHA mismatch or malformed model artifact.
    #[error("invalid model artifact: {0}")]
    InvalidArtifact(String),
}

impl From<ExtractError> for AgidbError {
    fn from(e: ExtractError) -> Self {
        AgidbError::Extraction(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ExtractError>;
