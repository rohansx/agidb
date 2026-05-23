//! Unit tests for ModelManager and the verify_sha256 helper.
//! No real network — these exercise the cache + offline + verify paths.

use agidb_extract::error::ExtractError;
use agidb_extract::model_manager::{verify_sha256, ModelManager};
use agidb_extract::models::ModelRef;
use tempfile::TempDir;

fn fake_model_ref() -> ModelRef {
    ModelRef {
        repo: "fake/repo",
        revision: "main",
        sha256: "TBD-PIN-AT-FIRST-DOWNLOAD",
        file: Some("model.onnx"),
    }
}

#[test]
fn cache_hit_skips_download_when_placeholder_sha() {
    let cache = TempDir::new().unwrap();
    let mgr = ModelManager::new(cache.path().to_path_buf(), false);
    let r = fake_model_ref();
    let path = mgr.cache_path(&r);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"hello onnx").unwrap();

    let got = mgr.ensure_cached(&r).expect("cache hit");
    assert_eq!(got, path);
}

#[test]
fn offline_mode_errors_on_miss() {
    let cache = TempDir::new().unwrap();
    let mgr = ModelManager::new(cache.path().to_path_buf(), true);
    let r = fake_model_ref();
    let err = mgr.ensure_cached(&r).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("offline"), "expected 'offline' in error; got: {msg}");
}

#[test]
fn verify_sha256_matches_known_hash() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("f");
    std::fs::write(&path, b"hello").unwrap();
    // sha256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
    let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
    verify_sha256(&path, expected).expect("hash matches");
}

#[test]
fn verify_sha256_errors_on_mismatch() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("f");
    std::fs::write(&path, b"hello").unwrap();
    let bogus = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let err = verify_sha256(&path, bogus).unwrap_err();
    assert!(
        matches!(err, ExtractError::InvalidArtifact(_)),
        "expected InvalidArtifact; got: {err:?}"
    );
}

#[test]
fn cache_path_is_sanitized_per_repo() {
    let cache = TempDir::new().unwrap();
    let mgr = ModelManager::new(cache.path().to_path_buf(), false);
    let r = fake_model_ref();
    let path = mgr.cache_path(&r);
    // "fake/repo" must NOT leak the slash into the path layout.
    let s = path.to_string_lossy();
    assert!(s.contains("fake_repo"), "got: {s}");
    assert!(s.ends_with("model.onnx"), "got: {s}");
}
