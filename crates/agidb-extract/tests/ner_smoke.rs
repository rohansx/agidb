//! Gated GLiNER smoke test. Requires `--features model-tests` and a
//! connected machine on first run (~hundreds of MB download). Pin the
//! real SHAs into `agidb_extract::models` after the first successful
//! download.

#![cfg(feature = "model-tests")]

use agidb_extract::models::{GLINER_DEFAULT, GLINER_TOKENIZER_DEFAULT};
use agidb_extract::ner::NerExtractor;
use std::path::PathBuf;

fn cache() -> PathBuf {
    // Avoid pulling `dirs` as a dev-dep; use the conventional path.
    let home = std::env::var("HOME").expect("HOME");
    PathBuf::from(home).join(".cache/agidb/models")
}

#[test]
fn extracts_at_least_one_known_person_or_place() {
    let ner = NerExtractor::new(
        cache(),
        GLINER_DEFAULT.clone(),
        GLINER_TOKENIZER_DEFAULT.clone(),
        vec!["Person".into(), "Place".into()],
        0.3,
    )
    .expect("load NER");

    let ents = ner
        .extract("Sarah recommended Bawri in Bandra")
        .expect("inference");

    let texts: Vec<&str> = ents.iter().map(|e| e.text.as_str()).collect();
    assert!(
        texts.contains(&"Sarah") || ents.iter().any(|e| e.entity_type == "Person"),
        "expected at least one Person entity; got {texts:?}"
    );
    assert!(
        texts.iter().any(|t| *t == "Bawri" || *t == "Bandra")
            || ents.iter().any(|e| e.entity_type == "Place"),
        "expected at least one Place entity; got {texts:?}"
    );
}
