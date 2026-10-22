# M4 staged App adoption — verification

**Status: Accepted as a bounded ownership/adoption component**, after complete separate review, serial R1/R2 correction, affected checks and targeted independent rereview. Full M4/A5/A6 remain pending.

Candidate after draft commit `b831bd06ba405f3f0c5c84957c553b4f5cdaa868`. This bounded component uses the existing App/Worker; normal contents Simulate and saved voxel rerun remain unavailable. It is not native user-workflow, performance, measured memory, physical GPU or whole-feature scientific acceptance.

## Contract and source

`voxel_material_launch.rs` calculates actual retained History/Inspector/frame, immutable Request/profile/force/mesh, Scene and result capacities before starting output. Canonical owners expose checked capacity queries, including legacy decoded selections and nested configured metadata. Shared aliases are conservatively charged twice. The unchanged 384MiB enumerated policy reserves new numerical/history/four-publication/staging/surface groups; allocator, SQLite, egui/GPU, UI-only text and independent inactive pages are explicitly excluded. Current contents/editor staging remains covered by the original 48MiB editor/staging reserve; this is not a process RSS bound.

The new immutable Request stays pending while old result bindings and selection remain paired. Old timeline, Inspector, Scene and viewport controls are held. Acknowledgment requires the new history's exact constructor frame and an already verified raw/typed selected constructor matching its frozen input digest under the Inspector lock. It precedes join even for a finished worker. Only that UI-thread boundary moves Request/history/Scene/result presentation. Known absent authored forces has an explicit binding identity, distinct from missing legacy metadata. Before acknowledgment a failed/cancelled worker preserves old owners/playback and reports its separate attempt; afterward it retains its new prefix. A late cancel flag cannot relabel a completed voxel archive.

## Executed checks

Pinned Rust1.92.0 and locked dependencies, same workspace toolchain as previous components. Logs under ignored `.generated/voxel-material/`; no test or platform policy was weakened.

- Six `voxel_material_launch::tests`, serial: passed,0ignored. Actual existing Worker and SQLite,4³ cells, at most3intervals per launch. They cover held old selected frame/Inspector/immutable recipe/playback, constructor adoption, fast completed first poll, late cancellation, pre-output cancellation and conflicting output file, new acknowledged cancellation prefix, actual spare-capacity refusal before output, and History allocations. The capacity refusal reserves a real240MiB Vec capacity; this checks admission accounting, not physical committed memory.
- Four `inspector::payload::tests`, serial: passed,0ignored. Real constructor/accepted SQLite selections refuse absent/raw-only, wrong digest and accepted generation acknowledgment. Exact spare-capacity tests cover legacy particle/diagnostic/raw text and native/configured profile members/text; a deliberately poisoned owner lock refuses rather than claiming zero.

Initial compile failures were fixture-only: an App functional-update expression conflicts with its Drop implementation, missing test import and Sender type. They were corrected before execution. Initial executed staged tests found an implementation mistake: voxel previews intentionally share `material_color: Some` for appearance, so this is not a discriminator for rejecting a constructor. The erroneous extra condition was removed; the explicit voxel tag/stamps and typed record remain required. All six then passed. No numerical assertion/tolerance changed.

Affected regression runs and independent review are in progress; final results and corrections will be appended before component acceptance/commit. Actual native save→run→both-phase inspect→cold-open/rerun remains pending.

## Complete independent round and serial corrections

The separate implementation review collected R1/R2 before corrections. R1: a supplied cloned Request could share the retained mutable receipt. The launch owner now establishes fresh per-attempt receipt custody unconditionally. Its real-Worker fixture deliberately submits an initially shared receipt, waits after new publication/before adoption, and checks distinct Arcs plus unchanged old path and capacity through the terminal result.

R2: a queued contents creation could bypass the Pending UI hold. Launch now refuses queued create/save/load before Worker/output. The actual update uses one narrow production `service_initial_contents_commands` boundary: while preparing/running it refuses stale queued commands into their own feedback, performs no authoring/file operation, and preserves preparation status. A newly issued idle command still works. No new command framework or durable-state format was introduced.

Eight corrected staged fixtures passed. The R2 fixture uses an actual old legacy carrier Worker/recording/selected decoded state, no existing contents (creation would otherwise succeed), verifies pre-output queued-command refusal, then injects stale commands while the new Worker is held. It composes the visible preparation Scene, actual command boundary, held timeline and disabled actual viewport with pointer press/held/release and Ctrl+Z. Pointer targets the painted Particles tab, which remains Appearance; old selection/playback/data remain retained. This is an egui/source interaction check, not a native window claim. Ordinary idle Draft creation afterward succeeds.

Baseline affected runs passed before this correction: core recording24; desktop voxel39 plus explicit fresh child1, Inspector8, History6, Scene10, viewport11, timeline5, setup9 with1pre-existing ignore, force UI7, configured Worker6. The root's routine boolean simplification and placement of the new free capacity helper before its test module resolve strict Clippy findings without suppressions. Final affected reruns, strict Clippy and separate targeted rereview are pending at this paragraph's write boundary.


## Final deciding checks and admission

All13 final source identities in [targeted independent rereview](m4-adoption-independent-review.md) match the commit candidate. No further required correction remains in this bounded component. Canonical plan stays Active; normal capture/rerun entry and actual native A5/A6 remain pending.

| Command after `cargo` | Final result |
| --- | --- |
| `test --locked --features desktop --bin rheon-desktop voxel_material -- --test-threads=1` | 41passed; explicit fresh child1passed;2child-only ignored entrypoints retained |
| Same desktop command, filter `inspector::` | 8passed, including4new owner fixtures |
| Filter `scene_ui::` | 10passed |
| Filter `viewport::` | 11passed |
| Filter `timeline_ui_tests` | 5passed |
| Filter `setup_ui_tests` | 9passed,1pre-existing ignored harness |
| Filter `initial_contents_ui::tests` | 10passed |
| `clippy --locked --features desktop --all-targets -- -D warnings` | Passed |
| `fmt --all -- --check`; `git diff --check` | Passed |

Unchanged recording, History, force UI and configured Worker owners retain the earlier affected passing results above. No benchmark, new numerical campaign, physical GPU or native end-to-end acceptance is claimed. Main/other worktrees/publication remain untouched. No new action was denied, pending or uncertain in this component; prior denied external actions and uncertain Library writes remain stopped.
