# sochdb — biological memory mapping

this doc explicitly maps the five tiers of biological memory to their representations in sochdb. it answers two questions:

1. *"how does sochdb actually relate to how the brain remembers?"* — for investors, journalists, skeptics, and anyone evaluating whether the "brain-inspired" framing is real or marketing.
2. *"what's in scope and what's not?"* — for contributors and integrators who need to know where sochdb's boundary lives.

## the five tiers, mapped

cognitive psychology generally recognizes five memory systems with distinct neural substrates, timescales, and access modes. sochdb maps each one explicitly.

| biological tier | timescale | what it stores | sochdb equivalent | in scope for v0.1? |
|---|---|---|---|---|
| sensory memory | <1 second | raw perceptual signal before perception | upstream of sochdb | no — explicitly out of scope |
| working memory | seconds to minutes, ~7 items | currently active context | session-scoped recall with recency boost | yes, lightweight |
| episodic memory | hours to lifetime | autobiographical events with time, place, people | the core `Episode` type with bi-temporal stamps | yes, this is sochdb's strongest tier |
| semantic memory | indefinite | facts decoupled from when they were learned | `SemanticAtom` produced by consolidation | yes |
| procedural memory | indefinite | how to do things, skills, workflows | `Procedure` (typed episode shape) | partial — storage yes, special retrieval deferred |

three are first-class. one is lightweight. one is deliberately out of scope.

## tier 1 — sensory memory (out of scope)

**biology:** raw sensory signal held for milliseconds before it gets processed into perception. iconic memory (visual, ~250ms), echoic memory (auditory, ~3s), haptic memory (touch). the function is to give the brain a tiny buffer to integrate signals across time before they're either perceived consciously or discarded.

**agent analog:** input pipelines. audio frames before transcription. screen captures before OCR. webcam frames before vision-model inference. raw network packets before parsing.

**why out of scope:** this is a streaming concern, not a database concern. it happens upstream of any memory system. a memory database that tried to buffer raw sensory input would either be a streaming system pretending to be a database, or a database doing a streaming system's job badly. sochdb is the long-term store; sensory buffering belongs in the agent's input pipeline.

**what we expect upstream:** the integrator's input pipeline transcribes / OCRs / captures into text observations and passes them to `sochdb.observe()`. sochdb takes over from there.

## tier 2 — working memory (lightweight)

**biology:** the active workspace. roughly seven items, held for seconds to minutes. located primarily in prefrontal cortex. function: keep the immediately-relevant pieces of information available for current reasoning. it's not storage so much as *active attention*.

**agent analog:** the LLM's context window plus any active tool state. the chunks of long-term memory currently surfaced into the prompt. the recent conversation turns. the in-progress chain-of-thought.

**sochdb's role:** sochdb sits underneath working memory. the LLM's context window is the working memory; sochdb is the long-term store from which working memory gets populated. but sochdb plays one key role in *making* working memory work:

> when an agent calls `recall()`, results from the current session should weight higher than results from a year ago.

this is the "freshness gradient" that makes recent context feel like working memory and older context feel like archival memory. sochdb implements this via:

1. **session scoping.** `ObserveOpts::session_id` lets you tag observations with the session they came from. `Query::session_id` lets you scope recall to a session or boost results from that session.
2. **recency weighting.** confidence scores in recall results are multiplied by a recency factor — recent episodes (transaction time within the last hour) get a small boost, very recent (within the current session) get a larger one.
3. **the implicit current session.** if `Query::session_id` is unset, sochdb infers it from the most recent observe — recall behaves as if the agent is "still in the current conversation."

this gives the agent a working-memory-like feel without sochdb having to actually maintain a separate working-memory store. the same database serves long-term and active recall; the query parameters determine which feels active.

## tier 3 — episodic memory (core)

**biology:** autobiographical events tied to a specific time, place, and set of people. "I met Sarah at PyCon last year." stored initially in the hippocampus, gradually consolidated to the neocortex over weeks to years. function: remember the specifics of what happened to you, when, and with whom.

**agent analog:** the agent's interaction history. every observation it made, every conversation turn, every tool result, every user statement. with timestamps and identity attribution.

**sochdb's representation:** the `Episode` is the primary data type in sochdb. every call to `observe()` produces an episode with:

- raw text of the observation
- extracted triples (subject, predicate, object)
- entities with canonical names and types
- valid time (when it was true in the world)
- transaction time (when sochdb received it)
- provenance (who or what produced this observation)
- HDC signature (the bound-and-bundled representation used for retrieval)
- confidence score

episodes are immutable. updates happen via supersession — a new episode is written, and the older one gets `t_valid_end` set and `superseded_by` linked. nothing is destroyed.

**this is sochdb's strongest tier.** the bi-temporal model, the provenance tracking, the supersession semantics, the HDC signatures — they all serve episodic memory first. semantic memory is a derivative.

## tier 4 — semantic memory (consolidated)

**biology:** factual knowledge decoupled from the specific event during which you learned it. "Python is interpreted." "Mumbai is in India." you don't remember when or where you learned these — you just know them. neuroscience suggests semantic memories form gradually from repeated episodic experiences during sleep-driven consolidation, eventually living mostly in the neocortex independent of the hippocampus.

**agent analog:** consolidated knowledge the agent has acquired across many sessions. "Sarah usually wants meetings in the morning." "the production deploy script is `./deploy.sh prod`." "this customer prefers email over slack." these aren't tied to a single conversation; they're patterns across many.

**sochdb's representation:** the `SemanticAtom` type, produced by the consolidation worker.

```rust
struct SemanticAtom {
    concept:          ConceptId,       // what the fact is about
    statement:        String,           // canonical form of the fact
    signature:        HV,               // bundled signature over evidence
    evidence:         Vec<EpisodeId>,   // source episodes
    evidence_count:   u32,              // how many episodes support it
    confidence:       f32,
    first_consolidated: DateTime<Utc>,
    last_referenced:    DateTime<Utc>,
}
```

semantic atoms emerge from the consolidation worker (see ARCHITECTURE.md → consolidation loop). when N episodes share a similar bound pattern (same subject + predicate, similar object signatures, overlapping time windows), the worker:

1. bundles their signatures into one semantic signature
2. creates a `SemanticAtom` with the canonical statement
3. links it to the source episodes (provenance preserved)
4. sets `evidence_count = N`

after consolidation, `recall()` returns semantic atoms alongside episodic matches — typically with higher confidence because they have more evidence behind them. the user can filter `Recall::semantic_atoms` vs `Recall::episodes` separately.

**the episodic-semantic split in action:**

an agent asks "what does Sarah like to eat?"

- *episodic answer:* "on April 12, Sarah said she liked the thai place. on April 28, Sarah ordered thai again. on May 9, Sarah recommended Bawri (thai)."
- *semantic answer:* "Sarah likes thai food. (evidence: 7 episodes, confidence 0.91)"

both are correct. the agent gets both. for "give me the latest specific event," the episodic answer wins. for "what's generally true," the semantic answer wins. sochdb returns both with explicit type tags so the agent can choose.

## tier 5 — procedural memory (partial)

**biology:** how to do things. motor skills (riding a bike), perceptual skills (recognizing faces), and cognitive routines (multiplying two numbers). stored in basal ganglia, cerebellum, and motor cortex — completely separate substrate from declarative (episodic + semantic) memory. function: execute learned behaviors without conscious effort.

**agent analog:** workflows, tool-use patterns, skill recipes. "when the user asks about a github issue, first fetch the issue, then check related PRs, then summarize." "to deploy to staging, run `./deploy.sh staging --skip-tests`." "to greet a returning user, check their last session's context first."

**sochdb's representation:** a typed episode shape called `Procedure`.

```rust
struct Procedure {
    name:           String,           // canonical handle
    description:    String,           // human-readable summary
    trigger:        String,           // when to invoke (natural language)
    preconditions:  Vec<String>,      // what must be true before
    steps:          Vec<ProcedureStep>,
    postconditions: Vec<String>,      // what should be true after
    success_count:  u32,
    failure_count:  u32,
    last_invoked:   Option<DateTime<Utc>>,
}

struct ProcedureStep {
    description: String,
    tool:        Option<String>,      // tool to call, if applicable
    args:        Option<Value>,
}
```

procedures get stored via:

```rust
db.observe_procedure(Procedure {
    name: "deploy_to_staging".into(),
    trigger: "when user wants to deploy to staging".into(),
    steps: vec![
        ProcedureStep {
            description: "verify tests pass".into(),
            tool: Some("run_tests".into()),
            args: None,
        },
        ProcedureStep {
            description: "run the deploy script".into(),
            tool: Some("shell".into()),
            args: Some(json!({"cmd": "./deploy.sh staging"})),
        },
    ],
    ...
}).await?;
```

and surface from `recall()` like any other typed episode — when the cue matches a procedure's `trigger`, the procedure ranks in the result set.

**what we're explicitly NOT doing in v0.1:**

- specialized procedural retrieval ("find the right procedure for this situation" with success-rate-weighted ranking)
- procedure composition (chaining procedures into bigger procedures)
- procedure execution (sochdb stores procedures but doesn't run them — that's the agent framework's job)
- skill abstraction (learning new procedures from observed successful sequences)

these are v0.2+. the v0.1 commitment is: **procedural memory is a first-class storage type, retrievable like any other episode.** that's enough to plant the flag on "sochdb covers all five biological memory tiers."

## why this matters for the pitch

three claims this mapping unlocks:

1. **"sochdb is the first agent memory database with explicit mappings to all five biological memory tiers."** mem0 talks about episodic, semantic, and procedural (since 2026) but doesn't have a working-memory story or a clean episodic-semantic separation. letta has explicit memory tiers but they're OS-inspired (RAM/disk), not biology-inspired. nobody else covers all five.

2. **"sochdb's design is grounded in 50 years of cognitive neuroscience."** Tulving's episodic-semantic distinction (1972), Baddeley's working memory model (1974), Squire's declarative-procedural distinction (1980s), McClelland-McNaughton-O'Reilly's complementary learning systems (1995). these aren't just citations; they directly inform sochdb's data model and consolidation behavior.

3. **"sochdb is the missing primitive between input pipelines and agent frameworks."** sensory buffering happens upstream (transcription, OCR, capture). agent reasoning happens downstream (LLM, working memory, tool use). sochdb owns the middle tier — long-term episodic, semantic, and procedural storage with biology-inspired consolidation.

## the boundary diagram

```
┌──────────────────────────────────────────────────────────────────┐
│ UPSTREAM: input pipeline                                         │
│   audio → transcription → text                                   │
│   screen → OCR → text                                            │
│   sensors → parsing → structured events                          │
│                                                                  │
│   ↑ sensory memory lives here, NOT in sochdb                     │
└────────────────────────────┬─────────────────────────────────────┘
                             │
                             ▼  observe(text, opts)
┌──────────────────────────────────────────────────────────────────┐
│ SOCHDB: long-term memory database                                │
│                                                                  │
│   episodic memory — Episode + bi-temporal + HDC signature        │
│   semantic memory — SemanticAtom (consolidated from episodes)    │
│   procedural memory — Procedure (typed episode shape)            │
│   working memory — session scoping + recency boost on recall()   │
│                                                                  │
└────────────────────────────┬─────────────────────────────────────┘
                             │
                             ▼  recall(query) → Recall
┌──────────────────────────────────────────────────────────────────┐
│ DOWNSTREAM: agent framework                                      │
│   LLM context window ← working memory in the prompt              │
│   tool use ← procedures invoked                                  │
│   reasoning ← over retrieved episodes + semantic atoms           │
│                                                                  │
│   ↑ agent runtime lives here, NOT in sochdb                      │
└──────────────────────────────────────────────────────────────────┘
```

sochdb is the middle layer. it has clean boundaries. it doesn't try to be the input pipeline; it doesn't try to be the agent framework. it does one thing: long-term, brain-inspired memory storage and retrieval.

## what to put in the pitch deck

a single slide:

> **sochdb is the only agent memory database with explicit mappings to all five biological memory tiers.**
>
> - episodic memory (events with time + place + people) — bi-temporal HDC signatures
> - semantic memory (decoupled facts) — consolidated semantic atoms
> - procedural memory (workflows + skills) — typed procedure shapes
> - working memory (active context) — session-scoped recall with recency boost
> - sensory memory (raw signal) — explicitly upstream, not our problem
>
> grounded in 50 years of cognitive neuroscience. shipped in one rust binary.

## next reads

- [ARCHITECTURE.md](../architecture/architecture.md) — how the three engineering layers implement these five biological tiers
- [LAYER_1_RECALL.md](../architecture/layer-1-recall.md) — the episodic-semantic split in retrieval
- [TECH_SPEC.md](../spec/tech-spec.md) — the `Episode`, `SemanticAtom`, and `Procedure` types
