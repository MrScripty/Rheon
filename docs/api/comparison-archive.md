# Saved native comparison evidence, schema 1

`desktop/comparison_archive.rs` adds immutable recipe and derived-result artifacts
on top of the native analysis contracts in
[recording-comparison.md](recording-comparison.md). It uses the existing M2 run-set
owner and its publication/decoding helpers. It stores intent, identities, coverage
and reductions; numerical fields remain in the existing SQLite recording owner.
No worker, model runtime, numerical store or integrator checkpoint is added.

This continuation is admitted by the parent/user for the new comparison archive
module, focused tests and this API document, plus desktop-only serialization in
`recording/comparison.rs`. The original PR86 commit remains intact. Shared module
exports, Cargo, desktop composition, the plan and all merges remain lead-owned.
The application module expects `super::native_comparison` to name the numerical
analysis module and `super::run_sets` to name the M2 source owner. The desktop
now registers that alias to the actual public `rheon::recording::comparison` API
and registers the comparison archive owner; focused tests use that same numerical
API and the tracked M2 source module.

## Identity and source custody

`SavedRecipe::capture` takes a complete immutable `RunSetRevision`, two explicit
member IDs, ordered source ordinals and the full native `Recipe`. Each chosen
member must resolve through M2 `open_member` as a ready, terminal, pinned native
archive. Its `RunAttempt.finalized` retains the exact whole-file SHA256, record
count, terminal outcome and accepted generation. The recipe embeds the complete
run-set revision and its M2 canonical-content SHA256, plus every selected
`SelectionIdentity` (run/source/ordinal/generation and exact publication SHA256).
The canonical run-set digest identifies the revision's content; it is not a hash
of an external manifest file. Embedding the revision makes cold recipe inspection
independent of an external manifest or a mutable latest alias.

The source reader validates native profile/spec provenance, exact geometry,
recording limits, terminal state, publication schedule and field schemas through
M2. No source provenance or controls Debug string becomes runnable input. Source
artifact hashes and selected-publication hashes serve different purposes: a
changed terminal receipt invalidates the whole artifact pin even if the chosen
field bytes remain unchanged. Archive ownership, protected paths and immutable
terminal files remain caller obligations throughout capture and verification.

`SavedRecipe::open` checks the artifact and recipe structure/digests without
opening archives. It can cold-open on a machine with no model runtime. `compute`
resolves the exact pinned members, reselects the explicit ordinals, checks every
publication identity and calls the existing native `align`. No latest file,
live state, nearest ordinal, field substitute or implicit re-execution is used.
A missing/corrupt/changed source returns its explicit source failure.

## Derived result verification

A result embeds its exact saved recipe and complete native `Coverage`, including
analysis version, full recipe, both identities for every pair, unmatched identities
on both sides, available finite metrics or typed unavailable reasons, units,
gauge report and common finite color bounds. Pair identities and unmatched
identities must partition the selected sources exactly once. The codec checks
that versions, recipes, units, gauge policy, finite/nonnegative metrics and bounds
are structurally consistent. Nullable relative L2 and gauge keys must be present;
JSON null retains their explicit absence. Unknown keys and closed enum variants
under schema 1 are invalid codec input.

`UnverifiedComparison::open` deliberately returns an unverified type. Its
`recorded_coverage` accessor exposes stored derived evidence, not verified facts.
A valid envelope hash proves internal content consistency, not accurate metrics,
authenticated provenance, accepted physics or source availability.

`verify(root)` reopens exact archive pins and recomputes the entire comparison,
then compares canonical serialized coverage digests. Only successful recomputation
returns `VerifiedComparison`; plausible finite metrics with a self-consistently
rehashable envelope still fail if they differ from their sources. Verified means
verified against the exact recorded source bytes in that operation. It does not
qualify a solver, model, physics plan, GUI workflow or later mutable file state.
The verified type exposes immutable recipe/coverage accessors, content digest and
`save_new`. Reopening its saved artifact starts unverified again.

Native analysis semantics remain owned by analysis version 1: physical-time/stage
alignment, native geometry/association/units, raw f32/f64 reduction, explicit
pressure gauge diagnostics and no cross-run particle pairing or remapping. The
M2 source profile requires canonical native carrier publications; an offset
analysis policy does not bypass that source validation or promote diagnostics.
Warmup, attempted/fallback counts, timing scopes and teacher/independent model
attempt evidence remain unavailable when not present as typed source facts.

## Codec and publication

Both artifact kinds use a strict JSON envelope:

| Field | Meaning |
| --- | --- |
| `schema_version` | Comparison artifact syntax version, currently 1 |
| `kind` | `recipe` or `result` |
| `sha256` | SHA256 of the payload's canonical serde JSON serialization |
| `payload` | Saved recipe, or saved recipe plus full coverage |

Schema, analysis and embedded run-set versions are independent. Unknown outer
schema versions return unsupported before full decoding. Within schema 1, unknown
analysis versions in the supported integer domain return unsupported; unknown
required variants/keys and malformed, duplicate, out-of-domain or truncated
values are invalid codec input. Shape changes require a schema change. The
original bytes are decoded through strict types; no generic JSON map erases
duplicate fields before validation.

Publication reuses M2 `publish_new`: stage a fresh sibling file, sync it, hard-link
without replacing the destination, sync the directory, remove staging and sync
again. It requires caller-owned protected local paths and filesystem support for
hard links and directory sync. Interrupted staging is not authoritative and is
not silently replaced/deleted as recovery. Once linked, later failures retain
`PublicationUncertain`; callers reopen to reconcile. Existing recipe/result files
are never overwritten. Each recomputation is explicitly published to a new path.

Artifacts are bounded to 1 MiB. Each source selects 1–128 strictly increasing
ordinals. Capture/compute incrementally enforce a 64 MiB aggregate selected raw
field-byte budget before retaining another selected publication. This is a
selected-payload/work budget, not a process RSS, allocation or I/O guarantee:
recording verification may read the entire bounded archive, a read can temporarily
hold one additional bounded record, metadata takes memory, and numerical analysis
allocates promoted f64 arrays. M2's recorded/caller recording limits still apply.

## Evidence and remaining integration

The contract tests use actual Jacobi and SGS simulations with nonzero smoke
forcing, `bound_native_run`, `StepRecorder` and pinned `RecordingReader` sources.
One producer process writes archives/recipe/result and exits; a separate process
cold-opens and recomputes the result. Tests cover every native field unit, exact
zero-reference absence, pressure gauge labeling, unavailable geometry and
unmatched selections, no-overwrite, moved/missing/corrupt/changed source files,
changed exact selection identities, plausible rehashed metric tampering,
structural/gauge/unit/coverage damage, unknown versions/variants/keys, duplicate
keys, truncation, digest damage, required nullable facts and artifact/selection
bounds. The independent numerical oracle remains in the native comparison tests;
this codec does not replace that evidence with round-trip agreement.

Observed on Linux CPU, Rust 1.92.0 (2026-10-15): 12 archive contract tests and 15
native comparison tests pass with the desktop feature; the 15 core comparison
tests also pass without default features. Focused desktop Clippy with warnings
denied and Rust formatting pass. No new dependencies were added; serde and
serde_json remain gated by the existing desktop feature. Core-only comparison
continues to compile without those optional codecs.

```sh
cargo test --locked --features desktop --test comparison_archive_contract --test recording_comparison_contract
cargo test --locked --no-default-features --test recording_comparison_contract
cargo clippy --locked --features desktop --test comparison_archive_contract --test recording_comparison_contract -- -D warnings
```

The serial integration includes tracked M2 helpers and actual public analysis
registration. Fresh isolated-target validation passed 12 saved-comparison contracts
and 16 numerical-comparison contracts, with scoped Clippy/formatting. Historical
isolated helper hashes above are not substituted for the integrated source.
Native desktop comparison interaction and the measured trained runtime remain
separate acceptance gates; a saved result is still unverified until recomputed.
