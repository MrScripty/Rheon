# Independent actual-source review: M4 native qualification entry

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`. Base commit: `85ec0eddfa7bec7d783c3003aa7d779fd3b8843e`. This is the complete source findings round for the current native-entry slice against `m4-native-admission.md`, the prospective design review and previously accepted capture/staged-launch boundaries. Only this report was written. The reviewer ran no build, test, native process, simulation, benchmark, external publication or Library operation.

**Disposition: no required source correction identified in the reviewed snapshot.** The temporary debug transport and permanent selected-Contents controls are suitable for freezing the bounded actual-App protocol and proceeding to its separately reviewed native observation. This does not accept A5/A6, normal availability, a release artifact or the whole feature. Normal `AVAILABLE` remains false. Actual native execution and its independent evidence review are still pending.

## Complete source findings

No blocking or nonblocking source defect requiring correction was found in this round. The following boundaries were checked in the actual dirty production diff and new tests, rather than inferred from compile success.

### CLI transport and lifetime

`voxel_material_setup.rs::configure_arguments` parses without importing a mesh, creating contents, constructing a Worker or writing output. It refuses missing/empty/option-shaped qualification arguments, duplicate qualification, unknown arguments, qualification combined with collision/autostart options in either order, an existing collision path, and output strings above 4096 bytes. A non-debug build returns an error before setting qualification availability. Existing collision assignment semantics, repeated legacy options and deferred import/autostart are preserved. `main` invokes the parser before its existing collision import and `start(false)` operations; errors exit that construction path. The eframe window/context and default App are created before parsing, so this is specifically absence of import/autostart/output side effects, not a claim that argument parsing precedes all GUI initialization.

The private App boolean defaults false and is set by the explicit successful debug argument. It is not stored in Request, frozen input, draft, run-set recipe, archive metadata or numerical state. Its consumers are the common UI-availability projection and visible qualification caption. There is no additional executor, seeded document or startup run. The selected defaults match the admission: 8³, 1 m, eight intervals, maximum dt 0.001 s, Earth gravity, 16 MiB history, 32 MiB database/4 MiB publication and document-grid preview; the cylinder is explicitly disabled in this mode. Numerical policy and method remain the previously accepted capture/worker owners.

The parser selects a path string; it does not certify that a directory is protected, create it, or weaken the later output/publisher checks. The procedure must supply the caller-owned protected local destination stated in the admission. The fixed defaults also do not constrain all subsequent UI edits to the qualification schedule. Freeze and audit the actual chosen input before each of the admitted launches. Remove the temporary argument/default/boolean/caption transport when normal admission makes it redundant, and review/smoke the final normal artifact separately.

### Readiness, launch and stale-frame revalidation

`contents_capture_readiness` is the extracted read-only structural gate also called by `capture_contents_request`. It checks closing, worker/Pending, Draft context, canonical Contents selection, Scene/contents gesture ownership, configured-workflow exclusion, emitter/mesh/authored-force exclusion, actual document geometry, active editor and occupied-document validation. `contents_run_readiness` adds the common availability policy. Neither query cancels a gesture, changes selection, silently clears an unsupported feature, publishes files or constructs numerical state.

The global button returns an explicit `SetupAction::RunContents`; dispatch calls the existing `start_contents`. The panel receives an owned per-paint Result projection through Scene, and additionally requires `editable` and `!busy`. Its queued action reaches the same production command service/start owner. The actual capture rechecks structural state after current-frame form or document changes. There is no legacy carrier fallback on a failed contents action. An enabled structural readiness result is not a promise that full frozen numerical/configuration/output/resource admission will succeed: those checks still execute at capture/launch and return the existing refusal. This limited query is consistent with the admitted nonallocating structural projection, rather than a second full recipe constructor.

Scene does not retain the Result as a new authority. A selection/document change later in a frame can temporarily change the next painted enablement, but actual execution still revalidates. Saved view likewise receives only the App availability boolean; its rerun action still routes through the original immutable voxel recipe, executor/native-host/resource validation and staged launch. Current Draft settings are not substituted into that saved recipe. Ordinary startup supplies false availability to both Contents actions and saved voxel rerun.

Pending and running paths remain the existing owners: the global control presents Cancel; preparing Scene/UI holds old result state; production command service refuses stale queued editing/file/run actions; full launch retains its existing atomic adoption and finalization. This slice changes none of those source owners or resource arithmetic.

### Truthful selected-Contents settings

The new owner-local setup branch reads actual document counts/spacing and states the fixed incompressible, closed free-slip, initially resting, no-surface-tension model. It reuses the actual App maximum dt, interval count, gravity vector, output directory, publication/database/history limits and preview policy consumed by capture/launch. Particle, thermal, initial-velocity and unrelated legacy controls are not painted in this branch. “Accepted intervals can be shorter” matches the accepted ratio-shortening method. The spacing caption is a formatted decimal view of original geometry, not a rewrite or bit-exact serialization.

“Disable emitter for contents” is an explicit user action; readiness itself does not mutate the emitter. “Use document preview” deliberately selects the existing Adaptive policy. Configuration workflow has the existing Setup selector as a remedy. Mesh/force refusal remains tied to the actual scene owners, rather than pretending unsupported geometry/forces affect this solver. The ordinary legacy settings branch and its fields remain intact when Contents is not selected. Existing sidebar/compact layout and Settings/Inspector navigation are reused; compact-size reachability, scroll behavior, focus and actual visual usability remain native observations, not conclusions from these source reads.

### Scope, fixture repairs and evidence adequacy

Reviewed production changes are confined to the five admitted owners: `rheon-desktop.rs`, `voxel_material_setup.rs`, `initial_contents_ui.rs`, `scene_ui.rs` and `run_set_view.rs`. Direct Scene/controls fixture callers, including the cfg(test) presentation harness, now pass explicit unavailable projections. Those narrow signature repairs do not enable that harness or make it deciding native evidence. No physics formula, numerical policy, persistence/schema, dependency, retained-capacity accounting, source-format or shared-main change appears in this diff.

The six new component fixtures cover ordinary unavailable global/panel clicks, successful debug defaults without document/output, conflict/invalid argument refusals, unchanged legacy argument assignments, shared structural refusals, an actual enabled global action reaching the existing voxel Worker, panel editability and a queued panel action refusing subsequently created Vacuum. Tests use actual egui pointer input and the real component owners; they do not establish an OS window workflow. The enabled panel positive case proves queuing, then deliberately tests capture refusal; it is not a separate successful native panel launch. Release rejection is established here by source inspection of `cfg!(debug_assertions)`, not an executed release-binary test. The permanent settings branch, visible qualification caption, saved-rerun control and compact/zoom behavior must be observed in the admitted native procedure. No extra cohort is requested merely to replace these honest evidence limits.

Read-only inspection of implementer logs confirms `.generated/voxel-material/m4-native-controls.log` records six passed fixtures, zero failures, and `.generated/voxel-material/m4-native-clippy.log` records a successful check. The parent separately reports passing affected voxel/Scene/Contents/force/setup/source-payload checks. These are implementer-executed checks; the reviewer did not rerun them. Compile/type/style success supports, but does not decide, semantic or native acceptance.

## Remaining gates

Before the first native workload, freeze this source/binary identity and the independently specified metric brush masks and bounded schedule. Use the ordinary non-test main, real controls and existing Worker on the stated owned authenticated Xorg/Mesa session; the cfg(test) native presentation main cannot satisfy this gate. Preserve actual saved IDs, constructor/publication fields, original recipe and archive identities. Distinguish software GL, authoritative cells and screenshot appearance. The admitted baseline, fresh native saved rerun and promptly cancelled bounded attempt remain separate observations; completion winning a Cancel race does not establish cancellation. Native evidence and any audit-only helper introduced later need their own review. No A5/A6 or normal exposure decision follows from this source disposition.

## Reviewed identities

SHA-256 of the final formatted snapshot read in this round:

| Owner | SHA-256 |
| --- | --- |
| `src/bin/rheon-desktop.rs` | `23a96a09f5f31aa6a8aa757bdcb3da309b602c1da184320fbe653d6460ff91dd` |
| `src/bin/desktop/voxel_material_setup.rs` | `c5418dc76111d00535993f12cb2ed556813c5f2b9b9057db5396865a549bd20f` |
| `src/bin/desktop/initial_contents_ui.rs` | `422b66e761178b4ed0712dbfd89f5e6d241e371267117e9e315c5bd9247b5ce5` |
| `src/bin/desktop/scene_ui.rs` | `2bb367f9464f52bf88b0c3a79b3266e71f7e294f12304f488644a955c8d9bf0a` |
| `src/bin/desktop/run_set_view.rs` | `2ae5c608ba5efbe4d12a333a6cbc8fa7b9d7cc842e70b0b2017cbb3c34dbceb2` |
| `src/bin/desktop/voxel_material_native_tests.rs` | `4e469d036170dbc2e17a234baf06771ba8cf80e8b7b34f600a09fc35950e523e` |
| `src/bin/desktop/force_runtime_ui_tests.rs` | `d65a8682bf9239441d64fbc3f230af3449827a5494792d9078aa23ba203fd0a9` |
| `src/bin/desktop/native_presentation_harness.rs` | `bfa6afd0fa968225c323dc18e0f4480be548d7ad5d80ff50363a47b56510b6da` |
| `src/bin/desktop/initial_contents_ui_tests.rs` | `4b81d5cfb7205a23a3ccc590ae48ec215854412e9eb001be102aa37bb202f073` |
| `src/bin/desktop/voxel_material_authoring_tests.rs` | `bc2f6e744139ea87444d47630db8616e09366ec85f65808ccba7bb71f74acb9b` |
| `reports/m4-native-admission.md` | `b35f881b0ca08bb952006613520240768e4214bb2be2200c73660f055e30ef26` |

No action attempted by this reviewer was denied, pending or uncertain. Historical denied external/package actions and uncertain Library uploads were not retried. The remaining native gate is pending by design, not replaced with this source review.
