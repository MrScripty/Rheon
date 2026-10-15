# M2–M3 clean persistence and comparison stack

2026-10-15. Implementation is continuing; this report does not close A1–A8.
Integration lead owns shared source, qualification inventories and all merges.

## Material and scope

[Draft PR89](https://github.com/MrScripty/Rheon/pull/89), branch
`implementation/configurable-persistence-m23-20261015`, starts at M1
`a527c42ee8ad1db05ee3088820de5b213f994466`. The lead's subsequent `50b40cf`
changes only its execution ledger; its later `f75a714` changes worker termination
qualification and is not yet in this stack's tested base. The stack selects the reviewed owned files
from [PR85](https://github.com/MrScripty/Rheon/pull/85) and
[PR86](https://github.com/MrScripty/Rheon/pull/86); their unrelated inherited
training/voxel-viewer ancestry is excluded. Original draft branches are preserved.
The initial combined source/codec commit is
`6f121082209df057729211178c997a81c3c8d46f`.

The accepted-native slice freezes actual typed constructor controls, future dt,
source schedule, horizon and archive limits. Existing Simulation and
StepRecorder/RecordingReader remain the numerical and recording owners. A saved
set embeds immutable specs, exact differences and terminal archive pins. It has
no numerical arrays, mutable latest alias, restart checkpoint, worker, model
loader or alternate numerical store. Its actual configuration envelope and
publication uncertainty are specified in [saved-run-sets](../../../api/saved-run-sets.md).

Native comparison selects explicit source ordinals but aligns actual stages and
physical times. Geometry and schema must match; no interpolation, particle-index
matching or display-pixel reduction is performed. Metrics use native fields,
including staggered-face divergence. Exact zero reference norm has unavailable
relative error; pressure mean removal remains an explicit diagnostic with the
removed offset retained. [Comparison semantics](../../../api/recording-comparison.md)
version this analysis separately from the archive and simulation.

Saved recipes embed the complete immutable run-set revision, explicit source
members and selected publication identities. A saved result decodes as Unverified.
Only reopening the exact pinned source archives, matching each selection and
recomputing the complete result produces Verified. Missing, corrupt or changed
sources cannot substitute a live/latest publication. The codec reuses M2's bounded
atomic no-replace publication; it introduces no new store. Limits are 128 explicit
selections per source, 64 MiB aggregate selected field payload and 1 MiB document
size. The payload budget is not an RSS, total I/O or rendering-time guarantee.
[Saved-comparison contract](../../../api/comparison-archive.md) defines the boundary.

## Fresh observed verification

On the actual combined checkout at `6f12108`, Rust 1.92.0 Linux CPU:

- `cargo test --locked --features desktop --test saved_run_set_contract --test comparison_archive_contract --test recording_comparison_contract` passed 17 + 12 + 15 tests, respectively.
- Focused desktop Clippy with `-D warnings` passed for those three targets.
- `cargo fmt --all --check` passed.
- `cargo test --locked --no-default-features --test recording_comparison_contract` passed 15 tests.

The native tests execute both real Jacobi and SGS owners and preserve f32/f64
field bits. An actual producer OS process exits before a separate reader starts;
the reader opens no model and checks selected bytes against SQLite BLOBs. Actual
capacity failure retains accepted generation 2 while generation 1 is the last
confirmed recorded interval. Cold reopen preserves the failed outcome and
unrecorded tail. Actual SQLite terminal-status contention leaves the archive
incomplete, without a guessed retry or overwrite. Selection stability under another
track's progress is an in-process interleaving test, not simultaneous GUI evidence.

Saved-result tests also use real native producers and fresh reader processes.
Negative cases cover missing/moved/corrupt/changed sources, altered selection
identity, strict bounded/duplicate/versioned decoding, result content mutation and
rehashed plausible metric forgery. Independent read-only GPT-6.1 Sol High review
reran the focused tests and challenged rehashed unavailable outcomes, swapped
pair identities, forged relative-error nulls and generation substitution; the
actual source recomputation or selection check rejected each. No material finding
remained. Reviewed codec source SHA256:
`95f735ca5e82044115f962b1fe180bcb36b4a4584ddfb1c76b4b4acdb4e5f3a9`.

The independently authored Python standard-library oracle reads SQLite/native
bits directly and verifies all six field roles, selection fingerprints and
nonempty real-native aligned coverage. It uses coordinate lookups and `math.fsum`
with an explicit propagated floating-point divergence bound. This checks the
stated analysis formulas for the sampled native fixtures, not physical accuracy
or accepted-provider qualification.

PR86's exact original `c17efaa` CI passed the default, core-only and desktop feature
matrices, executable/native CPU/rational/conditional/coupling and VTK reader
checks. PR85 has additional source subjects; old results are not transferred to
new commits. PR89 checks are separate and pending at this report's creation.
The Pages source-inventory gate rejects unadmitted changed/added source. The lead
must review the complete integrated inventory and run fresh qualification on the
committed integration. No old receipt, inventory refresh without review, skipped
check or changed numerical gate supplies that evidence.

## Environment and outstanding composition

Executor access after the disconnect notification was confirmed by an actual
fetch and completed combined test run. Workspace-only official Debian signed
packages provided Xvfb/xdotool; the existing snapshot endpoints failed through
the proxy, while signed Debian trixie live indexes/packages succeeded. An
authenticated local X11 display, 1600×1100, reports Mesa 25.0.7 llvmpipe software
OpenGL 4.5. This is only development-display availability. No Rheon native-window
workflow or hardware GPU test has run here; no startup screenshot is acceptance.

The cold saved-comparison page and headless interaction fixtures are in progress
in new owned files. Shared existing native `run_owned`/Inspector capture and
paired-view geometry/scalar/unit projection still await the lead's explicit
serial allocation or supplied hooks. Root navigation/worker lifecycle remains
lead-owned. These are concrete composition dependencies, not permission to add a
second loop, supervisor or renderer.

M1's current raw AI diagnostic Scene/Frame evidence remains distinct from
accepted publications. Typed diagnostic storage, equal-request teacher capture,
stop-on-reject, warmup/fallback/count/cost presentation and physical qualification
of any production provider binding still require their actual owned contracts.
Native accepted-v1 records cannot invent those absent measurements. Diagnostic M1
alone does not satisfy accepted-provider A3. The measured trained bundle, real
model inference, full native interaction, crash/power-loss durability and broader
platform acceptance remain unqualified by this stack.
