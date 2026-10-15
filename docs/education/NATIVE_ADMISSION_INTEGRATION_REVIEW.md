# Recording and periodic S1 source admission review

This is a bounded source admission decision for **attempting** fresh native and
book qualification. It is not a numerical receipt, S1 acceptance, native-window
review, restart contract, or authorization to merge an unqualified component.

The candidate is the isolated integration branch at
`21d4122c3d0edb2ca5c3ce089175c70035d737a3`, descended from main
`06eb5daee9e77e2b0843a5ee0a4105f6950e9b1b`. Trial merges retain recording PR69
`b1acf2e514b88b528cf43e873035a598a1080ae1`, carrier PR71
`ecfc741af733f909c79170530080a98fdc5c78eb`, validation PR73
`27193c77b173a7437494584438847fedc8ecf552`, and native S1 PR74
`5a049ed604b99817a428d10bbc0140732d3421d6` as ancestors. The reviewed prospective
canonical plan at `fc2270a85e38080d9943048aaf689d99545d3715` stays byte-identical
to current main. No component or main ref is changed by trial composition.
Parent subsequently delivered final PR74 head
`a0efbc1aaf67bb65a724de028d4a773270977875`. Its two documentation changes were
reviewed and trial-merged at `c29cb2a`; the entire native source closure is
unchanged from the above candidate. S1 A/C independent evidence is now supplied,
with checkpoint disposition still owned by the parent and canonical planning owner.

## Complete admission closure and ownership

The previous runtime epoch contains 73 native/build/independent-qualification
paths. This candidate has 78: five added, nine changed, none removed. All other
64 paths, including grid geometry, aligned strain, pressure/SGS numerics, the
aligned native example, rational oracle and qualification runner, retain their
runtime-admitted bytes. Exact path-set comparison covers every `src/` file,
`.cargo/` input, Rust toolchain file and existing qualification input; it does
not project away unused modules to bypass complete crate admission.

| Added/changed inputs | Review basis and bounded claim |
| --- | --- |
| Cargo.toml/Cargo.lock | Source-identical to reviewed recording PR71. Pinned native-only rusqlite 0.37.0 (MIT) and sha2 0.10.9 (MIT/Apache-2.0); SQLite is explicitly bundled. Existing package identities remain, with locked transitive additions. No S1 dependency added. SHA256 detects accidental archive corruption; it does not authenticate hostile rewriting. |
| src/recording.rs, recording/material.rs, recording/carrier.rs | Unchanged PR69/71 implementations and independent lifecycle review. Bounded accepted-publication capture, transactional committed prefix, indexed field decoding and strict identities/units/association/stage. Poison/uncertain terminal writes require cold reconciliation; no completion retry. Keep-all then stop at explicit limits; missing accepted tails disclosed. No universal field coverage or integrator restart claim. |
| src/simulation.rs | Unchanged PR71 success-stamped interval pressure accessor and actual controls/configuration. Scratch is invalidated before attempts and published only after acceptance. Solver arithmetic is unchanged. Thermodynamic pressure from S1 remains independent of this gauge-fixed projection pressure. |
| src/vtk_export.rs | Unchanged additive PR71 carrier contract-v1 VTI writer. Native f32 faces/tracer and f64 accepted interval pressure preserve association/stage/time/gauge; missing fields remain absent. Derived cell velocities are labelled derived. Existing material/session contracts retained. |
| src/bin/desktop/archive_view.rs and changed desktop history/inspector/particle_view/main | Unchanged PR69/71 persistence and selection integration. Cold selection and CSV/VTK use selected numerical publication, never latest-state substitution. Observed audit/read failures invalidate all cached display/export capability. Explicit selections perform I/O; immutable repaint reuse is separate. Headless egui tests do not establish native-window rendering. |
| src/compressible.rs | Unchanged independently reviewed PR74 bounded periodic first-order ideal-gas LLF/Euler owner. Five conservative cell-average components, unique periodic faces, atomic acceptance/retry/cancellation and compensated budgets. No link to incompressible pressure/session/recording owners, GPU or S2 scope. Final owner A/C evidence and independent prospective B/D/E/F/G evidence remain subject-pinned, separate from combined publication qualification. |
| src/lib.rs | Sole composition conflict, resolved additively: retain native-only recording module and all S1 exports. Prior exports unchanged. No conditional solver selection, fallback method, shared EOS or changed caller route. |

The validation tool and native field producer remain byte-identical to their
reviewed component subjects. Their original prospective 16/24/32 receipt remains
evidence for its exact source; it is not relabelled as integration-source evidence.
Original 8/12/20 failure receipts remain failed. Independent review and focused
tests cover component source behavior; clean committed fresh qualification must
cover this actual composed crate and all build inputs.

Independent GPT-6.1 Sol High recording reviewer inspected this composition and
the epoch protocol read-only. It confirmed numerical source separation and the
unchanged qualification kernel, and found the standalone validation lock stale.
The isolated integration repair adds 22 required native identities, preserving
all 49 prior identities/checksums. Precise alignment of newly resolved cc/smallvec
retains the reviewed root graph's versions; all additions now match root lock
identities. The [license addendum](../plans/recording-compressible-integration/reports/dependency-licenses.json)
records local registry MIT/Zlib texts and bundled SQLite source identity. Root
and browser locks preserve previous identities/checksums with 13/21 additions.
No new dependency is admitted through floating-version resolution. Locked
metadata and build checks remain required on the repaired standalone harness.

## Explicit new epoch and preserved receipts

Preserve the historical inventory bytes at SHA256
`84c036618b32de66623d8ded24ae8cf95194457e36b9a130087a9c2bf3bd3f6a` and the runtime
inventory bytes at SHA256
`6ada2133501aab449ed7ee73f802d8a4593db50c857f0a8f8ce2af4507721d6d`.
The new separately named integration version-three inventory pins this reviewed
composition ancestry and all 78 current paths. It binds both predecessor digests
and this review record. A present malformed/stale new epoch refuses admission;
it cannot fall back to an earlier epoch. Fresh native receipts must bind all
three inventory files, this review record, every native digest, and their actual
committed blobs. There is no automatic inventory generation in any build or CI.

The immutable primary tree and ancestry checks stay in force. Source identity is
an admission input, never numerical qualification. Require fresh clean committed
native/rational execution, real-packet rejection tests normally and under `-O`,
locked native/browser builds, and full hosted book/runtime/publication checks.
Lean proof bytes are unchanged; no new kernel claim follows from these additions.
Native window inspection is pending until a supported display is available.
Execution receipts and final review outcomes belong in the integration plan's
evidence report, separate from this fixed admission decision.

## Configurable diagnostic pressure M1 admission, 2026-10-15

The previous 78-path v3 review above remains historical evidence. This explicit
review refreshes that same integration epoch after PR84 main `c85dfee` and selected
comparison imports `20b451d`/`d448563`. The independent GPT-6.1 Sol High reviewer
confirmed the complete current closure: 90 paths, 12 added, 6 changed, 72 unchanged,
none removed. The unchanged v3 schema/base ancestry and v1/v2 predecessor hashes
remain authoritative. Prior qualification receipts are not evidence for this tree.

Added paths are `desktop/ai_comparison/{adapter,cpu_contract_tests,engine,mod,
model_process,record,scene,view,worker}.rs`, `desktop/camera.rs`,
`src/bin/rheon-ai-compare.rs` and `src/solver_provider/context.rs` (desktop paths
under `src/bin`). Changed paths are Cargo.toml/lock, desktop/README.md,
desktop/viewport.rs, src/bin/rheon-desktop.rs and src/solver_provider.rs.

The selected original comparison imports retain the existing sealed-water native
operator and add an independent diagnostic trajectory/UI, shared camera and owned
worker/process. They do not modify Session, Simulation, accepted recording or
aligned-strain numerical kernels. The new public context is additive and checks
borrowed physical/lineage coherence; it is not upstream composition acceptance.
Manifest/CPU protocol uses closed bounded trusted-local contracts and existing
process ownership, now with queued input writes inside prediction deadlines.
Generic frame-v3 does not invent a trained checkpoint; legacy frame-v2 remains.
Source review closed history/domain, legacy-reader and finished-worker UI findings.
Focused actual stdlib CPU tests and desktop Clippy passed before admission; they
are not trained Torch/model, native-window or descendant-cleanup qualification.

Cargo adds the comparison binary and pinned optional serde 1.0.229 / serde_json
1.0.154 (float_roundtrip), activated only by desktop; no new crate identity or version is
introduced into the lock (only the root package dependency list changes).
Viewport camera movement was extracted into the shared owner with existing tests;
unrelated training/voxel work is excluded. All conservative selector paths,
including test-only desktop modules, are inventoried without projecting an assumed
transitive numerical subset. Independent review found no material admission issue.

This authorizes attempting fresh qualification only. The reviewed bytes are bound
by the refreshed inventory and this record's digest, committed together. Require
clean-source native/rational execution, packet rejection tests normally/under -O
and hosted checks on this exact source. Existing missing trained deployment and
native window cannot be qualified by source admission. Future M2/M3 source wiring
requires its own explicit reviewed inventory refresh and fresh receipt.

### M1 liveness-oracle correction

The selected environment's /proc mount does not reflect the child PID namespace:
a live owned child may already have no /proc/PID entry. Consequently this
execution environment's previous negative /proc checks cannot prove termination.
Independent GPT-6.1 Sol High source review approved strengthening both selected
test-source files (cpu_contract_tests.rs and worker.rs): a fixed `/usr/bin/kill
-0 -- PID` probe must first succeed while the child is resident and fail only
after owned Drop/join. Probe spawn/abnormal-status errors fail the test. Fresh
focused CPU tests passed these positive-before-negative checks. The 90-path M1
closure and all production/numerical/dependency/qualifier/oracle bytes are unchanged;
only the two test-source digests and review binding change. This is immediate-child
evidence with existing wait/join ordering, not descendant custody or a hard bound.
Historical M0 producer receipts remain historical and are not rewritten. Refresh
pins and run fresh clean qualification for this exact test-source revision.

## Saved native intent and desktop M2 admission, 2026-10-15

Relative to the reviewed M1 90-path closure, this explicitly reviewed composition
has 92 paths: added desktop/run_sets.rs and desktop/run_set_view.rs; changed desktop/
inspector.rs and src/bin/rheon-desktop.rs; none removed. All paths are included by
the unchanged conservative selector. The recording schema, numerical kernels,
aligned-strain example, oracle and qualifier, root dependency identities and all
other selected bytes remain unchanged. Historical v1/v2 digests remain preserved.

Independent GPT-6.1 Sol High review admits this source to attempt qualification:
new app modules own immutable intent/member custody and explicit cached cold views;
Inspector only delegates a bound constructor run to existing StepRecorder. The
existing worker remains the sole numerical loop, using the exact frozen dt/source
schedule and recording accepted states before any fixed-dt unsupported return.
Writer closure precedes file pinning. Successful publication alone updates the UI
path receipt; cancellation/uncertainty retains exact honest member state. Reruns
create fresh roots/run/attempt identities from captured intent, never resume fields
or overwrite artifacts. Tests cover actual joined app success/cancellation,
constructor-zero rerun bits, cold identity and immutable originals. Source review
fixed terminal path handoff and fresh attempt identity before acceptance.

This is the native slice only. Typed accepted provider/diagnostic storage, trained
Torch, native window, Lean/book publication and broader generic milestone gates
remain distinct. Final inventory/review digests are committed before fresh clean
qualification, and evidence is recorded outside this fixed admission review.

### M2 final composition follow-through

The serial composition retains M1's reviewed positive-before-negative OS liveness
probes and selects the companion's newer constructor-capture and real storage-
failure contracts (`33a689d`, `c1079bc`). Independent review confirms constructor
capture is generation/time-zero only and reads actual typed controls; shared
app-private bounded read/publication helpers preserve version/duplicate-key,
byte-limit and no-overwrite checks. Actual capacity, terminal-busy and cold
interleaving tests cover retained accepted tails and honest incomplete states.
There is no new dependency, numerical arithmetic or archive-schema change.
The closure is still exactly 92 paths; current run-set and two PID-test source
digests are refreshed explicitly. Source review admits a fresh clean isolated-
target qualification attempt, not any previous receipt or full generic acceptance.


### M3 native comparison composition admission

Independent GPT-6.1 Sol High source review admits the integrated native analysis
and saved comparison owners to attempt fresh qualification. The closure is 94
paths: adds src/recording/comparison.rs and src/bin/desktop/comparison_archive.rs;
none removed. Refresh src/recording.rs and src/bin/rheon-desktop.rs registrations
and retain the final M2 and nonvacuous M1 liveness source bindings. Historical v1/v2
inventories, conservative selector, numerical gates and qualifier are unchanged.

Native analysis uses exact publication identities, physical-time/stage alignment,
bit-identical geometry, explicit units/gauge and unavailable/unmatched coverage.
Its public sample wrapper now rejects derived nonfinite divergence as
ArithmeticRange; a finite-face/extreme-spacing regression covers this review fix.
A separate stdlib native-bit/SQLite oracle rejects wrong identities and metrics
normally and under Python optimization without relying on assert statements.

Saved recipes embed exact M2 revision and selected-publication identities. Cold
results remain Unverified until exact pinned archives are reopened and all
coverage/metrics recomputed. Bounded codec/publication helpers preserve schema,
duplicate-key, selection/work limits and no-overwrite/uncertain publication.
No numerical loop, model execution, accepted schema or dependency was added.
Actual tests use the public analysis API, not a copied private module. Source
review has no remaining material finding. Native comparison-page interaction,
measured trained runtime, accepted-provider and diagnostic-mode acceptance remain
separate gates; this admission grants no stale receipt or milestone acceptance.
### Cold native rerun identity correction

Actual software-window producer-exit/cold-rerun interaction exposed source run-ID
reuse when the existing process-local counter restarted. Root Request now retains
ALL direct source member run IDs before clearing attempts, and the existing worker
skips those IDs when assigning a saved rerun identity. Ordinary process-local
identity/provenance semantics remain unchanged; this is freshness relative to the
selected source revision, not global or full-ancestry uniqueness. No numerical,
archive-schema, dependency, qualifier or timeout change.

Independent GPT-6.1 Sol High review confirms all Request constructors and the
source-relative exclusion. New regression invokes actual App execution in separate
producer and cold-rerun processes, requires the new-ID set disjoint from all source
IDs, preserves typed specs/native time/field bits and snapshots EVERY source file
before rerun to verify no overwrite afterward. The closure remains 92 paths; only
the root desktop source hash changes. This admits fresh qualification, not transfer
of d8b0eb1 evidence or trained/generic acceptance.


### Native cold comparison page and root navigation

Select companion bb48cee's new cached page and seven headless interaction
contracts, binding tests to the actual public recording::comparison API. Existing
root app registers the page and supplies explicit sidebar/run-set navigation and
Back handling. The existing worker's poll/cancel/close path precedes page routing;
new entry is disabled during work. Run-set action passes that exact manifest/root,
not live/latest data. Independent GPT-6.1 Sol High review confirms selection edits
invalidate stale capability, Unverified cannot supply a pair, compute/verify reopen
exact pins, and source reads occur on explicit actions rather than each repaint.
No remaining material source/navigation finding in this reviewed delta.

The closure is 95 paths: adds src/bin/desktop/comparison.rs and refreshes desktop
root with navigation plus the reviewed cold-rerun ID correction. No removed paths;
predecessor inventories, numerical/runtime/recording owners, conservative selector,
qualifier and physical thresholds stay unchanged. This admits fresh qualification
of this source, not transfer of companion harness or prior integration receipts.
Linked field projection, actual root-window interaction and typed diagnostic modes
remain separately qualified work; no full M3 acceptance follows from page wiring.


### Exact native presentation values

Independently reviewed selected additive 06ccf43 numerical/page hooks retain
actual public raw native_samples and its overflow regression. compared_values
first authorizes through existing compare, then applies only the returned declared
pressure offset; it carries exact native geometry/lattice dimensions/associations,
not cell-averaged faces. Page materialization explicitly reopens both whole-file
pins and exact selected-publication identities and matches the verified summary.
Source failures clear capability; scientifically unavailable pairs retain analysis
but return no materialized array. Renderer caches must clear on every Err/None.
Caller/source budgets bound selected payload; no peak RSS promise is introduced.

Source review caught assert-only decisive checks in the new independent pointwise
Python oracle. Replaced them with explicit checked failures and tested valid pass
plus perturbed sample rejection in normal, -O and PYTHONOPTIMIZE=1 modes. The
existing independent metric oracle remains unchanged. Focused20 numerical+9 page
contracts and scoped Clippy passed; the reviewer ran no tests. No remaining
material source issue in the reviewed correction. Still95 paths; refresh only the
two owned existing comparison source hashes. Fresh clean qualification required;
companion/root/previous source receipts are not substituted.

## Linked native slice/probe root integration, 2026-10-15

This explicit source review admits the 95-to-96-path native M3 extension to
attempt fresh qualification. Selected companion PR93 source76cc71af is retained
as90ee568: new comparison_field_view.rs SHA256
911b7095ccc2897d811b7946a5fabadc31703263879bec73d9b8f082573d1106;
contract tests and API documentation retain their frozen companion bytes.
Independent GPT-6.1 Sol review checked the actual source and tiny root registration:
identity includes root, captured set, both members and complete recipe/pair;
changed/absent/unavailable selections and failed reloads clear cached fields;
source/payload/sample/plane limits precede materialization. Native cells/faces,
physical XYZ/time/stage/pressure intervals, gauge diagnostics and common bounds
remain explicit; repaint performs no I/O. The root owns one persistent field view,
updates page controls before selection and wraps both views in a scroll region.
No numerical, worker, recording, model or qualification owner changes. All96
conservatively selected inputs are rebound; prior receipts remain historical.
Admission authorizes fresh clean committed tests/native-window and native/rational
qualification, not a transferred companion receipt or trained-model acceptance.
