# Preliminary current-main integration

Original head7f1733f and main15026a9 are preserved. No AGENTS.md exists in either
tracked source tree or the workspace instruction directories. Read current public
MrScripty/coding-standards Core/router and applicable implementation, planning,
verification/oracles, architecture/contracts/performance, Rust/API, commit and
documentation guidance. No standards exception or approval bypass is used.

Conflicts in liquid_step.rs and simulation.rs are additive: warm initialization
coexists with current force fields/configured-provider ownership. Provider routes
remain cold. Session's new initialized entry point now captures the actual pre-step
clock and clears obsolete configured receipt/history only on accepted publication.

Explicit Cargo feature narrowing is recorded in protocol.md and usage.md. No
particle implementation, physical threshold, failure gate, tolerance, iteration
budget or horizon changed. Original gains remain scoped to their historical base;
original +2.20% particle risk remains visible.

Default and feature-enabled `cargo check --locked --tests --examples` passed.
`focused2.log` is the source-frozen release focused check including configured and
interval-start regressions. `focused.log` is an earlier overlapping preliminary
check, not final qualification; the second invocation awaited Cargo's build lock.
No benchmarks ran concurrently. Earlier compiler logs are retained separately.
The real-model ignored test requires absent RHEON_AI_EXPORT and is not passed by
these local checks. Current native gate fails on ancestry while the merge remains
uncommitted; this observed outcome is retained, not treated as a numerical failure.

Final #142→#138→#143 composition, reviewed complete source admission, prospective
default controls, independent exact-source review, required CI and serial lead
merge authorization remain pending. No source is advertised as merge-ready yet.
