//! HuggingFace ONNX model download + cache with SHA verification.
//!
//! Patterned after `ctxgraph-extract::model_manager`, trimmed to what
//! agidb actually needs.
//!
//! **Constitutional contract:** zero network calls at read/write time.
//! Downloads happen only when the first call to `ensure_cached` finds
//! the file missing. `AGIDB_OFFLINE=1` (or `offline = true` at
//! construction) forbids downloads entirely — required-but-missing
//! becomes an error rather than a network call.

use crate::error::{ExtractError, Result};
use crate::models::ModelRef;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct ModelManager {
    cache_root: PathBuf,
    offline: bool,
}

impl ModelManager {
    pub fn new(cache_root: PathBuf, offline: bool) -> Self {
        Self {
            cache_root,
            offline,
        }
    }

    /// Where the file for `m` lives in the cache.
    ///
    /// Repo names like `"urchade/gliner_multi-v2.1"` are sanitized to
    /// `"urchade_gliner_multi-v2.1"` so the `/` doesn't accidentally
    /// nest the cache.
    pub fn cache_path(&self, m: &ModelRef) -> PathBuf {
        let repo_safe = m.repo.replace('/', "_");
        let file = m.file.unwrap_or("model.onnx");
        self.cache_root.join(repo_safe).join(m.revision).join(file)
    }

    /// Return the on-disk path to `m`, downloading + SHA-verifying if
    /// the file isn't already cached.
    pub fn ensure_cached(&self, m: &ModelRef) -> Result<PathBuf> {
        let path = self.cache_path(m);
        if path.is_file() {
            if Self::is_placeholder_sha(m.sha256) {
                tracing::warn!(model = m.repo, "SHA placeholder; skipping verify");
                return Ok(path);
            }
            verify_sha256(&path, m.sha256)?;
            return Ok(path);
        }
        if self.offline {
            return Err(ExtractError::ModelDownload(format!(
                "offline mode: required model {}/{}/{} not in cache",
                m.repo,
                m.revision,
                m.file.unwrap_or("model.onnx"),
            )));
        }
        self.download(m, &path)?;
        if !Self::is_placeholder_sha(m.sha256) {
            verify_sha256(&path, m.sha256)?;
        }
        Ok(path)
    }

    /// Treat any `sha256` starting with `"TBD-"` as "not pinned yet" —
    /// skip verification so the maintainer can pin the real digest after
    /// the first successful download.
    fn is_placeholder_sha(sha: &str) -> bool {
        sha.starts_with("TBD-")
    }

    fn download(&self, m: &ModelRef, target: &Path) -> Result<()> {
        let file = m.file.unwrap_or("model.onnx");
        let url = format!(
            "https://huggingface.co/{repo}/resolve/{rev}/{file}",
            repo = m.repo,
            rev = m.revision,
            file = file
        );
        tracing::info!(url = %url, "downloading model");
        fs::create_dir_all(target.parent().unwrap())?;
        let mut resp = reqwest::blocking::get(&url)
            .map_err(|e| ExtractError::ModelDownload(format!("get {url}: {e}")))?;
        if !resp.status().is_success() {
            return Err(ExtractError::ModelDownload(format!(
                "{url} -> HTTP {}",
                resp.status()
            )));
        }
        // Atomic write: stream to `.part`, rename into place.
        let part = target.with_extension("part");
        let mut out = fs::File::create(&part)?;
        resp.copy_to(&mut out)
            .map_err(|e| ExtractError::ModelDownload(format!("copy_to: {e}")))?;
        fs::rename(&part, target)?;
        Ok(())
    }
}

/// Stream `path` through SHA-256 and compare to `expected_hex`.
/// Returns `ExtractError::InvalidArtifact` on mismatch.
pub fn verify_sha256(path: &Path, expected_hex: &str) -> Result<()> {
    let mut f = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let got = format!("{:x}", hasher.finalize());
    if got != expected_hex {
        return Err(ExtractError::InvalidArtifact(format!(
            "sha256 mismatch on {}: got {got}, expected {expected_hex}",
            path.display()
        )));
    }
    Ok(())
}
