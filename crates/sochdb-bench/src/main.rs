//! sochdb benchmark harness — LongMemEval-S, LoCoMo, BEAM.
//!
//! Publishes the full six-metric stack (BLEU, F1, LLM-judge, token cost,
//! p95 latency, noisy-cue degradation) per the constitution article X.
//!
//! Phase 7 lands the full harness. This stub exists so the workspace compiles.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    eprintln!("sochdb-bench — pre-alpha. See docs/phases/phase-7-decision-gate.md.");
    Ok(())
}
