# M4 existing-worker component verification

**Status: Accepted as a bounded existing-worker component after complete separate source review, serial R1/R2 corrections, final affected execution and targeted independent rereview.** Canonical authority is `../plan.md`; this component does not accept M4 or A5/A6 as a composed feature. Source remains isolated from main, and normal contents launch/saved voxel rerun stay unavailable.

The existing Worker now dispatches one owned immutable voxel recipe to the real variable-density two-material numerical owner. It runs in the existing serial Rayon/CPU scheduler, records exact fields through StepRecorder and publishes explicitly tagged immutable liquid appearance after successful append. The constructor uses original captured geometry and coefficient bits. Actual accepted dt and interval count remain distinct from requested maximum dt and nominal horizon. Immutable original/started/final child revisions use protected no-replace publication and the existing typed closed-archive audit.

Four new numerical/API responsibilities are kept in their canonical owners: native request/configuration/resource refusal in the desktop dispatcher; actual preview allocation/shape checks in History; actual raw/decoded capacity checks in typed recording; and owned immutable recipe/manifest capacity accounting in their existing persistence owners. No evolution formula, material palette, old schema, dependency, worker manager or numerical fallback was added. Retained Inspector routing, method captions and alternate viewport availability recognize the explicit voxel tag. Result thumbnails are a bounded max projection of liquid appearance only.

## Executed component checks

Pinned Rust1.92.0, locked dependencies, saved Linux environment. Commands used `PATH=/workspace/.voxel-cargo/bin:$PATH`, `CARGO_HOME=/workspace/.voxel-cargo`, `RUSTUP_HOME=/workspace/.voxel-rustup`. Commands below are local automated supporting/component evidence; they are not native editing, physical-GPU, performance, RSS or scientific validation.

| Command after `cargo` | Observed result |
| --- | --- |
| `test --locked --features desktop --bin rheon-desktop voxel_material_integration_tests -- --nocapture` | 11 parent fixtures passed;1 child-only ignored entrypoint explicitly executed and passed in its real subprocess, after R1/R2 corrections |
| `clippy --locked --features desktop --all-targets -- -D warnings` | Passed after final source-review corrections |
| `test --locked --test voxel_material_recording_contract` | 5 passed |
| `test --locked --features desktop --bin rheon-desktop inspector::tests` | 2 existing Inspector fixtures passed |
| `test --locked --features desktop --bin rheon-desktop inspector::voxel_material_tests` | 2 typed voxel Inspector fixtures passed |
| Same desktop test command, filter `history::tests` | 6 passed |
| Filter `viewport::tests` | 9 passed |
| Filter `timeline_ui_tests` | 5 passed |
| Filter `setup_ui_tests` | 9 passed,1 pre-existing deliberate ignored fixture not executed |
| Filter `force_runtime_ui_tests` | 7 passed |
| Filter `configured_worker_tests` | 6 passed,10.67s; no hanging process or blind rerun |

New integration fixtures use real existing Worker start/join, a4³ Water/Air column and actual SQLite/history. They compare original f64 fraction/MAC fields with the captured numerical owner, assert positive gas mass, captured digest and signed-zero origins, distinguish constructor/accepted reports, exercise serial/unsupported/aggregate/immutable-time refusal before output, test actual oversized/invalid preview and raw selection capacity, and reproduce original fields/time/dt in a fresh test process with a fresh source-excluding ID and new archive. A6³ h=0.1m case proves the scalar presentation round-trip differs while the actual archive preserves h bits. An explicit saved-spacing versus exact-preview policy fixture verifies distinct refusal boundaries.

A real ratio-shortened1s requested interval completes and records the actual smaller positive dt with attempts>1; original requested schedule remains unchanged. A cancellation at an observable first-interval progress boundary preserves constructor plus one accepted record/frame and pins a cancelled attempt. A bounded64KiB database reaches DiskFull on generation6 append: six acknowledged records/frames remain, accepted generation6 is an unrecorded tail, terminal outcome is failed, and a confirmed final pin remains inspectable. No append or uncertain publication is retried. This is a deliberate per-archive page budget, not an actual host-disk exhaustion campaign.

An initial captions fixture incorrectly read AccessKit labels for static text; its empty result failed. Corrected it to inspect actual painted text with an explicit screen rect. A later strict-lint finding was a test-only manual range expression; corrected without suppression. These failed drafts are not acceptance evidence; subsequent passing logs are the deciding checks. The earlier8-parent and9-parent runs were superseded by the final11-parent run after adding decisive original-spacing/executor, four-group boundary and appearance oracles. Default recording and desktop regressions were not re-executed without a changed affected concern.

Logs are local ignored `.generated/voxel-material/m4-worker-*.log`; retained actual fixture artifacts are under `.generated/voxel-material/m4-worker/<pid>-<ordinal>/`. They are not tracked generated outputs. Reviewed/final source and log identities are recorded below.

## Remaining gates

Production App retained-context calculation, staged constructor acknowledgment and atomic request/history/Scene adoption remain unimplemented. These headless fixtures use zero retained context because they have no old App results; they do not authorize a real App caller to submit zero. Loading/replacing a Draft needs identity collision checks and fresh session-only undo history. Actual native paint/save/run/inspect/cold-rerun remains pending. No old-result preservation on native startup, native simulation, whole-feature conservation/accuracy, acceptance/exposure, main integration or external publication is claimed by this component.

## Independent findings and corrected deciding evidence

The complete separate [source review](m4-worker-independent-review.md) found R1 four simultaneous publication groups during closed-archive auditing and R2 missing value-level preview evidence. Corrected only the reserved count at the same384MiB ceiling; normal stepping still drops producer bytes before preview and uses at most three groups. Pinning may overlap the selected raw+decoded pair with the audit raw+decoded pair. A controlled retained-context threshold that fit three groups now refuses Worker start before output; the exact corrected four-group boundary validates. No production retained-context measurement is inferred from this arithmetic fixture.

The preview fixture states independent fixed expected planes for the known liquid column at original4³, finer5³ and nondivisible/coarser3³. It also advances a zero-flow mixed state through a real accepted interval and checks F=.25/.5 at original indices21/42 projects to bytes64/128, occupied-gas complement is transparent and all f64 fractions/geometry remain unchanged by projection. It does not repeat the producer's nested resampling calculation or claim native visual acceptance. A failed attempt with confirmed failed terminal and pin is Ready for inspection, not completed execution; its comment now matches that actual contract.

## Final source and execution identities

SHA-256 identifies the checked local bytes, not authenticated build provenance. Ordinary compiled executor source/build declarations may remain explicitly unspecified. The fresh-process fixture executes the same current test binary; it does not establish cross-compiler/host equality.

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/voxel_material_execution.rs` | `c2d04c805afb919ce7383da01818d64e6f8e8cffc11812f2fe7e4c87fd15f35b` |
| `src/bin/desktop/voxel_material_integration_tests.rs` | `af408b8b2611f0f56f0ff0455218093ab3a276deac59294e6dcfc14905915fc9` |
| `src/bin/rheon-desktop.rs` | `c1e76932e517e439424013c1778578fb053d5b76e46cf2c92a0e68dfafd28611` |
| `src/bin/desktop/history.rs` | `543175ebae4ac8b951d857077b21bf1aef9ccaa0bbd675a54e4eb6a2b88029c9` |
| `src/bin/desktop/viewport.rs` | `5ffcfe2ec6a6b1aff087d5c17c37eecb82e4f724e351471225d62a09a4fbd635` |
| `src/bin/desktop/run_sets/voxel_material.rs` | `6a5b1f0cffea4992476c9ae6b9c8a88061fe89ddec8f4fde833a356479ccb8e0` |
| `src/bin/desktop/run_sets/voxel_material/run_set.rs` | `6165917638619e9f1f250bae017e0381780b2838c0522384ecc3c859ac128958` |
| `src/recording/voxel_material.rs` | `35148ff3972a7e22e4dea75d6c665d6a8437103e6dec1d818e56a479c0588e05` |
| `.generated/voxel-material/m4-worker-tests.log` | `a50b3d9e94f6d32470da481d80f4fc4263adf630928847c9cc989abbebf592b9` |
| `.generated/voxel-material/m4-worker-clippy.log` | `a8c3e9fa80abad9f4841224aaa59effb76b960c33db1b960350aeea3d31ce84a` |
