# phase 2 — storage

**duration:** weeks — (inherited from sochdb v1)
**status:** complete (inherited from sochdb v1)
**depends on:** [phase 1](./phase-1-hdc-kernel.md)

## goal

land layer 3: redb metadata + mmap'd signatures + bi-temporal schema. open, observe with a placeholder extractor, close, reopen, recall by exact match must work end to end.

## deliverables

- [x] `agidb-core/src/store.rs` with redb tables:
  - `episodes` — episode_id → row (text, signature_offset, triples, timestamps, provenance, confidence)
  - `triples` — triple_id → row (subject, predicate, object, confidence, episode_id)
  - `concepts` — concept_id → canonical_name + aliases + entity_type
  - `concept_index` — entity_name → list of episode_ids
  - `inverted_index` — active_dim → roaring_bitmap of episode_ids
  - `semantic_atoms` — atom_id → row (statement, concept, evidence_count, last_seen, signature_offset)
  - `consolidation_log` — append-only audit
- [x] `agidb-core/src/signatures.rs` — `signatures.dat` with append + offset lookup; memmap2-backed
- [x] bi-temporal columns on every fact: `t_valid_start`, `t_valid_end`, `t_tx_start`, `superseded_by`
- [x] crash-safety tests:
  - kill mid-write → reopen recovers consistently
  - torn writes detected via redb checksum
- [x] export / import via jsonl (`agidb export`, `agidb import`)
- [x] `manifest.toml` with `format_version`

## exit criterion

`open → observe (with placeholder regex extractor) → close → reopen → recall by exact entity match` works end to end with ACID guarantees at the episode level. crash tests pass.

## tasks

1. define the redb table schemas (no code yet, just the types)
2. write the bi-temporal invariants as property tests
3. implement append-only signatures.dat
4. wire redb tables behind a `Store` trait
5. write the placeholder regex extractor (5 patterns, throwaway — phase 3 replaces it)
6. wire `Agidb::open` / `observe` / `close` / `recall_exact`
7. write the crash-safety harness using `nix::sys::signal::kill` against a child process
8. land export/import to jsonl

## risks

| risk | mitigation |
|---|---|
| redb format churn between versions | pin a redb version; document migration path in `manifest.toml` |
| mmap on windows differs from unix | test windows in CI from day one |
| inverted index size for dense signatures | roaring bitmaps; cap active dims per signature in HDC encoding |
| fsync cost on every write | batch via a configurable flush policy (`sync_every_n` default 1) |

## what unblocks next

phase 3 needs `observe()` callable with a real extractor swapped in. phase 4 needs the inverted index populated for the similarity scan.

## references

- [architecture/layer-3-storage.md](../architecture/layer-3-storage.md) — schema rationale
- [architecture/architecture.md](../architecture/architecture.md#the-write-path) — where this plugs in
