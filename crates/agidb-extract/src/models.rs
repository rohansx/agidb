//! Pinned model references.
//!
//! Updating a model = a code change here + a new SHA. The `sha256` is a
//! placeholder until the first `model_manager::ensure_cached` call
//! downloads the artifact and the maintainer pins the real digest into
//! this file.

/// One pinned reference to a model artifact on HuggingFace.
#[derive(Debug, Clone)]
pub struct ModelRef {
    pub repo: &'static str,
    pub revision: &'static str,
    pub sha256: &'static str,
    /// Optional file-within-repo. `None` means the standard `model.onnx`.
    pub file: Option<&'static str>,
}

/// Default GLiNER model for NER (the .onnx file). The SHA gets pinned
/// the first time `model_manager` successfully downloads + verifies the
/// artifact.
pub const GLINER_DEFAULT: ModelRef = ModelRef {
    repo: "onnx-community/gliner_large-v2.1",
    revision: "main",
    sha256: "TBD-PIN-AT-FIRST-DOWNLOAD",
    file: Some("onnx/model.onnx"),
};

/// Default GLiNER tokenizer (tokenizer.json — `gliner`/`orp` load this
/// separately from the ONNX file).
pub const GLINER_TOKENIZER_DEFAULT: ModelRef = ModelRef {
    repo: "onnx-community/gliner_large-v2.1",
    revision: "main",
    sha256: "TBD-PIN-AT-FIRST-DOWNLOAD",
    file: Some("tokenizer.json"),
};

/// Default GLiREL model for relation extraction. The repo candidate
/// gets confirmed (or swapped) in plan task 10 when actually loading.
pub const GLIREL_DEFAULT: ModelRef = ModelRef {
    repo: "jackboyla/glirel_beta",
    revision: "main",
    sha256: "TBD-PIN-AT-FIRST-DOWNLOAD",
    file: Some("model.onnx"),
};
