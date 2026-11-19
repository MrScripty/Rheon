# M4 native draft file interaction verification

**Status: Accepted as a bounded file/UI ownership component**, after complete separate source review, serial corrections, affected execution and targeted independent rereview. Canonical plan remains Active; launch/adoption and actual native A5/A6 remain pending. Worker dependency is commit40a8b607adc05b639a20421e4b36c21adfce0616. No main movement or external publication.

The existing contents panel now exposes a collapsed Draft file section with labelled local path and Save draft/Load draft. Load is available without a current document. Text explicitly states that Save needs a new file and Load replaces contents/domain and resets contents undo. Actual App service consumes each queued action once, revalidates idle Draft/Worker/edit state, and reuses the reviewed strict versioned draft/no-replace owner. Successful Load validates geometry and all live/retained force/model ID conflicts before any adoption; it preserves original IDs/revision/cells/coefficients/spacing/origin, installs fresh contents history and advances the Scene allocator without reviving exhaustion. Retained Request/history/results remain separate. Unsupported native editing geometry refuses without resampling. Save preserves redo and never overwrites.

An actual PublicationUncertain result stores its exact target and pauses further session saves; no automatic or alternate-path retry is performed. The uncertainty fixture tests the reaction to an already uncertain owner state, **not an executed filesystem fault or completed reconciliation**. Load remains file-read-only and does not clear that latch. This is a session guard around the existing durable owner, not a new crash journal.

## Executed checks

Pinned Rust1.92.0 and locked dependencies in the saved Linux environment; same isolated toolchain as preceding M4 components. Logs are ignored local `.generated/voxel-material/m4-draft-ui-*.log`, actual files under `.generated/voxel-material/m4-draft-ui/<pid>-<ordinal>/`. All following final commands passed:

| Command after `cargo` | Result |
| --- | --- |
| `test --locked --features desktop --bin rheon-desktop voxel_material_authoring -- --nocapture` | 9 passed |
| Same desktop command, filter `scene_ui::tests` | 9 passed, including1 new force-feedback preservation fixture |
| Filter `initial_contents_ui::tests` | 10 passed |
| Filter `viewport::tests` | 9 passed |
| Filter `setup_ui_tests` | 9 passed,1 pre-existing ignored workload not executed |
| Filter `force_runtime_ui_tests` | 7 passed |
| `clippy --locked --features desktop --all-targets -- -D warnings` | Passed |
| `fmt --all -- --check` and `git diff --check` | Passed at commit boundary |

File fixtures exercise actual App actions and local files: exact non-dyadic/signed-zero data and fresh history; redo-preserving Save/no-replace bytes; invalid/offset/busy refusal retaining old state; current force/model and other retained-track ID conflicts; high/MAX-ID allocation; retained history Arc, preview payload and playback untouched. One actual4³ existing Worker is held at its first accepted interval gate; both file actions refuse before I/O, then cancellation is released/joined. It is a tiny ownership fixture, not a new benchmark or numerical-validation campaign.

The input fixture clicks the actual painted path field, sends Ctrl+A/Text and clicks real Save/Load buttons; queued actions reach App service. It does not programmatically focus the field. The separate composed fixture uses actual Scene UI, App service and Viewport. It queues Load through the real button, deliberately delays service in the fixture to construct a pending action plus coalesced raw input, then services inside the real egui frame before the viewport. Coalesced press/release and held-primary tails leave restored IDs/revision/history exact, preserve camera/forces, suppress the held tail through release, and allow a fresh later brush. This is component/input evidence, **not a native-window or exact full-App scheduling acceptance claim**.

## Independent corrections and evidence limits

Complete separate review found R1 clearing unrelated force-owned Scene feedback and R2 replaying the load frame into newly installed contents. Removed only that unrelated feedback clear. Actual invalid-force validation/data now survive a real App Load. Successful service stamps its current egui frame; the contents canvas drains that frame before brush/undo handling and preserves the existing primary-release latch. Direct restore still installs fresh UI state. The [targeted rereview](m4-draft-ui-independent-review.md) resolves both.

Failed fixture drafts were corrected without changing product assertions: App's Drop prevents struct-update construction; a ResultRow Debug bound required err().unwrap(); sparse AccessKit release updates omitted an unchanged label, so actual painted text locates the path field; the second composed iteration initially reused stale button geometry after feedback layout, so it now queries current painted bounds; empty force names are permitted, so the feedback fixture now uses the real nonfinite-vector validation error. These are not native/product acceptance failures, and superseded logs are not counted. No validator, numerical tolerance or legacy test assertion was weakened.

Production old-result payload calculation, staged constructor acknowledgment and atomic retained Request/history/Scene adoption are the next slice. Actual native paint→save→run→both-phase inspection→cold rerun remains mandatory before exposure. Contents launch and saved voxel rerun remain unavailable; no A5/A6, physical GPU, performance/RSS or scientific validation claim follows from these checks.

## Final checked identities

SHA-256 binds reviewed source and final logs, not authenticated build provenance.

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/voxel_material_authoring.rs` | `eab322f20b7f17033dea43acd422a799fce6cfb99cd53a90c434a05e7aac5daf` |
| `src/bin/desktop/voxel_material_authoring_tests.rs` | `9a4adc0dcd4a0ec2280cf80eb2064b8f93c4c9fc0d9df54dafa77a179cf1d858` |
| `src/bin/desktop/initial_contents_ui.rs` | `0829ee860fae6116b0d49c7e013529f6db52e2ff274703faf666a02085668ac4` |
| `src/bin/desktop/scene_ui.rs` | `863b410df4d99230dc65f7bf6a89227a1bc32e7137ad5830ed175aca55c8f089` |
| `src/bin/rheon-desktop.rs` | `3bab0198375310e9fd328ed2af138efa4b2392b1c246fca6d066d41307799fe5` |
| `.generated/voxel-material/m4-draft-ui-tests.log` | `cc9c076aab1c371827bd7c218c6026a545e364dc75a8cf606b233b26e769f736` |
| `.generated/voxel-material/m4-draft-ui-scene.log` | `cc515d68998b641a1d4bf2b9e60df5a7a12778388216abaa64216f8101f3c0e2` |
| `.generated/voxel-material/m4-draft-ui-clippy.log` | `4dc2309f5d66e1889c6f64fab3c2de310b9e910e6ae614f89eb0076e9b8344ad` |
