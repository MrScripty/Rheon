# CPU recording verification

Environment: selected Linux cloud workspace `/workspace/Rheon`, official Rust/Cargo
1.92.0, default native allocator/filesystem and bundled SQLite. Commands source
`/workspace/rheon-setup/env.sh`; auxiliary target directories isolate compilation
from long existing numerical runs. No GPU/remote solver execution. No display,
WAYLAND_DISPLAY, Xvfb or xdotool was available; native-window smoke was unavailable.
The egui interaction evidence below is headless, not screenshot/native-window review.

## Deciding evidence

- Final `cargo test --locked --no-default-features --test recording_contract`:
  **11 passed, 0 failed, 0 ignored** (ten scenarios plus child-process harness).
- Same target with default features: **11 passed, 0 failed, 0 ignored**.
- `cargo test --locked --features desktop --bin rheon-desktop -- --test-threads=1`:
  **36 passed, 0 failed, 0 ignored**. Includes actual egui pointer activation of CSV
  and VTK controls, historical step-1 CSV generation and byte-equal existing-contract
  VTK after generation 3, reload after dropping the producer, absent timestep refusal,
  missing particle ID preservation, reentrant selection, point view and worker tests.
- Default, no-default and desktop `cargo clippy --locked [flags] --all-targets -- -D warnings`:
  passed. `cargo fmt --all --check` and `git diff --check`: passed.
- `cargo check --locked --manifest-path browser/solver/Cargo.toml --target wasm32-unknown-unknown`:
  passed. Rust 1.92's official WASM std target was installed explicitly for this
  compatibility check. Locked browser tree excludes recording/SQLite/SHA dependencies.
  Existing root/nested locked package identities are unchanged (13/21 new identities).

Record tests compare every binary64 particle/diagnostic bit with canonical solver
snapshots and use direct legacy VTK bytes as an interchange oracle. They also cover
binary32/binary64 signed zero/subnormals, full u64 run identity, empty populations,
zero-tuple fields, evaluation versus publication times, constructor absence, actual
accepted dt, canonical MaterialSnapshot capture, duplicate/rejected/cancelled physics
publication refusal, generation gaps, payload/quota/page caps, poisoned admission,
explicit unrecorded accepted tails, indexed selection while a writer advances,
corrupt blobs/metadata/dimensions/inventory/header/lifecycle/index, orphan rows,
unsupported versions, truncation and changed metadata after reader opening.
Negative fixtures assert the intended typed diagnostic (including SQLite FULL).

The crash case starts a real child process, commits a constructor, starts an actual
SQLite transaction, forces journal spill and exits with code 73 without destructors.
The parent cold-opens the actual hot-journal database and proves committed-prefix
recovery, absence of the partial step and active/unknown lifecycle. This establishes
sampled process-interruption recovery under the local filesystem; it does not prove
power-loss behavior on faulty storage or every I/O failure mode.

## Broader regression, with limits kept explicit

Full `cargo test --locked --no-default-features` passed **437 tests**, zero failures,
**3 existing ignored real-checkpoint tests**, 54 targets including empty/doc targets.
This run preceded the final units/header-integrity/UI refinements; final focused
core/default and desktop evidence above covers those changed paths. Physics source
and algorithms were unchanged throughout.

An initial full default run passed 366 tests, had 2 existing checkpoint ignores,
and stopped on one recording test's incorrect expected diagnostic. Changing units
from `m` to `cm` correctly tripped byte-inventory validation before the hash boundary.
The fixture now changes `m` to `s` (same length) and checks the intended digest error.
Production validation was not relaxed. The corrected default recording target and
all remaining 11 default targets passed: **87 passes, zero failures, 1 existing
checkpoint ignore**. These overlapping runs are not added or advertised as one
fresh final full-default run. Final focused tests supersede the earlier fixture.

One concurrent desktop rerun passed 35 tests but the unchanged carrier scrubber
lifecycle case hit its existing ten-second wall-clock deadline while numerical
suites were running. It had passed the initial 36-test run. An isolated serial
rerun passed all 36 without changing the deadline, carrier fixture or solver.
Final isolated verification includes the completed recording changes.

The three ignored tests require a SHA256-verified frozen AI checkpoint/export.
No model input was supplied, no tolerance relaxed and no real-model qualification
is claimed by this recording task. Full research-book source admission/qualification
must be renewed by its owner after review of these new crate/build inputs; existing
pinned receipts and epochs were not refreshed or weakened.

## Runnable cold-process example / independent consumer

`recorded_material record .generated/recording-validation/final.sqlite
 authored-final-feature-source-under-validation` ran a real MaterialSession and
created constructor + five accepted steps, 64 particles, database 81920 bytes.
A separate `recorded_material inspect ... 1 .../final-step-1.vtp` process verified
and exported ordinal/generation 1 after the producer ended. Publication = 0.00001 s,
evaluation = 0 s, actual dt = 0.00001 s. Source/build identity is explicitly declared
or unspecified; this precommit example is not a clean-source reproducibility receipt.

An independent Python sqlite3/hashlib/struct consumer decoded LE metadata and all
field shapes/units, independently recomputed run/every-step/lifecycle SHA256, checked
six publications and complete status, and confirmed ordinal-1 position bytes differ
from generation 5. First selected position:
`[0.1249999999718927, 0.12499999899122771, 0.1249999999718927]` metres.
Generated databases/VTK/consumer output remain ignored and outside commits.

Logs live under `/tmp/rheon-recording-*.log`; final sample under ignored
`.generated/recording-validation/`. The source contract and runnable commands are
[recording.md](../../../api/recording.md) and `examples/recorded_material.rs`.

## Remaining scope and integration ownership

Material is the only integrated desktop producer; carrier numerical history and
cold desktop opening are explicit follow-ups. Public API/example cold reload is
implemented. Keep-all/stop-at-limit is the sole retention policy; the logical database
cap excludes temporary rollback-journal/allocator/SQLite overhead. No whole-process
memory cap, full integrator state, restart checkpoint or performance promise.
PR67 and main are unchanged. Parent/sole integration owner controls dependency merge,
fresh source qualification, native-window acceptance and final branch disposition.
