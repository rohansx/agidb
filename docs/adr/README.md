# Architecture Decision Records (ADRs)

This directory holds the durable, dated record of every non-trivial architectural decision in agidb.

## When to write one

Write an ADR when:

- a constitutional principle is amended (see the amendment process in [`../spec/constitution.md`](../spec/constitution.md))
- a new dependency, runtime, or storage primitive is chosen over alternatives
- a backwards-incompatible change to the public API is being considered
- a non-trivial tradeoff is being made and the *why* would not be obvious from the code

If a future contributor would reasonably ask "why was this done?", write an ADR.

## File naming

`NNNN-kebab-case-title.md` where `NNNN` is a zero-padded sequential number.

Numbers are never reused. Superseded ADRs are kept; the superseding ADR links back.

## Template

Each ADR follows this structure:

```markdown
# ADR-NNNN: <Title>

- **Status:** Proposed | Accepted | Superseded by ADR-MMMM | Deprecated
- **Date:** YYYY-MM-DD
- **Deciders:** <names>

## Context

What is the situation? What is the problem we're trying to solve? What constraints apply?

## Decision

What did we decide? Be explicit; include the chosen option and any rejected alternatives.

## Consequences

What follows from the decision? Both the upside and the cost.

## Alternatives considered

What other options were on the table, and why were they not chosen?

## References

Links to PRs, issues, prior art, or related ADRs.
```

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](./0001-license-apache-2.md) | License: Apache-2.0 | Accepted |
| [0002](./0002-async-runtime-tokio.md) | Async runtime: tokio | Accepted |
| [0003](./0003-agidb-v2-constitution.md) | Adopt the agidb v2 constitution | Accepted |
