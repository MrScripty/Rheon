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


### Partial typed diagnostic storage admission

Select independently reviewed new diagnostic archive module and actual SQLite
contracts from f87a7a2/fb8405a, with serial recording-owned module registration and
explicit accepted-reader archive-kind rejection. The selected closure is96 paths:
adds src/recording/diagnostic.rs and updates src/recording.rs only beyond final M3.
Accepted schema1 writer/publication types are unchanged; default application_id0
is accepted, while nonzero archive kinds reject on open and subsequent header
verification. Diagnostic application_id RHDP has distinct typed tables, request/
attempt ordinals, host-observed qualification/transition and terminal prefix facts.
No candidate becomes a RecordedStep or accepted Session publication.

Independent GPT-6.1 Sol High review closed three material findings before admission:
request/header domain matching now preserves exact floating bits (including signed
zero); known rejected seeds cannot initialize either mode and teacher seed status
must be explicitly observed Accepted; failed/cancelled unrecorded-tail generation
claims are checked against seed, produced count and durable prefix with checked
arithmetic. Regressions cover invalid/legitimate chronology, signed-zero rejected
append with cold prefix preserved, native teacher Accepted history, biased model
observations leaving actual native evolution unchanged, and kind/header mutation.
No remaining material source finding. The reviewer ran no Cargo or runtime.

The existing recording owner supplies bounded SQLite connections, transaction and
poison lifecycle. Header/request/sample/transition hashes, bounded decoding and cold
verification preserve honest prefixes; Drop never completes. Native-prefix length
is explicit caller-observed declaration, not inferred lineage/qualification proof.
This admits fresh qualification of partial storage only. Engine/model-process/
worker capture, saved diagnostic specs and full teacher/stop A6 remain pending;
no measured model, source authenticity, durability or global physics claim follows.

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

## Typed runtime plus linked native composition admission, 2026-10-15

This explicit independent GPT-6.1 Sol review admits the combined103-path source
closure to attempt fresh qualification. Relative to partial storage96 paths it
adds diagnostic_runs/view/worker/worker_tests/engine_typed_tests/
prediction_contract_tests under desktop/ai_comparison and the separately reviewed
linked comparison_field_view.rs. Existing engine/mod/model_process/record/worker,
comparison CLI, recording/comparison and recording/diagnostic change; root desktop
retains the independently reviewed linked registration. No path is removed,
selector narrowed, numerical qualification input changed or dependency added.
Historical v1/v2 inventory bytes and v3 ancestry remain unchanged.

The existing engine performs teacher and independent stop policies; exact committed
endpoints carry history, teacher future stays native, and an independent reference
uses the identical captured RHS. Requests/candidates/actual host thresholds and
wall observations are separate typed schema2 attempts, never accepted ordinals.
Native diagnostic seed origin stays diagnostic. Cancellation before advance retains
candidate evidence; append precedes JSON/UI, poisoned writes stop execution and
only a reconciled terminal can pin after close. Legacy policy and JSON remain.
Typed process failures retain dispatch and actual invalid-wire digests; frozen
provider identity is checked before spawn. The existing leased Record/Worker and
SQLite owners retain lifetime/persistence, with fresh frozen intent and create-new
reruns, exact cold source/category checks and honest unrecorded tails.

Independent review closed cold statistics tampering, persistence/cancellation
precedence, incorrect offset-gauge labeling, programmatic inspector namespace
invalidation, typed CLI failure exit and old-frame/new-rerun-geometry findings.
Actual worker/cold/subprocess tests and the egui5-to6-grid rerun button regression
exercise those boundaries. The explicit cold inspector caches one request, shares
native physical slice/probe/common range and displays real availability. Summary
export reopens the exact manifest/pinned archive, compares the cached attempt and
publishes create-new. The new stdlib verify_diagnostic_pressure.py derives raw
SQLite IEEE norms with fsum and explicit checks; changed-metric rejection remains
active normally, under -O and PYTHONOPTIMIZE=1. The shared reducer preserves existing
native metric order and semantics, with direct canonical gauge and no offset.

This is source admission, not a transferred fixture/model receipt or completion
of A1/A2/full generic milestone gates. Freeze this complete composition and run
fresh native/rational, recording, desktop and actual native-window qualification.
Measured trained bundle/runtime/parity remain separately blocked as documented.

### Saved-executor provenance correction

Independent review identified an execution provenance gap after the combined
freeze: a saved old source stamp could be copied into a new archive under a
different executor. The corrected source shares current_executor_revision between
capture and an execution-only full-stamp equality guard. Worker.start_saved rejects
before output/thread/process creation; the execution owner checks again before
spawn. Historical validate/digest/open/header verification remain unchanged.
Equal unspecified stamps explicitly do not prove build parity; no frozen intent
or digest is silently rewritten. Independent GPT-6.1 Sol review approved the four
changed selected sources (diagnostic_runs, worker, diagnostic_worker and its tests).
The new real-process regression proves a mismatched stamp has no spawn marker or
output root, explicit manufactured historical cancelled metadata cold-opens Ready,
and a matching stamp executes to completion. Focused test and strict binary Clippy
passed. Still 103 conservative paths; refresh these four pins and the review
binding, then qualify the clean corrected tree. Earlier receipts remain evidence
for their exact subjects, not this correction.

The saved-rerun error consumer also retains execution failure in the inspector's
status, so an existing finished worker's repaint cannot erase the mismatch
explanation from App.status. Valid historical fields remain inspectable. Independent
review approved this additive mod/diagnostic_view correction; both affected UI
contracts and strict binary Clippy passed. Rebind those two selected source hashes
as well (still 103 paths) before final clean qualification. No solver, context,
archive, matching-stamp execution or historical-read behavior changes.


### Diagnostic custody, chronology and oracle composition, 2026-10-15

The lead selected only PR96 d58044a and PR97 68b8bd1's five allocated files onto
frozen PR95 34f3d1d. Independent GPT-6.1 Sol review approved the scaled-fsum oracle
and saved-intent-before-mutable-preflight ordering. A remaining app-only chronology
gap was found and corrected before admission: the generic recorder deliberately
allows broader seeds, whereas this app begins at its actual zero-origin native
prefix. The app now requires no seed at prefix zero, and otherwise native-qualified
diagnostic origin/status, exact prefix generation and repeated-DT end time.
The produced horizon and full-prefix checks apply before Ready/Unpinned/Incomplete
classification. Genuine within-horizon capacity tails remain permitted.

The independent reviewer approved the completed guard and public-recorder
regression: generic verification succeeds first, then the app rejects shifted
generation, one-ULP time, zero-prefix-with-seed and AcceptedNative-origin sources
across Failed, Cancelled and genuinely appended incomplete archives, without
observations or pins. Existing valid history and capacity-tail coverage is retained.
Reviewed final SHA256:

- diagnostic_runs.rs: 7038fff60a8dc88a64236aa12e88c2d4ce856cf93cc4261adf29743644663bf5
- diagnostic_worker.rs: a75e78beff3c85dec528c54c15a29c6ce6db7c918279bda2af7b5af91104991b
- diagnostic_worker_tests.rs: fd39822957fed70fac97bf5025cdb57028817c414175320b901119dac4fa50a8
- tools/verify_diagnostic_pressure.py: afa9845b1513a4119777f9564e5020e8503e0eabaed897aead8e2a11e9f5a299
- tests/diagnostic_pressure_oracle_contract.rs: b492194a384779fd15ba967bac93a5068d1635d5ff5b443d13a31252716134d7

Only the first three files belong to the unchanged conservative native selector;
the complete closure remains 103 paths. The oracle and automatically discovered
Cargo contract are reviewed explicitly above, not added to or hidden by a changed
selector. The qualification path, numerical gates, generic archive/schema, accepted
Session/provider semantics and source companion branches remain unchanged. This
admits fresh composed qualification; it transfers no earlier receipt or model gate.


### Cached inspector and synchronized projection composition

PR98 249b66b contributes only diagnostic_view.rs and mod.rs. Independent GPT-6.1
Sol read-only review found no material source issue in its composition with529a9e22.
The display cache retains a strong Arc to the exact immutable attempt, builds the
request comparison only on replacement, and draws retained metric/range values
without full-volume digest/reduction or archive I/O. Visible slice drawing still
visits the plane's cells. New/absent publications, failed loads and changed owner/
grid clear the appropriate state. Live diagnostic controls have independent
ownership from the legacy frame controls. The selected probe's X/Y remap onto the
current Z before both panes and physical readout. Actual native reference gate/
history labels remain distinct from candidate gate observations. Explicit export
and rerun reopen and verify source custody independently of the projection cache.

Reviewed exact SHA256:

- diagnostic_view.rs: 1e6b8011f945ba05ce04c8e4c7f88de23bd53fc99c776311fd30d3021a1c7fe2
- mod.rs: f419a639e07f30da0e039897a2496584d083fcb347c40d370e0e1168c7cfdfb2

The complete selector remains103 paths, with these two reviewed pins rebound in
addition to the three custody pins above. No numerical/schema/provider/accepted
archive semantics, source selectors or qualification gates change. Earlier529a
receipts remain pinned to that earlier subject; the final seven-file proposal
composition requires fresh clean qualification and current native-window evidence.


### Accepted Session/context and sealed Newtonian composition

Independent GPT-6.1 Sol read-only source and fixture review approved the
six numerical/API files and public contract tests. The complete conservative
closure grows from103 to105 paths: two added modules, four changed sources,
99 unchanged paths and none removed. The selector, qualification runner,
independent rational oracle, historical inventories and numerical gates stay
unchanged. This admits fresh clean qualification only, not model or full-plan
acceptance. Review and limits: ../plans/configurable-solver-components/reports/accepted-session-review.md.

The existing accepted pressure array supplies borrowed prior history; the
Session owns context lineage, physical composition and atomic publication.
The existing viscosity kernel precedes actual RHS formation. Invalid identity
metadata is dropped before accepted receipts. Eight public native-fixture
contracts and a deterministic final-callback deadline regression retain
pressure/volume/cancellation gates and bounded allocations. Runtime bridge,
typed persistence and measured aa82 parity remain separate gates.

- src/liquid_step.rs: f99dce6a99f6af8a9aef2436af7f33de58dac3be5998a48e9187fbdf2acc3f93
- src/session.rs: 604e579bf4a2c780c4941889230654432a2e491059013a94f20300b5ecbe665d
- src/session/configured.rs: f8a8b7687de99884ad2b7790e9fb3da9c9674d0bcc1a19a21b1e6f7751ac4187
- src/simulation.rs: 8dea790daf2a2d0801db3003094f9fffe034c9b87946d520a1eacc266c95f078
- src/solver_provider.rs: 0ab7db08a9016e73d664c730e4ff513d42599a84c680ea061e6604859432a5a5
- src/solver_provider/configured.rs: 124ca65dd1dec3a8a89043198fb085453a42ca8aff0dc87c67b04b6a986109be
- tests/configured_provider_session_contract.rs: ba4a8f26fbf0324d705a80a4b108b1e9d09a293ac2eb29a8278fe01f65dc5a00


### Immutable accepted constructor configuration readback

Independent GPT-6.1 Sol review approved the additive SessionConfiguration
readback and its tests. The Session stores the same validated constructor inputs
used for its owners and returns only an immutable borrow. Original logical
memory limits remain constructor controls, distinct from internal budget splits;
per-call viscosity/provider/forcing schedules are not inferred. Both native
implementations are covered across constructor, acceptance, failure, cancellation
and stop. Review: ../plans/configurable-solver-components/reports/session-configuration-review.md.

- src/lib.rs: 91f3cdc1a9b0f782e2efcc647eb956230b201e6b0a3429e4b2cd4955f5f2a2eb
- src/session.rs: 2caa26be33d5497609bfe1b6f16456d217372119d0720a989697fbf5debb0544
- tests/session_configuration_contract.rs: 4f53c8d35eeb44819791341b8ce1a80421d9562ab8413f528a3234da233632f7

The selector remains105 paths. Only these two selected source pins change;
103 other selected paths, historical inventories, oracle and qualification
runner remain unchanged. This is admission for fresh clean qualification only.


### Exact accepted Session interval-start authority

Independent GPT-6.1 Sol review approved owner-captured pre-step time for
all three accepted Session paths. Constructor is None; rejected/cancelled
attempts and stop preserve the last accepted start. The public regression
executes64 nonbinary1/60 intervals for all3 paths and both native methods,
requires a genuine end-minus-dt bit counterexample and verifies failed-attempt
retention/retry/quota. Review: ../plans/configurable-solver-components/reports/session-interval-review.md.

- src/session.rs: 367c370e2257e35de09943e8dc3e1257b994c5e01d1409af1a989682f41c583e
- src/session/configured.rs: d99ae990e9c7ddc956f95f414a25832411a4dc84ab434df95ff9f8f68cf52a83
- tests/session_interval_start_contract.rs: 4cdc0a85d21d77784ef142ba04f1ef20fb0333555d7bbb15b43465f475934769

The unchanged105-path selector rebinds only these two numerical-owner source
pins. Other103 selected paths, old inventories, mathematical oracle and runner
remain unchanged. This admits fresh clean qualification, not old receipt reuse.


### Portable accepted Session hash dependency

Hosted browser-fixture qualification exposed a real compile regression: the new
unconditional core configured context imports sha2 while Cargo declared it only
for native targets. Independent GPT-6.1 Sol review approves moving the existing
exactsha2=0.10.9 dependency to the common table, with no version/feature/default
change. Both lockfiles already contain the same package and remain unchanged.
The fixed actual wasm32 library build passes; complete fresh browser/native
qualification is still required. Review:
../plans/configurable-solver-components/reports/portable-session-hash-review.md.

- Cargo.toml: 10a7b438ec5e7bdb2c8eb95137cf5a42d1cd35687ed47bf63b05a3e42dc1674e

The complete selector remains105 paths, only this manifest pin changes and104
selected paths are unchanged. Native-only filesystem/SQLite/process capabilities,
numerical kernels/gates, selectors and historical inventories remain unchanged.
Native qualification alone does not establish this browser regression fixed.



### Frozen configured Session constructor and future intent

Independent GPT-6.1 Sol read-only review approves the distinct schema2 pure
intent module, bounded immutable constructor/future controls, exact host versus
provider declarations and explicit parent registration. Native schema1 bytes and
meaning remain unchanged. This module executes no interval/provider and grants
no archive Ready state. Recording binding, runtime admission, production worker
composition and A4 acceptance remain separate gates. Review:
../plans/configurable-solver-components/reports/configured-intent-review.md.

- src/bin/desktop/run_sets/configured.rs: 606e435e27f6138df93c7d7ddc205e921c066da70c476a4a40232ac0c72c53a4
- src/bin/desktop/run_sets.rs: 405332f953a31e476a704ce189b85781faf19620a2c54e5b4cb077fdd95f2a5f
- tests/configured_run_sets_contract.rs: 29b29888112126677262bdd05792cad70170338eaac8c4767fc2ab0292a9e927
- tests/saved_run_set_contract.rs: 6e001075a4307da0e51df04c4521bc8599194ce06fbf010c08e34bab8f693ef9

The complete conservative selector grows105 to106: one added module, one
registration-only changed source and104 unchanged selected paths. Existing
historical inventories, selector, mathematical oracle and qualification runner
are unchanged. Only these explicitly reviewed hashes are admitted; no automatic
refresh occurs. This permits fresh clean qualification, not receipt transfer.


### RCP2 configured recording and cold immutable member binding

Independent GPT-6.1 Sol lead reviews approve the exact RCP2 profile and cold
archive binding over the existing accepted SQLite envelope. The profile captures
actual viscosity-stage presence, full constructor controls, exact interval starts
and typed bounded accepted/fallback lineage. Closed frozen members additionally
compare every constructor/header control even when empty; requested versus actual
interval policy; exact applied viscosity and provider/prefix facts; known terminal
tail and completed horizon; and protected closed-file hashes before/after audit.
Cold opening creates no Session/provider/runtime or deployment resolution.
Force/request scheduling remains a producer declaration, not reconstructed RHS
proof. Failed/Cancelled may retain a declared unrecorded accepted tail; unknown
accepted tail cannot become Ready. Reviews:
../plans/configurable-solver-components/reports/configured-recording-stage-review.md
and ../plans/configurable-solver-components/reports/configured-archive-review.md.

- src/bin/desktop/run_sets/configured.rs: 45fa8cf6976da7f8ab3eff27fd69ba5511c98c4be541739eefcdf762826abac9
- src/bin/desktop/run_sets/configured/archive.rs: 51a2ac2952c6dfd8881e3a5abb6080a5e785f99c5d24b67685aaa3ae01b7bf4f
- tests/configured_archive_binding_contract.rs: 16a239f34960677ab65c8106c0b21e06ecd9929a04d496938dd919eb11962004
- docs/api/configured-session-intent.md: 6c1c694d4ff06bec4117e1a78518a35e402445c96efc45f05a7c973928effce5
- src/recording/provider.rs: 91e69991e352013be0ecc48bf7fc0b6c1c75e4a7564d9c7f9ed7d2f312c52ff3
- src/recording.rs: d022f8f46c288538298896ae444c9780265a6bf77fa9a4e1ba4b9732cf026eb2

The complete conservative selector grows106 to108: two added modules, two
changed source paths and104 unchanged selected paths. The portable hash fix is
already inherited. Source selectors, old inventories, numerical kernels/gates
and mathematical oracle remain unchanged. Fresh clean final-source qualification
is required; production worker/UI/runtime custody and complete A4 are separate.



### Accepted Session / actual CPU context fixture contracts

Independent GPT-6.1 Sol review approved the optional cooperative poll in
the existing ModelProcess owner, its legacy false-callback entry points, child
contract registration and new Rust/Python two-cell fixture contracts. This is
actual analytic CPU execution through accepted Session contexts, not a learned
model, GUI runtime connection or durable typed failure recorder. The unchanged
20ms wait checkpoints are cooperative opportunities, not a response guarantee.
Cancellation kills that runtime; retry uses a fresh process owner. Existing
consumed malformed-response digest precedence remains unchanged.

Review: ../plans/configurable-solver-components/reports/accepted-cpu-context-review.md.

- src/bin/desktop/ai_comparison/accepted_provider_contract_tests.rs: 040ffd5ad5500ffa0275a61f99d0f3105618597175355db526393645c94bbf56
- src/bin/desktop/ai_comparison/cpu_contract_tests.rs: d7f2be7ea559891450f4dc1baae47183615bab8b999bc9bfe9d0151bb4d27580
- src/bin/desktop/ai_comparison/model_process.rs: 77070b2ae78dabf8f2d046a0b83f97053ef6460f8c2f19f47dcc3264e34f9754
- tests/fixtures/accepted_pressure_adapter.py: 958ba74ebde596aac8375eb436898a1e2342ba1b7868c08bfee99c09c02dc27c

The conservative selector grows105 to106 paths: one added test module,
two changed selected sources,103 unchanged paths, none removed. The separately
reviewed Python compiler fixture remains explicitly pinned above and in the
fresh whole-Git qualification manifest. No selector, gate, old inventory,
mathematical oracle, protocol schema or numerical kernel changes. This admits
fresh clean composed qualification only; earlier receipts stay source-bound.


### Reviewed CPU provider component composed with frozen cold binding

The independently reviewed CPU poll/context inputs and production provider
component are composed by ancestry with the separately reviewed RCP2/frozen
cold binding. Eight actual production-provider contracts passed within66 AI
application tests (one existing measured-deployment ignore) on the bridge source;
these are analytical stdlib fixtures, not production worker/UI acceptance.
The component retains a bounded latest failure only; its consumer must associate
it with the current prediction. Existing process/callback/Drop owners remain.
Review: ../plans/configurable-solver-components/reports/accepted-provider-bridge-review.md.

- src/bin/desktop/ai_comparison/accepted_provider_contract_tests.rs: 040ffd5ad5500ffa0275a61f99d0f3105618597175355db526393645c94bbf56
- src/bin/desktop/ai_comparison/model_process.rs: 77070b2ae78dabf8f2d046a0b83f97053ef6460f8c2f19f47dcc3264e34f9754
- src/bin/desktop/ai_comparison/cpu_contract_tests.rs: d7f2be7ea559891450f4dc1baae47183615bab8b999bc9bfe9d0151bb4d27580
- src/bin/desktop/ai_comparison/accepted_provider.rs: f86d624e4324af9f257ef2b9643b2f116443c18d3f39e34cafd2a0de6a92868f
- src/bin/desktop/ai_comparison/accepted_provider_tests.rs: 35607bb996208614933e8b1aa2c462403dba7f9c99ec322038bc4c791afaf371
- src/bin/desktop/ai_comparison/mod.rs: 81ea11b59ca3bd78efcc224be946fd55c4672d049a0b94d901b798a237a8ddb4

This explicitly admitted merge grows108 to111 selected paths (three added,
three changed,105 unchanged). No selectors, old inventories, numerical kernels
or gates change. This is source admission only: production caller wiring and
full static/runtime/recording validation remain required before integration.


### Controlled desktop tracked-material build and compiled declaration

Independent GPT-6.1 Sol reviews approve the five Rust declaration/consumer
paths and separate controlled-build wrapper, contracts and documentation.
The complete conservative selector grows106 to107 paths: one new central
module, four changed registrations/consumers,102 unchanged, none removed.
Only the exact independently reviewed source hashes below are rebound; the
selector, old inventories, numerical kernels, oracle and runner are unchanged.

Ordinary Cargo builds retain explicit unspecified source/material values when
compiled declarations are missing or malformed. The controlled wrapper exports
exact clean committed regular Git blobs into a fresh external tree, discards
inherited build overrides, records trusted tool declarations and tracked-source
material before building both binaries, and binds actual artifacts in an
external receipt. This is bounded custody, not hermeticity, authentication,
complete compiler-read closure, reproducibility or numerical/model acceptance.
Reviews: ../plans/configurable-solver-components/reports/desktop-build-stamp-review.md
and ../plans/configurable-solver-components/reports/qualified-desktop-wrapper-review.md.

- src/bin/desktop/build_identity.rs: a94fea1f039a49f8d0eb79b019b3fbd0f2c9da1762ba854d10cba1e5fd87a25b
- src/bin/desktop/identity.rs: df451a986476b9df252a9a4dbc1e33218bf5ab3cb268b83fa78b47dfffdae184
- src/bin/desktop/ai_comparison/diagnostic_runs.rs: 80559d9d69d8164c24b8dd45f71c51b972fc8af9349c9e56b3e36b14b827b41c
- src/bin/rheon-ai-compare.rs: e102cccdc1a86afbdcf5a942ec6b1125655b9d1fcc7f4eb479935940633082bc
- src/bin/rheon-desktop.rs: b484c75de20df8cb36010e31cd9cbab879e4f348050b2d551f2bcf553cb25d3f

This admits a fresh clean qualification attempt. Fake-compiler wrapper
contracts do not establish an actual native build; exact committed-source
wrapper execution and native consumers remain required evidence.


### One-interval configured Session and CPU owner composition

Independent GPT-6.1 Sol source review approves the bounded ConfiguredTrack
component, contracts, separate independent analytical fixture, bridge interval
observation reset and explicit registration. Review:
../plans/configurable-solver-components/reports/configured-track-review.md.
It reuses the existing Session, pressure/viscosity workspaces and ModelProcess;
separate inert construction/startup allows intent/constructor recording before
mutable package checks. Actual context support, fallback and whole-interval
commit stay with existing owners. Current typed failures are owned before
retirement; host quality rejection does not retire a healthy runtime.

- src/bin/desktop/ai_comparison/configured_track.rs: 9f677e5782a1547f4f46ca2280462bba08b479377a0fb00a5830a8a874872a7b
- src/bin/desktop/ai_comparison/configured_track_tests.rs: a0dcc1c09d188ad14fc42e78535cd51bcb9bacae0522f4f95262340e4bd86b7c
- src/bin/desktop/ai_comparison/accepted_provider.rs: 662e1def1b50332fa00a7aa1deded481b84ccfad1a98ffca2c7e062adbae79bd
- src/bin/desktop/ai_comparison/mod.rs: ca7f6af6aa04986bb5401383b059c1efd8ffedb672c7eb29a6895e41bc0ec370

The complete selector grows112 to114 paths: two added, two changed,110
unchanged, none removed. The Python fixture is outside that unchanged selector
and independently reviewed at01ad2acb063b21c29cc379191c1048f5a365d707de428693cec7ae30c744e98f;
whole-source committed qualification additionally binds it. No selector, old
inventory, numerical kernel, oracle or qualifier changes. This is source
admission only: production caller/recording/UI integration and strict Clippy
remain pending. The unwired graph currently has dead-code diagnostics; no
gate is waived or old qualification transferred.

### Combined accepted CPU context and configured recording closure

Lead independent GPT-6.1 Sol read-only review of PR103 exact5e6d3c7 approves
the bounded current recording/math closure for fresh qualification, composed
with separately reviewed CPU context contracts and current Session authority.
The configured profile retains raw rejected IEEE diagnostics independently of
finite accepted fields, actual interval starts, source/config/geometry binding
and cold prior/prefix validation. Existing SQL/native/material/diagnostic meanings
are unchanged. Review:
../plans/configurable-solver-components/reports/configured-recording-review.md.

Known assigned follow-ups remain: actual applied viscosity-stage presence
(Disabled differs from Newtonian zero), its versioned codec, and a public typed
header accessor for empty-archive constructor binding. These block their complete
physical/frozen Ready claims. This source admission makes none of those claims
and authorizes only fresh present-closure mathematical qualification.

- src/recording/provider.rs: c42fab3fb955ec3002cf9aaabf348edc46d47893dc367790ef92716b339ed86d
- src/recording.rs: d022f8f46c288538298896ae444c9780265a6bf77fa9a4e1ba4b9732cf026eb2
- tests/recording_provider_contract.rs: f65865366bd2918a7308e7337df7985a345f7a5facf8caf1ee1e7996b09bfa6d
- examples/recorded_configured_provider.rs: d00e9504f395e3e46d016e157eb68bc2b95124a69c1fb8cf88f2b10da58c4ffa
- docs/api/configured-provider-recording.md: f11c5e693a5ce82ce98d546ea74007f9a040ef27470d7c814fc0eeb1ec6df165

The complete conservative selector grows106 to107: one new recording module,
one registration-only changed source and105 unchanged selected paths. The
selector, old inventories, mathematical oracle and qualification runner stay
unchanged. No automated inventory refresh or old receipt reuse occurs.



### Explicit RCP2 applied-stage and cold-header correction

Independent GPT-6.1 Sol lead review approves exact PR103beb69f6 follow-up.
The actual accepted LiquidStepReport viscosity presence is retained as its own
boolean; Newtonian zero and Disabled remain distinct. RCP2 explicitly refuses
RCP1 instead of inferring the lost stage. Applied zero enforces density identity;
disabled noncanonical/nonzero and nonzero subnormal coefficients are refused.
The public typed header accessor validates constructor controls without a row
or numerical/runtime construction. Review:
../plans/configurable-solver-components/reports/configured-recording-stage-review.md.

- src/recording/provider.rs: 91e69991e352013be0ecc48bf7fc0b6c1c75e4a7564d9c7f9ed7d2f312c52ff3
- tests/recording_provider_contract.rs: cc1bf048c06054a8be52449383e215d73fb00c430d5d181186a4e8592043e6d2
- docs/api/configured-provider-recording.md: bed659ef1e03bd8b5121afd589f9bb4e795bfa4afdf8674ef3e97c5edbb76c60

All107 selected paths remain present; only provider.rs changes, with106
unchanged selected paths. Source/qualification selectors and older inventories
are unchanged. The previously assigned stage/header gap is closed in source;
fresh combined qualification and subsequent frozen/production integration
remain required. Earlierfc5a704 evidence is explicitly RCP1, not this RCP2 source.


### RCP3 runtime and RCT1 terminal custody on configured component union

Independent GPT-6.1 Sol reviews approve exact PR106 cross-contract source and
PR1085c8326a runtime/terminal codec. Reviews are retained in
../plans/configurable-solver-components/reports/accepted-recording-cross-review.md
and configured-runtime-recording-review.md in the same directory. The codec
retains exact RCP2 absence, explicit RCP3 current-request observations and typed
RCT1 retained accepted/next-unaccepted terminal association with honest tail
coverage. Tests with authored runtime metadata are not actual IPC evidence.

- src/bin/desktop/ai_comparison/accepted_recording_contract_tests.rs: 6cfddefe84a64c89f126598b817d22afe6d1b5731e1d533ea92cecbd1870c942
- src/bin/desktop/ai_comparison/accepted_provider_contract_tests.rs: 643636776b009337fb80dfdf3d5dbee975e06ed7f7a7cf3f534c07301b61c729
- src/recording/provider.rs: c4d48af7de2769349292819c48494f5a248edfb53eb58c9d53a535c165f5b427
- src/recording/provider/runtime.rs: d7b2a1f7cfa866b33887f7c8f804999058d6e70a1c9d71421372d151152ebccb

Complete committed-index closure grows114 to116 paths: two additions, two
changed sources,112 unchanged. Only exact reviewed pins are admitted; old
inventories, selector, oracle and qualifier stay unchanged. Root's separate
working-tree production caller changes are not included or admitted by this
merge. Actual process mapping, configured cold-consumer extension and fresh
final whole-source qualification remain required.


### Actual configured desktop Worker source admission

Independent GPT-6.1 Sol review in
../plans/configurable-solver-components/reports/configured-desktop-composed-review.md
approves the exact production process bridge, existing Worker/Session/recorder
composition, frozen controls, fresh saved reruns and cold Inspector changes.
All four review findings are resolved: sticky typed failure, constructor
recording under terminal ownership, distinct labels and metadata-only zero-row
inspection. Native carrier allocation is boxed to keep the dispatch enum small;
strict lints are retained. Exact reviewed source pins:

- src/bin/desktop/accepted_session.rs: 7a42a42bbe8d044f703c2c52d488bf8a4e8e3b3fb1448851efcfb04599d5bf31
- src/bin/desktop/ai_comparison/accepted_provider.rs: 922c98958bec45dbd4dee199e0bbdbdd6e3fe4ac52d7a70472ce8132461416fc
- src/bin/desktop/ai_comparison/accepted_provider_tests.rs: 11ae34ba76dc93a87f15c8ce9f745403b2d22d0194987d945cf2edc53c624237
- src/bin/desktop/ai_comparison/adapter.rs: 7038fb09ef2bd33a17495414c4c0986a6fb63ba949fe440e9d19927baca712b2
- src/bin/desktop/ai_comparison/configured_track.rs: 3959018b52697ef072884864c01de4682d1b4cd351c273426d914d02f0641fa1
- src/bin/desktop/ai_comparison/cpu_contract_tests.rs: 4d6d333c7428e0a8a2a15f5b25365c1ce1e5ce3b328d183c3f43656ac48246c9
- src/bin/desktop/ai_comparison/mod.rs: a0def0814e19e43e2d5134dc2b739a6c00f60be763bb5c615b522e21785dd0f6
- src/bin/desktop/ai_comparison/model_process.rs: e0ffa8d47466f030ecd51e2f4ab44e64787d7c91347e11ac456d1ac688d75a1a
- src/bin/desktop/archive_view.rs: 9f96f1d6b11b46e7fd523825e19a8fc0e0ec862669acd129f5e39e91f83c89ff
- src/bin/desktop/carrier_execution.rs: fd77b9b00085cda58aac8056fe35ad0fd36b9058c82a8023a313e677da390ffc
- src/bin/desktop/configured_controls.rs: 05d317554c4db033dd35c5e153dceaea34367eed33125ec8c7f59466f83ede36
- src/bin/desktop/configured_worker_tests.rs: bf8b0c0900bfd2d1fb01b93b10cab6acef27913415bfcec929b559fa0243b9c4
- src/bin/desktop/history.rs: 49dfc0cb697e2e861f8fbc5d3c0b67ded8a233283d24732344d7c4beebdab9c2
- src/bin/desktop/inspector.rs: bb6bb83529896323fbf5414d6e7f41f8462b21e80a6d1f620edb6eecd2fc6649
- src/bin/desktop/run_set_view.rs: a646384025669447cf61852b37dcdbc9189359e1180e372dc3d4042ae7a82527
- src/bin/rheon-desktop.rs: 3c03e4ff19fc9d392dcac74f682ff05d95960f668aa68abeef24ab6ea31947a3

Complete conservative closure grows116 to120 paths: four additions, twelve
changed,104 unchanged, none removed. Only the independently reviewed hashes
above are admitted; no source selector, historical inventory, numerical gate,
oracle or qualification runner changes. The accepted graph is registered in
its actual desktop consumer rather than suppressed as unused in the AI lab.

Actual focused Worker tests exercise production ModelProcess calls and durable
runtime/terminal association. Component and working-source execution receipts
retain their subjects. Fresh committed whole-source qualification is required.
Configured comparison UI and final native-window acceptance remain active
work; PR110's separately reviewed frozen terminal-clock fix is still required
before its extension is admitted. No learned accuracy, performance, restart
or final-plan completion follows from this bounded source admission.

### Accepted CPU and RCP2 recording cross-contract composition

Independent GPT-6.1 Sol source review approves PR106 exact16157372 versus
reviewed RCP2 parented4346da. The new test module uses the existing test-only
Driver/ModelProcess/SQLite owners; the sole parent change is explicit child
registration. Review and scope:
../plans/configurable-solver-components/reports/accepted-recording-cross-review.md.

Nine recording scenarios cover accepted native fallback and rejected whole
intervals across consumed replies, predispatch cancellation and Linux x86_64
proven blocked writes. Exact child readers run after runtime/Session/writer
drainage with the owned manifest hidden, requiring verified markers and
unchanged field/selection/file hashes. Producer parent process remains alive.
Retries create fresh Sessions/archives, with changed restored blocked-fixture
pins; this is not checkpoint resume or same-owner quota evidence. Fine process
reason/dispatch/payload hashes remain live-only in this RCP2 slice. No hard
shutdown, descendant, portability, learned-accuracy or production-UI claim.

- src/bin/desktop/ai_comparison/accepted_recording_contract_tests.rs: 6cfddefe84a64c89f126598b817d22afe6d1b5731e1d533ea92cecbd1870c942
- src/bin/desktop/ai_comparison/accepted_provider_contract_tests.rs: 643636776b009337fb80dfdf3d5dbee975e06ed7f7a7cf3f534c07301b61c729

Complete conservative closure grows107 to108 paths: one new test module,
one registration-only changed path,106 unchanged, none removed. Portable
Cargo pin10a7b438 is retained from independently reviewed PR102770e26.
Selectors, old inventories, kernels, oracle and qualifier are unchanged. This
authorizes fresh clean composed qualification, not transfer of older receipts.



### Lead-owned cold and accepted-runtime consumer base

This union retains PR10796dd2c3 and exact portable/cross-contract PR1049d2011e
ancestry. Complete conservative closure is110 paths: two added accepted CPU
contract modules, two changed CPU/test sources and106 unchanged relative to
the cold108-path closure. Native run-set registration is retained, not replaced
by the older native-only consumer parent. Exact admitted pins above retain
independent CPU and cross-contract source reviews; all source hashes match.
No selector, historical inventory, numerical gate, oracle or qualifier changes.
Fresh combined qualification remains required; component receipts retain their
original subjects. This isolated branch is a lead-owned integration base for
disjoint cold/comparison consumer acceptance, not full production UI readiness.



### Explicit RCP3/RCT1 consumer-base source admission

Exact independently reviewed PR1085c8326a is retained on the preceding
lead-owned PR10796dd2c3 + PR1049d2011e ancestry union. The committed review
at ../plans/configurable-solver-components/reports/configured-runtime-recording-review.md
approves optional explicitly versioned runtime observations and typed terminal
custody, unchanged RCP2 absence, and the five-path regression/document scope.
Actual operational metadata in that core fixture is authored, not actual IPC.

- src/recording/provider.rs: c4d48af7de2769349292819c48494f5a248edfb53eb58c9d53a535c165f5b427
- src/recording/provider/runtime.rs: d7b2a1f7cfa866b33887f7c8f804999058d6e70a1c9d71421372d151152ebccb

The complete selector grows110 to111 paths, one added runtime module, one
changed provider codec,109 unchanged, none removed. Existing selector, prior
inventories, numerical kernels, oracle and qualification runner stay unchanged.
Only exact reviewed hashes are admitted. This base unblocks disjoint configured
cold/comparison consumer contracts; it is not source/GUI acceptance of the
separately developed production desktop caller. Fresh committed union checks
remain required and component receipts retain their original subjects.


### Production Worker retains the published consumer dependency union

The lead-owned be0ed8e consumer base is retained by ancestry, including exact
PR10796dd2c3, PR1049d2011e and PR1085c8326a. All120 reviewed production
source hashes remain unchanged; only append-only review/ledger histories are
composed. No older source inventory replaces the reviewed desktop closure.
Fresh committed production qualification follows; this ancestry merge does
not transfer component receipts or accept pending configured comparison/window
work or the separately owned PR110 terminal-clock extension.


### Configured shared comparison library on actual production Worker union

Exact independently reviewed PR111ea6e5da is retained by ancestry. Source
review in ../plans/configurable-solver-components/reports/configured-recording-comparison-review.md
approves explicit typed configured eligibility, owner-supplied intervals, bounded
selection facts and pressure-call duration summaries beside the unchanged
shared field metrics/alignment/sampling engine. Exact reviewed source pins:

- src/recording/comparison.rs: 5a82201f4583cf9eb4cc38d43c7ee88bc54d10a84df46272c2dd84091aa4a307
- src/recording/provider.rs: 980ea5af7094016b13e7d0f39110aa2ff673b0173416e0fe583e6266b7c27f57
- src/recording/provider/comparison.rs: 90c1230c2cbfb2415c74e5d3c9b66e74f68c7b2096bf4063c5513ac2c6feaa75

Complete conservative closure grows120 to121: one added module, two changed
codec/comparison sources,118 unchanged, none removed. Existing selector,
historical inventories, numerical gates, oracle and qualification runner are
unchanged. Runtime metadata in the separate library fixtures is authored,
not actual IPC; actual bridge mapping belongs to the reviewed root Worker.
Fresh clean union qualification and existing desktop quantitative consumer
integration/native-window acceptance remain required. This merge also retains
current main8bf7a3e and separately reviewed CI11250732f5 setup ancestry; neither
changes these121 selected source pins or transfers prior whole-Git receipts.


### Shared saved native/configured source adapter

Exact independently reviewed PR1141c32eb0 source is retained by ancestry.
Review: ../plans/configurable-solver-components/reports/saved-source-review.md.
One new module and one bounded parent registration extend121 to122 selected
paths,120 unchanged. Native inner serialization/digest stays byte-identical;
existing typed manifest/archive owners retain validation and readiness.
Consumers must separately bind their envelope version to the source variant.
Latest reviewed cold terminal-clock policy, quantitative page wiring and fresh
final-source/window validation remain required. Exact source pins:

- src/bin/desktop/run_sets/source.rs: c948c24fbe3d0bbb4d281c427244f7908a16506236c0f24b194490c64033cd11
- src/bin/desktop/run_sets.rs: e194a48350954edf13cdcd01371d320cc210069b4256ba955f2ddd1132301cc1
- tests/saved_source_contract.rs: 39cb5152583c1662f8dfb71217409025084c4e05be1a18e1b7b3912fa4d21ca5


### Final reviewed frozen runtime custody and clock guard

PR110c230cf7 is retained by ancestry. The superseding independent review in
../plans/configurable-solver-components/reports/configured-runtime-cold-final-review.md
resolves the impossible-clock finding using repeated IEEE requested-duration
addition from the frozen +0 constructor, independent of observation presence.
Known RCT1 full audit, optional legacy custody, frozen package/phase/prefix/dt/
coefficient bounds and honest unavailable tail coverage retain their owners.
Complete committed-index closure stays122: one changed reviewed archive source,
121 unchanged. Root caller working-tree mutations remain separate and unadmitted
until their own final review; this merge does not silently refresh those pins.
Fresh composed qualification remains required. Exact final three-path pins:

- src/bin/desktop/run_sets/configured/archive.rs: eb1d8e77019eb57afd9e5fd5f57af55f8ff320527ec8efc935945bd7ff477806
- tests/configured_archive_binding_contract.rs: 821ddd7d173c84453650d2cde938bf024d907e2f062a894ea2b182e44ac56932
- docs/api/configured-session-intent.md: 0b3debf93ebacca8e45df814fd67694eac52f5bcac95a89c193b392b2e77b71d


### Existing saved comparison consumes configured Session sources

Independent GPT-6.1 Sol review in
../plans/configurable-solver-components/reports/configured-comparison-consumer-final-review.md
approves exact eleven-path caller/document/test scope. Existing SavedRecipe,
verified/unverified results, page, physical-time alignment, gauges, metrics,
linked fields/probes and Inspector exports retain their owners. Native inner
bytes/digests remain unchanged; recipe/result envelopes bind schema1 to native
and schema2 to configured sources. Configured typed facts, RHS hash relation
and sampled pressure-call summaries reuse PR111. Paint caches metadata and
never resolves sources. Latest reviewed PR110c230 custody is retained.

Exact independently reviewed Rust pins:

- src/bin/desktop/comparison.rs: 724ccf90e49bf3cbd7fa144fcee6acc91d72301651b99f2ed2f26d08c664ff5f
- src/bin/desktop/comparison_archive.rs: 813cc86187ef7313deacbe9d9cd15340f37ecdf62f5234a45f033909415459b9
- src/bin/desktop/comparison_field_view.rs: ec8ea0799522d5a7fadac47b06b42f26717f935bbb9a1677fc5052f096c1cc82
- src/bin/desktop/configured_comparison_tests.rs: e6716e8c707f30cc20304b16e3201f8fb1fd838650bd08a43a95ab4d68cbbf46
- src/bin/desktop/configured_worker_tests.rs: 8e6fd169186fe91b5e5eb6105716f29b56cc22e049d668de502def129607a51c
- src/bin/desktop/run_set_view.rs: ab2c841612f39b16f5bd1a363593eae5388bf4a81ec495dfb89e43bd7436580b
- src/bin/rheon-desktop.rs: 13cf1812ef2988cbd6484a830cb3ff5081912f9fee1508042d08ae15e244267c

Complete conservative closure grows122 to123: one added actual-Worker
comparison test module, six changed caller/test-helper sources,116 unchanged,
none removed. Only reviewed hashes are admitted; no source selector, historical
inventory, numerical gate, field arithmetic or qualifier change. Old native
contracts retain every behavioral assertion, with explicit source-variant
access in their fixture setup. Borrow-only lint repair suppresses nothing.

Five actual production comparison contracts and63 affected contracts passed
on the reviewed working source. They include actual producer-OS-exit and a
fresh reader with no model runtime on PATH; analytic fixtures are not learned
accuracy/performance evidence. Final strict lint, committed whole-source and
native-window evidence remain separately subject-bound. A separately allocated
actual after-accept recording-limit/unrecorded-tail fixture is still pending.


### Actual storage tail and truthful optional comparison custody

Independent GPT-6.1 Sol reviews approve the exact PR116766c62c producer-exit
fixture/three-line test registration and the bounded shared-summary correction:
../plans/configurable-solver-components/reports/configured-storage-producer-exit-review.md
and ../plans/configurable-solver-components/reports/configured-tail-summary-review.md.

PR116 uses existing production App/Worker/Session/model/SQLite owners. Actual
max_records2 leaves accepted generation2 beyond durable generation1/count2,
with typed StorageFailure and actual CandidateReturned observation; missing
pressure verdict, fields and preview remain unavailable. Producer OS exit
precedes a fresh environment-cleared/PATH-empty cold reader after package
removal. Full and correction cases bind all four immutable files and selected
publication hashes. Retention owns only an exclusive new task-prefixed /tmp child.

The shared page retains unknown accepted generation as unavailable, and does
not label known generation0/zero publications as an accepted tail. Existing
positive/absent known tails remain unchanged. Real owner-supported empty Failed
legacy finish(None) and actual Worker constructor-cap regressions exercise
these distinctions without database mutation or relaxed recording validation.
The three-path scope includes tests/saved_comparison_ui_contract.rs, explicitly
reviewed SHA256 f3bf401a00ccb9ba67de70ff72c6c3a0b7216525073a6f6c8239ec022e0bec1f.

Exact independently approved selected-source pins:

- src/bin/desktop/comparison.rs: ef52e29f4d9c59007c11ff973ed6a43c33ac9e1d583819ede11ebd7aea9ec0a5
- src/bin/desktop/configured_producer_exit_tests.rs: 7560ff5395acbd50ce111b1e50ae5e8aca1b1bab5356ec195929eec0994f24bb
- src/bin/desktop/configured_worker_tests.rs: 5ba01091e267e2ce51e61626807faf773426ec6243d586007c698bc2388c88ff
- src/bin/rheon-desktop.rs: 9488869f732dc43ce163002b8b24a9f6fb316ac3ded950d27b8afce97e565cfe

Complete conservative closure grows123 to124: one added test module,
three changed selected callers/test sources and120 unchanged; none removed.
No selector, historical inventory, numerical acceptance gate, codec, metric
engine, field arithmetic, worker loop or model supervisor changes. Source
admission only permits fresh clean committed qualification. Scoped actual
validation passed saved page9, constructor cap1, storage producer-exit1 and
configured comparison5, strict both-bin/all-tests desktop Clippy, fmt and diff.
One storage entry includes two cases and nested OS harnesses, not extra unique
tests. Prior rejected fixture variants were corrected without changing owners.

Actual bc6 saved/live software-window receipts and their independent reviews
retain bc6, while final Session ec46127 and producer6291ce have separate fresh
qualification. None is transferred to this later whole-Git consumer subject.
Final committed-source qualification, builds/stamps, final consumer controls
and all applicable hosted/main gates remain separate acceptance obligations.
The measured aa82 package remains unavailable; no learned accuracy/performance,
restart checkpoint, new physics, descendant custody or hard shutdown claim.


### Independent domain density and preview controls, 2026-10-16

Independent GPT-6.1 Sol source review in
[the density plan](../plans/domain-density/review.md) approves the revised
1000 cells/m, 10000 native-axis and 10000 independent-preview-axis settings.
Its [approved pins](../plans/domain-density/approved-pins.json) bind the exact
14 selected rebindings and complete 124-path closure: 110 unchanged, none
added or removed against main ad888bdbbbd5cae362439458af342196f51c58e2.
The selector, historical v1/v2 inventories, Cargo/lock/toolchain, numerical
kernels and scientific gates retain their bytes. This is explicit reviewed
admission, not automatic hash refresh or transfer of earlier receipts.

The changes reuse GridGeometry/BufferPlan, the existing App/Worker, numerical
and particle owners, recording and configured Session/PressureProvider.
Density/count/extent and exact/adaptive preview have separate contracts.
New actual physical sampling and explicit preview requests check the 1000/m
cap; historical frozen geometry remains exact. Checked history includes all
tracks and constructor frames. Conservative appearance resampling leaves
fields untouched. Renderer scan/face limits and fallible allocation explicitly
refuse unsupported display work without reducing stored/numerical resolution.
A separate thumbnail pixel budget removes the old late 128-axis failure.

Comparison density does not widen model support. Legacy v2 refuses higher
than 128/m inertly; dynamic declarations still validate actual support.
The unit-box qualifier rejects incompatible explicit geometry before residual
conversion. An independently found ordinary-count reciprocal-roundoff issue
was fixed while preserving original unit-box sources; the actual size49
comparison regression covers it.

The review is source-only. Root's fresh clean committed qualification and
exact-head hosted gates remain required. Continuously forced native CLI,
zero-flow archive custody and the source-stop IncompatibleRhs rejection have
separate claims; none supplies learned accuracy or new physics qualification.
The measured A1 owner-input blocker is unchanged. No training, replacement
checkpoint, Tuldok/surrogate-lab mutation or tolerance relaxation is admitted.


## Density comparison preflight correction, 2026-10-16

Independent GPT-6.1 Sol [follow-up review](../plans/domain-density/preflight-followup.md)
SHA256 `ff9648b5a8d79f2920ee06fa0be644598c5aefc10ef39306d94022fc0fde72bc` approves the exact
[supplemental pins](../plans/domain-density/preflight-approved-pins.json), SHA256
`4b36340a69d1a02b7a22fd74df121c83cd34eb3b5f6452f276cc718e3d50c6ee`.
The original source review and inventories remain immutable. Explicitly rebind
only the following two selected paths from the initial density admission:

- `src/bin/desktop/ai_comparison/engine.rs`: `610ba29f66eb91083f9bbd26b823b14e4bde4887eb057e929c089d6cd4592e55`
- `src/bin/rheon-ai-compare.rs`: `00000ec93161c04d9ff0515e5ae2ab6855a5467f56f23a4d6fc6c1059ef8ada9`

The same 124-path closure now has 15 changed and 109 unchanged inputs versus
ad888bdbbbd5cae362439458af342196f51c58e2, with no added or removed path.
The engine keeps its existing 3000-iteration pressure policy in a shared
constant. Preflight counts independent post-warmup native witnesses and names
its core Simulation array plan as a reference, not actual Engine memory.
No numerical kernel, tolerance, recorder/model contract, selector, qualification
gate, historical inventory or A1 status changes. Initial 94062 execution receipts
retain that historical subject; fresh clean committed qualification, affected
checks and current-head hosted gates are required before integration.

## Staged workflow and composition admission, 2026-10-20

This explicit reviewed refresh admits a fixed staged source only to attempt fresh
qualification. The source checkpoint is `f995923b84f0e963806654af292bd18073a7dcc9`,
tree `42e565eb670c1a4d4294fb8043dec287da5ba26e`. Its isolated trial composition
retains PR119–131 through PR130 `c2fbbea9f19f818b28b6c6863a40220a2bd0bbe5` and
PR131 `97cdce614c3c4438fa65ac806b48c4de6490583e`; the latter retains tested
workflow repair `34e2e73bd8f25f2c7134ad1f9764fc48733e5ea2`. No public main or
component branch is advanced by this trial composition.

Independent GPT-6.1 Sol review covered the complete unchanged conservative
selection policy: **135 regular files, 11 added, 16 changed, 108 unchanged, none
removed** versus the 124-path admission at main
`bc2f472787060d9a69cb619ed2c2fea10fd754b6`. All 124 historical identities were
verified against that exact main; all 108 reused identities remain exact.
The [per-path source review](../plans/fluid-accuracy-composition/reports/native-admission-20261020/provisional-readiness.md)
and [receipt](../plans/fluid-accuracy-composition/reports/native-admission-20261020/provisional-readiness.json)
retain every path's disposition. The provisional review subject `6137e46` remains
distinct from the staged checkpoint. The only subsequent selected source delta is
the independently approved [two-file iteration repair](../plans/fluid-accuracy-composition/reports/native-admission-20261020/iteration-repair-34e2e73-review.md):
`src/workflow.rs` at SHA256
`5d902d075f784c9c062d1d5372873b76844d1f8afb06218bff5c619c9d006eb9` and
`src/recording/workflow.rs` at SHA256
`caa2f224796f8ad6e92fb78d547c70df7dd88fe666843a715e90bd9257a36d34`.
All other 133 selected identities remain as reviewed. The exact final sorted,
compact JSON source-map SHA256 is
`9af9e8fceeb49febc428301ca51f0c428bfc25e38203d6eccd23fdece031e8e8`
(no trailing newline). Later PR127/130/131 evidence-only commits did not change
this selected map.

The admitted additions observe existing configured Session execution, add bounded
RCP4/RCT2 workflow evidence under the existing SQLite owner and present cached
workflow facts through the existing desktop worker. C1-A borrows existing native
family/pressure owners. C1-B1 validates supplied immutable pressure and velocity
arrays in disposable scratch; it does not invoke a candidate or publish Session
state. Q1 adds the explicit periodic represented-volume policy to its existing
owner, without admitting periodic pressure or whole-Stokes semantics. No second
numerical loop, recorder, runtime process supervisor or archive owner is added.
The iteration repair bounds native counts and requires candidate counts zero,
including later cancelled/failed attempts, while preserving cap-zero candidate
acceptance, rejected IEEE diagnostics and legacy RCP2/RCP3/RCT1 semantics.

The [mechanism review](../plans/fluid-accuracy-composition/reports/native-admission-20261020/review.md)
confirms that the established explicit v3 refresh suffices. Preserve all preceding
review text and source-bound receipts as history, the v3 schema/base, frozen v1/v2
bytes and digests, immutable primary tree, ancestry, selector, checker, oracle,
qualifier and packet refusal behavior. The active v3 map and this appended review's
SHA256 are committed together. Old receipts do not qualify the new map. This is an
explicit admission decision following complete review, never a build-time hash
refresh or a projection to an assumed numerical subset.

PR130 replaces all 15 optimization-sensitive assertions in four evidence
evaluators with always-on guards. Frozen evaluator bytes, campaign manifests,
results, failures and archive identities remain historical. Those tools are outside
the selected native map; independent full-AST reversal verifies unchanged
predicates/formulas, and the owner's separate tooling evidence reports 129 checks.
P1 remains stopped/unselected. Q1's incomplete transport/geometry/control evidence
and Q2's five positive native apertures on exact-zero-measure faces remain
authoritative. **The ongoing Q2 boundary repair is excluded from this fixed
checkpoint.** Its future exact source requires separate review and fresh applicable
qualification; this admission supplies no topology/DOF/B/M/S/W, coupled PDE,
whole-family replacement, learned-model or physical accuracy acceptance.

Require clean committed-source native/rational execution into a fresh external
directory with the unchanged qualifier, then a newly generated real native packet
and all packet/refusal tests normally and under `-O`. Do not use `--allow-dirty`,
transfer old receipts, suppress failed attempts or skip the stale-inventory gate.
The native qualifier does not execute every new workflow/native-family/composition
unit suite; retain fresh focused composed contracts separately. Hosted
Pages/browser/publication, desktop/native-window, performance and physical gates
remain separately scoped. Put execution results in a separate integration report;
do not append them here or change the inventory after qualification.


## Separate closed C1-B2 Session/custody admission, 2026-10-20

This explicit independently reviewed refresh authorizes a fresh qualification
attempt on final PR137 `d3e62377acde6641c8133cf9a9356cc046cfd02a`, tree
`f8be6e0ef85a6a965545b2882bff11500c7f1dfd`, retaining fixed PR133/8bf542e.
The [original source/contract review](../plans/fluid-accuracy-composition/reports/c1b2-admission-20261020/source-review/readiness-review.md),
[three-pin repair follow-up](../plans/fluid-accuracy-composition/reports/c1b2-admission-20261020/source-review/g1-cancel-followup.md)
and [original 140-path receipt](../plans/fluid-accuracy-composition/reports/c1b2-admission-20261020/source-review/readiness-review.json)
approve five added, eleven changed, 124 exact reused, zero removed regular inputs
under the unchanged complete selector. All 140 working/committed identities
match the final follow-up map; 137 identities are exact PR136 and its three
changed pins are independently reviewed. Canonical sorted compact source-map
SHA256 (without trailing newline) is
`08aa3d1b48617763da61291e788fc985bc5d011d8039f9ebf83fbc4ccc2a05a6`.
Cargo/toolchain/qualification inputs remain exactly fixed PR133; the separate
Q2 Cargo/reference/CV1 stage is excluded. Historical 9c/9a receipts remain fixed
and do not qualify this new source closure.

The new closed immutable BuiltinNativeV1 path reuses existing native producer,
B1 pressure/velocity qualifier, Liquid gates and atomic Session commit. Capped
scratch and typed immutable attempts add no accepted-state owner, scheduler,
process or store. RPS1/RPP1/RPT1 reuse SQLite and unchanged native RSC1/RCP2
payload validation, complete 16KiB caps, exact binding/prior-link checks and
explicit accepted-unrecorded tails. Mixed pressure-witness fallback refuses
publication without stripping observations. Existing capture/desktop/comparison
paths explicitly refuse the new profile. Optional viscosity, arbitrary/stateful
external providers, full C1-B/C/R, whole-family/transfer and physical acceptance
remain excluded. Component core634404e/consumera6e76a9 evidence retains its exact
three-UI-literal source split; final e56bc7b adds only reporting documents.
PR137's tested source70da36d and final report d3e6237 remain separately attributed.

PR137 repairs the legacy configured-Session advection cancellation label using
three matcher lines. Tests retain actual velocity/tracer cancellation before/after
acceptance, cold RCT2 custody, exact state/receipt, retry, historical Failed labels
and a true numerical failure control. No numerical formula, accepted order,
publication point, codec/version or UI implementation changes. The old defect
remains historical in fixed 9c/9a; no old archive is relabelled. Owner results stay
at tested70da36d, not this later lead execution. Fresh qualification is still
required; admission is not complete behavioral coverage of all 140 modules.

The existing v3 mechanism is sufficient. Preserve all prior review bytes,
schema/base, exact v1/v2 predecessors, immutable primary tree/ancestry and all
selector/checker/oracle/qualifier/packet refusal behavior. Commit the explicit
reviewed map and this appended review digest together, freeze them, then follow
the [prospective protocol](../plans/fluid-accuracy-composition/reports/c1b2-admission-20261020/protocol.md):
new clean committed-source native/rational execution, newly generated actual
packet with 26 normal and 26 optimized checks/no skips, plus separately recorded
B2/legacy/persistence/volume/borrow contracts. No stale receipt, allow-dirty,
build-time refresh or skipped gate. Retain failures and outcomes separately;
never edit these bound bytes after qualification. Hosted/default/desktop/GUI,
release/performance, model/Lean/GPU and physical gates remain separate. No merge
authority is supplied by admission; public merge actions remain paused.


## Separate closed C1-B3 producer-selection admission, 2026-10-20

This explicit refresh permits a fresh qualification attempt after independent
complete-source review of final PR140
`5dc1741feffa88b857194559b3e7e79be5369a64`, tested core4af3802 and consumer9ddf8d2.
The [independent source review](../plans/fluid-accuracy-composition/reports/c1b3-admission-20261020/source-review/readiness-review.md)
and [all-path receipt](../plans/fluid-accuracy-composition/reports/c1b3-admission-20261020/source-review/readiness-review.json)
approve the same 140 regular selected inputs: eight changed, 132 exact reused,
zero added/removed versus both PR137 and qualified186dc0a. Two additional changed
public contract targets are reviewed outside the unchanged selector. Canonical
sorted compact map SHA256 (no trailing newline):
`6cf9712dbebbec79309d9ead4b6514d5cec3cf7559452d0ff16824735d49d22d`.
The local composition retains exact owner and qualified186 histories; no old
execution receipt transfers. Cargo/toolchain and all 132 reused inputs stay exact;
Q2 Cargo/CV sources remain excluded. Existing v3 schema/base, v1/v2 identities,
primary ancestry/tree and selector/checker/oracle/qualifier/packet stay unchanged.
Every previous bound-review byte is preserved as an exact prefix.

ClassicalSelectionV1 selects the other existing native pressure method with
positive producer cap bounded by the constructor cap. Among producer errors,
only actual IterationLimit permits same-immutable-input full-cap native fallback;
other errors and cancellation terminate unpublished. Existing B1 field/momentum
rejection fallback remains; mixed-pressure witness fallback stays unpublished. Existing qualifier/Liquid/Session atomic
publication remains sole owner. Additive RPS2/RPP2/RPT2 binds distinct producer,
fallback and actual selection roles, with strict outer intent and embedded attempt
version refusal before payload parsing. Private method-only header adaptation
occurs only after outer validation; legacy configured/version-1 validators remain
strict. Graph/comparison refuses the new namespace. No stateful/learned/external,
whole-family/transfer or full C1-R admission is implied. Original failures/repairs
and owner227/5 execution claims remain their original subjects; no raw owner
archive audit is asserted.

Commit this append, its bound digest, complete reviewed inventory and
[prospective protocol](../plans/fluid-accuracy-composition/reports/c1b3-admission-20261020/protocol.md)
together after final independent metadata/protocol review, then freeze them.
Execute new clean committed-source native/rational qualification, actual exporter
and packet with26 normal+26 optimized checks/no skips, plus complete no-default
library, all B2/B3 public, legacy recording/provider, volume and borrow contracts.
Retain actual ELFs/logs/source hashes and failures; report outcomes in separate
files. Admission is an attempt permission, not acceptance from refreshed hashes.
Hosted/default/desktop/window/release/performance/physical/model/Lean/GPU and
combined-Q2 qualification remain separate. Public merges remain paused.


## Shared authored force fields and accepted recording custody, 2026-10-20

This explicit source-admission extension covers the backend force-domain branch
at `43ec505218c43594641ecbb80873bec8c88d7d2c`, based on main
`fd7c0f0aee63bc8710df99f9bb01154e7619b974`. The admission commit changes only
this appended record and its version-three inventory binding; native source
bytes remain those of the reviewed subject. The complete conservative closure
grows from 124 to 126 paths: two added, eight changed, 116 unchanged, none
removed. Every `src/` path, build/toolchain input and existing independent
qualification input remains selected, including unchanged desktop sources.

The bounded source scope is:

- `src/force_field.rs` adds version-one authoring values, stable object IDs,
  rigid transforms, enabled state, static half-open activation intervals,
  directional/radial/vortex fields and global/sphere/box support with optional
  smooth edge attenuation. Validated immutable snapshots share one evaluator.
  Acceleration is in m/s²; force density is in N/m³ and requires positive
  density in kg/m³. Finite-input, zero-distance and bounded-object checks
  refuse unsupported or nonfinite evaluation.
- `src/forces.rs` shares the existing MAC face-update loop with authored
  acceleration evaluation. Closed-box walls retain their existing treatment.
  `src/simulation.rs` adds explicit force-aware fixed-box stepping and
  success-stamped pressure/actual-force custody. The legacy pressure accessor
  cannot expose incomplete force controls; candidate attempts invalidate the
  scratch pressure stamp before use. Pressure projection itself is unchanged.
- `src/particle_fluid.rs` adds the same evaluator to gravity and pair forces
  at pre-integration particle positions, using the evaluated local particle
  density for force-density conversion. Accepted force controls are stamped
  only on success; rejected attempts preserve the existing accepted particle
  publication. Source emitters, constitutive stress and turbulence closures
  remain distinct owners.
- `src/recording/force_fields.rs` adds a bounded version-one binary metadata
  codec with exact float bits and scene validation. Force-aware run identity
  binds the scene digest; append and read check run/publication agreement.
  `src/recording.rs`, `recording/carrier.rs` and `recording/material.rs`
  retain actual accepted force inputs and decode them for selected archives.
  Historical unforced hash/control paths remain separate and unchanged.
- `Cargo.toml` adds an opt-in `force-serde` feature using existing dependency
  identities; `Cargo.lock` is unchanged. `src/lib.rs` adds public exports.
  No desktop/request/worker source is changed by this backend branch.

Exact added/changed source SHA256 pins:

- `Cargo.toml`: `f925dae7a0d74e6024e6e0d64962a275ae4f28fedec5b016341671021b8239ae`
- `src/force_field.rs`: `adecbc5f92b89179a2e38885be2acbcddd42df40061f00ffa5d758224ed045a5`
- `src/forces.rs`: `7b3b48c0813c995a937a44ca1b624411c59dde6419b62041b67456cde2d692d9`
- `src/lib.rs`: `30f5b78a98f6d0c010955a25fc00cc2de4aadf159d260e95afd3f3143dc6f9c3`
- `src/particle_fluid.rs`: `538c498122bbc1045ae59d7363ff87e1dd9a40003fb48eeae835228ad4c65893`
- `src/recording.rs`: `47b2b24bbe9af777ceafdefa6f0b82200a60cea590d0b64f326de1d50c6d7d2d`
- `src/recording/carrier.rs`: `5a9b10b929f650423ad9e58bd7047bfe80cc7bf18138d74bd628c2f3f17af711`
- `src/recording/force_fields.rs`: `2cf3ec9ec191c35c814d48ace301feaccd6cc73311a66970c9d112dad432c4a9`
- `src/recording/material.rs`: `e72e256aa3af46fef9002822de5281caf1de66ba070eb2109f77b083e20ae6ce`
- `src/simulation.rs`: `c08ede6330c4bf6ed7e858e556806ec80628522ceb42286cf52714bf7430c9a1`

This admits an attempt at fresh clean committed-source native/rational
qualification only. It is not a numerical receipt, scientific validation of the
new force laws, native-window evidence or completed desktop parity. Pure
evaluator/codec checks and the seven bounded adapter/recording tests have their
own pinned subjects under `../plans/force-authoring/`; they do not substitute
for the fresh independent qualification required here. Later desktop/runtime
composition must re-establish complete admission and fresh evidence for its
actual committed source.

The historical v1/v2 inventories and all preceding review text/receipts retain
their original bytes and subjects. The v3 ancestry/schema and predecessor
digests remain authoritative. No source selector, numerical threshold,
qualification runner, mathematical oracle, workflow or immutable primary
identity is changed. A stale or malformed present inventory must still refuse
admission rather than fall back. Fresh receipt binding must include the actual
committed manifest and this complete review record; earlier standalone CI
success or historical receipts cannot qualify this admission commit.

Independent review of this exact two-file candidate is required before atomic
commit and publication. This record contains no claim that an unseen patch was
previously approved. Native/rational CI uses its existing 20-minute job bound.
The separate research-book job performs admission preflight and real packet
rejection tests normally and under optimized Python within its existing
90-minute job bound; readiness requires those actual current-head outcomes,
not an assumption that the standalone qualifier includes packet tests.


## Scene authoring and gated force runtime integration, 2026-10-21

This explicit extension admits the complete UI source at
`da5c68891a7c627007ccb7136c9c9cd6b42fee26`, tree
`281834746edcb4c84e1d33a9b0eddb9578fda3b0`, for an attempt at fresh
committed-source qualification. Its complete selected closure is compared
against merged backend `7516a8e5d328c6dc8eda8ea577cb03f20b2e343a`, whose
126-path inventory includes the preceding shared-force admission. The UI
closure has 136 paths: ten added, ten changed, 116 unchanged, none removed.
The existing selector still includes every `src/` file, including desktop
tests and README files, plus build/toolchain and qualification inputs.

The complete merged backend review above is preserved byte for byte. The UI
branch's older 124-path inventory and shorter review do not supersede the
merged 126-path backend admission. This v3 rebind preserves all 116 unchanged
backend pins and explicitly adds or replaces only the twenty UI paths listed
below. Historical v1/v2 records, predecessor digests, reviewed ancestry and
immutable primary identity remain unchanged.

The reviewed UI scope is:

- `scene_ui.rs` owns canonical authored force objects, shared-domain IDs,
  validated properties and immutable snapshot freezing. Draft and Retained
  selection/property contexts remain distinct; retained values are read-only.
  `scene_view.rs` and `scene_handles.rs` add viewport selection, thin selected
  outlines and Move/Rotate/Size previews. Handle preview data is temporary;
  validated release applies through the existing transaction, while cancel
  leaves canonical draft data unchanged. These UI controls do not define a
  second force evaluator or alter physical units.
- `rheon-desktop.rs`, `carrier_execution.rs`, `comparison.rs`, `run_sets.rs`
  and `run_sets/source.rs` carry immutable track-specific snapshots through
  native-carrier and particle-material dispatch and retained presentation.
  Force-aware execution uses the canonical backend adapters and accepted
  controls. Saved native intents use version two for authored metadata while
  preserving the legacy no-force version-one bytes/hash path. This does not
  add a serialized particle-material cold-rerun API.
- `inspector.rs` binds and verifies the force-scene digest using canonical
  recording metadata before publication/readback. Historical no-force
  branches retain their existing controls. `settings.rs` supplies precise
  numeric editing; `configured_controls.rs` and `viewport.rs` integrate the
  compact timeline, Scene panel and selection/focus ownership.
- `force_run_policy.rs` distinguishes eligible native-carrier and
  particle-material targets from unsupported configured/session/provider/
  free-surface/AI paths. Production qualification remains `Pending` and
  authored production launches remain refused. Test qualification cannot
  grant capability to an unsupported execution owner. The associated policy,
  runtime, setup and timeline test modules are selected source inputs, not
  evidence that all runtime or native acceptance is complete.
- `native_presentation_harness.rs` is a test-configuration presentation
  harness. Its presence in the conservative closure does not establish native
  execution results. Backend force equations/evaluator, MAC/particle adapters,
  accepted recording modules and numerical/pressure policies are unchanged
  from merged `7516a8e5`.

The final two repairs were independently inspected as actual source diffs.
`37ac711a4b7134f628c7a71d54f62056d71ca5f3` moves unchanged Inspector custody
helpers and the test-configuration entry point before their test modules,
without lint suppression. `da5c6889` gives normal Draft property editing and
disabled previews one persistent per-object UI identity, preserving disclosure
state and numeric editor identity when invalid-input feedback adds rows. Its
two new regressions construct canonical metadata and egui contexts only; they
do not construct App/Worker or execute solvers, recorders or models.

The published source reviews and bounded execution subjects remain under
`../plans/ui-refinement/reports/`, particularly
`runtime-force-integration-source-review.md`,
`force-worker-runtime-results-review.md`, `typed-force-policy-source-review.md`,
`ci-item-order-source-review.md` and `scene-properties-stability-source-review.md`.
Their stated subjects and limits remain authoritative. Independent read-only
integration review checked the twenty added/changed UI paths, both exact repair
diffs, all transferred changed-file identities, full reconstructed trees and
the complete selected-source comparison against merged `7516a8e5`. This is a
source review; it adds no numerical, hardware or native-interaction result.

Exact added/changed source SHA256 pins relative to merged `7516a8e5`:

- `src/bin/desktop/carrier_execution.rs`: `c4455879373fc71eaaf398fdaa230c4d0b115dc7ff0a9da6356afeb24cba23b2`
- `src/bin/desktop/comparison.rs`: `27ad4b88db848430a68f5f9fae796fa10eab1df26af9dbae73386e55c9b5c74f`
- `src/bin/desktop/configured_controls.rs`: `5463e18b3b5a139c448dd327b23318605664e90aea4690187981177e7f68e60b`
- `src/bin/desktop/configured_worker_tests.rs`: `dabe1c3cf4214ee30ca42102c4c5387564b661f81b810d2c61035b8691669016`
- `src/bin/desktop/force_run_policy.rs`: `086d4ee8c18ff30f2165b8622a53d649bd9eb8166bbbbcb82dde3a0eb2427cc2`
- `src/bin/desktop/force_run_policy_tests.rs`: `a518b47b71cb2aff5d7ce46169918c60d534c00ffccdd0ee5c90abcebbdcdebd`
- `src/bin/desktop/force_runtime_integration_tests.rs`: `da753230b81df85227ab348c39ea4ac97e2ff093cf2da09ff9ac5dce225d0584`
- `src/bin/desktop/force_runtime_ui_tests.rs`: `1d242eaed1b416e3dfb2577d5610c4e0d6d0ca5040f80293681ae184090cebb9`
- `src/bin/desktop/inspector.rs`: `7a6582faf9bfff755d7fade624f6b11cca06813c75310fba25ccbfabff1d2aae`
- `src/bin/desktop/native_presentation_harness.rs`: `38b8b68ebc8a54769bcf8e65b2cf80890c03eac2251b4f838fac29ecbf63ecb6`
- `src/bin/desktop/run_sets.rs`: `b25f07c5d9f4a27572922956043224c35dde35d9a0d987ab8af4f3f3151c91f0`
- `src/bin/desktop/run_sets/source.rs`: `39e8982fffaf4e9adcd0ca0c60f49eabc96b829957bc6d9c05479c80eee99daf`
- `src/bin/desktop/scene_handles.rs`: `4818b85644a2891de562adc00fc203eb59c674bbf46aa88cd440bcab6477eb22`
- `src/bin/desktop/scene_ui.rs`: `46eeaeaf3cce71c7665cde4224ef2ed7e62100047796414c94d07e02dd4b7390`
- `src/bin/desktop/scene_view.rs`: `d16851804c3aa727c7fdf658a821ef8c8cd2c167e853b72d02604f2a14dfa02f`
- `src/bin/desktop/settings.rs`: `72a2e42de38100fa7cb517bfa7c1b9377975d23c3c4cd887615c296549c093c4`
- `src/bin/desktop/setup_ui_tests.rs`: `db3d1328aa159e79ea4f8a15cd59ebb6cb7c9352854760bd8164587014e35900`
- `src/bin/desktop/timeline_ui_tests.rs`: `606b0e95edba4a813221af776e7b12f776adcf9645de302ef827aa595111694b`
- `src/bin/desktop/viewport.rs`: `ee7acacd33ae29af7ec7f592361861315b30891b1637cb2989ba7390fa8320d3`
- `src/bin/rheon-desktop.rs`: `dfc472b78c46e83b45668a8d99bb82f721fcc771ddc584cdc8fdaf82272003ff`

This two-file admission commit changes no selected native source byte. It
admits fresh clean committed-source native/rational qualification only; it is
not a scientific receipt for authored fields, native repair acceptance,
production enablement or full voxel/particle desktop parity. Earlier bounded
worker/recording runs and local egui checks retain their own exact subjects.
The assigned native validator must separately verify the final repair's
specified window/scaling and interaction cases. A later enabled candidate
requires its own reviewed source, admission binding and separately bounded
actual App validation. Unsupported execution paths remain unsupported.

No selector, threshold, mathematical oracle, qualification runner, workflow or
historical receipt is changed. A stale present inventory must still refuse;
there is no fallback to older pins. Fresh receipts must bind this actual
committed complete inventory and review, rather than reuse backend or earlier
UI receipts. The existing dedicated native/rational workflow has a 20-minute
job bound; the separate research-book job performs admission preflight and
real packet rejection tests normally and under optimized Python within its
existing 90-minute bound. Both actual exact-head outcomes remain required.
The dedicated workflow is not automatically selected by admission-document
paths and must be dispatched explicitly for the clean committed branch.

Independent review of the actual complete two-file candidate is required
before atomic commit and publication. No prior review is represented as
approval of an unseen patch. PR147's separate research-book/FEM changes are
outside this source admission; current main and dependencies must be checked
again before any merge.


## Reviewed production-force enabling source candidate after qualified 3ce3f29

The preserved review prefix above is byte-identical to the complete reviewed UI
admission at3ce3f29cdec0613272064eb8df2f4fbad6ee0603: prefix SHA256
`3e190beb40264f575a54e36ee33160ae3da73e36fa7032c67702ab8a8f2ff48a`.
Its v3 manifest SHA256 is
`15e3f33639805229a22822137331e337f7e32b1461373563ecaa4d9ddc716300`.
That checkpoint's Rust39534833351, native/rational39534867735 and book39534833251
completed successfully; actual native/rational payload/source bindings and book
packet26normal+26optimized/no-packet-skips logs were independently reviewed. Its
PR deployment was skipped. Those receipts stay bound to3ce and are not receipts
for this new candidate. Historical6153/fe9 failures and cancellation remain evidence.

Parent assigned these desktop paths and admission files to the repository
integration owner while the original UI owner is idle. The actual eleven-file
source patch over3ce was independently reviewed as SHA256
`399db2ece70725d0e66eba4dce7abfeaf9f0d04bdfd710e2ba3009aa5c814f27`.
It removes every Request force_validation field, literal and cfg(test) branch.
Production and tests now use the same Validated policy for native-carrier and
particle-material execution. Configured/session/provider/free-surface/AI paths
still refuse authored presence, including empty/disabled/zero scenes. Unsupported
configured precedence, material-only single-track restriction, exact member force
binding and legacy None behavior remain. No solver equation, core adapter,
recording schema/hash, source emitter, material stress, closure or threshold changes.

Scene receives the effective target and derives its warning from the same policy;
the Run action emits intent only for eligible valid input. A live force edit must
be applied/cancelled before launch, avoiding a snapshot of its pre-edit draft.
The viewport's reachable stale runtime wording is corrected while alternate
lab/session restrictions remain. Other modified test/harness files only remove
obsolete Request fields. Existing historical native property acceptance is on da5;
this new source candidate still requires its own production App validation.

Strict desktop all-target Clippy and compile-only checking passed. The first new
Scene test initialization triggered field_reassign_with_default; that original
failure log was retained and the initializer corrected without suppressing lint.
A sealed current test binary SHA256
`6f07fa1e27b8dcd3f007b5722eba414ba6e1228e36d2d6dd8c30aa7645043ca5`
enumerated208tests. Ten independently reviewed exact pure tests each passed once,
serially, under30s wall/20sCPU/512MiBAS/1MiBfile+log, TERM then owned-child KILL
with1s grace, testthreads1 and no parallel feature pool. They cover four force
policy cases; mixed saved-track custody and recording-header preflight; supported/
refused Run intent without dispatch; Scene-only warning; disclosure stability;
and invalid-radius correction without refocus. No Worker, solver, SQLite, model,
native App, L1/MMS or extra scientific execution was performed locally.

The actual-App input/allocation and admission protocol is
`docs/plans/ui-refinement/reports/enabled-force-app-validation-contract.md`, SHA256
`80043585fd578cd625d5b2a24b4b66173fd852eed9643a50d82338a0908f0d10`.
It explicitly addresses real GUI carrier density1/192MiB/max2000, material's
minimum32768 allocation cap/default192MiB, recording1MiB/2records, exact small
finite workloads and separate rendering/process/temporary-disk overhead. It
requires native-owner existing GUI-compatible enforcement facts and an independent
actual execution-contract review before any App numerical execution. Managed
buffer arithmetic and prior Worker caps are not represented as process/RSS/disk
peak bounds. The native protocol's unresolved fields are not waived by this source
admission. No material serialized cold-rerun format or scientific parity is claimed.

The complete conservative native selection remains136paths:0added,11changed,
125unchanged,0removed relative to qualified3ce. Every unchanged selected path was
verified against its previous pin; only the eleven independently reviewed source
hashes below are rebound. The v1/v2 inventory files, bindings, historical reviews,
qualification tools, admission selectors, packet rejection tests and thresholds
remain unchanged. The new protocol is a documentation input, not a new selected
native source path.

| Reviewed changed path | Candidate SHA256 |
| --- | --- |
| `src/bin/desktop/configured_worker_tests.rs` | `06a9e7f5724b73c101a38dcc2c966d37b640900495643ea7bc04c13f4630f278` |
| `src/bin/desktop/force_run_policy.rs` | `cbb784c1cc403cfd524b715a52144c662f428b302014af1794bb3131d09bb97a` |
| `src/bin/desktop/force_run_policy_tests.rs` | `99d6b9e8b2a98eceab0514510d3f54873da50fdabc7626dd3896d45dd3c53383` |
| `src/bin/desktop/force_runtime_integration_tests.rs` | `04ba4f5883e48c5548cb420e17b6a6736436871d8e6c7792487939bfe5ed29a6` |
| `src/bin/desktop/force_runtime_ui_tests.rs` | `92475c2b577510a8f8fc11217f7eb1a5361dcaef20ade99f198f60d5b654966c` |
| `src/bin/desktop/native_presentation_harness.rs` | `8ce8efa63afbe0d09e22996ec00478de331fcbda7a5ac83882f2ed3fff0d8ea1` |
| `src/bin/desktop/scene_ui.rs` | `8ec81841a2792621205e58d7fdb80078ed75c94b823ddd6cb14976c0decda1a7` |
| `src/bin/desktop/setup_ui_tests.rs` | `278b7a20f5d48a4e8a6c8ee2a4b18f0377f09f78bd64b34cfb20d4545ebc2563` |
| `src/bin/desktop/timeline_ui_tests.rs` | `545e109f6d6808cc31482b8cffb4e21f8518d705424327728dc6c5d549a73df5` |
| `src/bin/desktop/viewport.rs` | `4a365fbb1a0f48bd016b3d673b30e635ad26390205cb22c24d25fa46738b6a08` |
| `src/bin/rheon-desktop.rs` | `d74cff5459f52d1cb01512b6300d9466411352f73b014491e3678d514844d1b3` |

The appended review, actual complete v3 manifest and execution-contract status
require independent review before atomic commit/publication; approval of the
source-only patch is not approval of unseen admission bytes or native execution.
Fresh native/rational qualification must use the clean committed candidate within
the existing20minute workflow. Existing hosted Rust and full book qualification,
including real packet normal/optimized rejection tests, must bind that actual
source. No receipt refresh, allow-dirty, gate skip, threshold relaxation, budget
expansion or PR merge is admitted by this source-identity record. FEMPR147 remains
outside this change; recheck main/dependencies before integration.

## Reviewed Pending-era Scene test assertion repair after 21e70bd

The entire preceding review is preserved byte-for-byte from
`21e70bdeb42f529b63ee3cab5a08331c09646c9f`: 92814 bytes, SHA256
`fe11734e336ae2527aa8f96cac81c284da3d50229317a2c1fb5a0c6b79e97510`. Its v3 manifest SHA256 remains
`3a2e70e515adc69ac3ce5b2713538bb0096c85505e8ac0f0142de0611fd7ba3b` in Git history.
The new repair does not reinterpret the fully qualified3ce checkpoint or21e's
fresh dedicated qualification receipts as receipts for the next source revision.

Actual21e Rust run39543779071 desktop job119037237201 failed its contract step:
`scene_ui::tests::scene_pointer_add_select_edit_collapse_and_retained_context`
still asserted NativeCarrier readiness was an error at scene_ui.rs741. The actual
checkout ccd3514e431f20aa7a53806ed90462c56f44b0c2 has the exact21e tree
cfadc610a1299a4e1fc84d68dba2ed3daeb4f7c6. Its desktop result was203passed,
1failed,4ignored; it stopped before integration targets. Strict lints had passed.
Original logs,21e receipts and the prior sealed local binary remain preserved.

The exact minimal source patch SHA256
`cbe01f17a79d9c9367c532778d4380865352dda214ce60aa4d4d068bf1ee7d90`
replaces this Pending-era expectation with exact NativeCarrier `Ok(())` and exact
Unsupported `Err(UNSUPPORTED.into())`. The valid scene contains three authored
objects after normal egui edits, with its Vortex disabled. Authored presence stays
unsupported for the unsupported owner; successful supported readiness now matches
the intentionally enabled shared production policy. This is a changed eligibility
contract assertion, not weakened numerical validation or acceptance of any error.

Only assertions inside `#[cfg(test)] mod tests` change. The production prefix of
scene_ui.rs before that module is byte-identical to21e, SHA256
`0e952e9d717d4b0ed9e6491d53ca334cf386d78d885fee33864aeade5a88d746`. No production control, solver, evaluator, recording, force-scene,
App input protocol, threshold, selector or qualification tool changes. The existing
actual-App contract remains SHA256
`80043585fd578cd625d5b2a24b4b66173fd852eed9643a50d82338a0908f0d10`.
Native process/graphics/SQLite/output/termination review and A/B/C execution remain
separate and pending; no numerical App work was duplicated here.

Independent review inspected the actual patch and complete pure helper closure:
SceneUi, canonical metadata and egui context/frame/click/key/replace only, widths
900/1250. No App/Worker, solver/evaluator, filesystem, recorder, model or subprocess
helper is reached. The newly compiled sealed test binary SHA256
`50b03c3df9af47e943c2f502f3d7835336d3d7c4786fda6daf4787cbd4bc2dfb`
enumerated208unique tests. The exact failing case passed once,1passed/207filtered,
exit0, no signals, under30s wall/20sCPU/512MiBAS/1MiBfile+log, TERM then
owned-child KILL after1s grace, testthreads1 and no parallel feature pool. Its log
SHA256 is `cfd66d3415f86438f655afefa87eb3c71540bb40926fbe78cb1eadfe8bc75a5f`.
Compile-only, strict desktop all-target Clippy and formatting passed. No broad local
tests, scientific L1/MMS or numerical/SQLite workloads were executed for this repair.

The complete conservative inventory remains136paths: only
`src/bin/desktop/scene_ui.rs` changes to SHA256
`e98c8aa0449b8f1adac4eb8a5bdc9a0951bd0b054f4dd2575d31dee30990c922`;
135unchanged pins were checked against actual files and the21e inventory; none
added or removed. Historical v1/v2 bindings and review bytes remain intact. The
actual complete three-file candidate requires independent review before atomic
commit/publication, then fresh clean committed-source existing native/rational
and normal Rust/book qualification. No stale receipt refresh, dirty-source bypass,
threshold relaxation, workflow expansion, merge or FEMPR147 changes are admitted.


## PR142 current-main composition and fixture admission, 2026-10-21

This explicit v3 refresh admits a fresh qualification attempt on PR142
`4f2d55323825c8907cf9209914f29130ec7de511` plus current main
`15026a966265cf9eaedf464e2b5a066bd0fef3d8` and its bounded cfg(test) fixture
repair. The unchanged conservative selector covers **152 regular inputs**.
Against PR142 inventory: 12 added, 19 changed,
121 unchanged, 0 removed; against main inventory:
16 added, 18 changed, 118 unchanged,
0 removed. The canonical sorted compact source-map SHA256
(no trailing newline) is `4d6fbe1927962b23da4c37e32b476b20a2ae7f48bf0d11805c36065c8a8b464d`.

The [independent complete-source review](../plans/fluid-accuracy-composition/reports/pr142-completion-20261021/source-review/review.md)
approves these exact identities: 94 match both parents, 24 match main, 27 match
PR142, seven compose both owners or repair the test fixture. Its
[per-path review](../plans/fluid-accuracy-composition/reports/pr142-completion-20261021/source-review/per-path-review.md)
and [machine map](../plans/fluid-accuracy-composition/reports/pr142-completion-20261021/source-review/source-map.json)
record every parent/current identity and disposition. Machine-map file SHA256
is `ae828885a18fbf5c986b73d65353197a85194f7496337080e88e6198e5558421`.

Simulation retains workflow capture, authored force fields and closed projection
inputs under their existing owner; all constructors initialize the complete
private domain. Lib/recording/desktop merges retain both histories and strict
publication/selection/profile semantics. A shared cfg(test) fixture now uses an
atomic sequence and exclusive directory creation so repeated clock values
cannot alias another fixture; an identical-timestamp concurrent regression is
added. No numerical formula, acceptance threshold, schema interpretation,
source selector, qualifier, oracle or packet consumer changes.

Both prior review histories above remain intact, including their common
72,250-byte prefix and complete divergent suffixes. Frozen v1/v2 bytes, primary
ancestry and every historical receipt remain. This new map cannot use either
parent's receipt as fresh qualification. The prior stale-review-binding refusal
is retained outside Git. Commit this explicit inventory/review and the
[prospective protocol](../plans/fluid-accuracy-composition/reports/pr142-completion-20261021/protocol.md)
before fresh clean native/rational/real-packet and ordinary supported-feature
contract execution. Do not rerun sealed numerical campaigns or weaken gates.

Fresh 2026-10-21 user instruction authorizes normal reviewed serial integration
through PR142 lead, superseding only the historical task's no-merge delegation.
Cloud-task reads/resumes and the earlier GPU artifact download remain denied.
No source branch is deleted or rewritten. Source admission is not numerical,
GUI-window, performance, hardware/model, physical or production acceptance.

## Dependent exact-geometry reference inputs, 2026-10-20

This separately reviewed checkpoint extends the fixed stage above only after
PR132/e6525a116a07af486e4a682cc2b157241d6b9150 and
PR134/f84f65104d6a0f6f8f1a9b903412b1e5155f72a2 are composed at
`cd21a9952dc411e01b77f58e3aae67aff1e473fd`, tree
`0447da683eecce1330adb563f1ca6e5dabfc0a62`. Tested component sources remain
`0f4e7e0944a7e0302690a0f71c08c21fe345cc89` and
`06b31902fd229624080bff1562f80beb1be2449c`. Fixed PR133/8bf542e and its actual
9c04278 qualification remain unchanged and do not qualify these additions.

The complete unchanged selector still contains **135 regular inputs: 0 added,
2 changed, 133 unchanged, 0 removed**. Every prior source identity except Cargo.toml
and Cargo.lock is reused exactly, including all Rust, toolchain, checker,
qualifier, oracle and qualification-test inputs. New Cargo.toml SHA256 is
`fc5bccf8799aa579c510d2ff7078a4532f794e61841e327d7d567257589f2409`; Cargo.lock
is `e64beca355b3963812b80d49ac5b17d0b1626f03f5d651b50e4dd25823dce96f`.
The full sorted compact source-map SHA256 is
`c1baa8b487bdacaa746446a0e21540b9605232c39131b32148253909249cb8f1`
(no trailing newline). No unrelated source is removed by comparing an incomplete
component-branch snapshot against the composed stage.

The [bounded source/dependency review](../plans/fluid-accuracy-composition/reports/q2-admission-20261020/source-review/readiness-review.md)
and [dependency receipt](../plans/fluid-accuracy-composition/reports/q2-admission-20261020/source-review/dependency-review.json)
pin new dev-only num-rational=0.4.2 and existing num-traits=0.2.19 requirements.
The lock adds num-bigint 0.4.8, num-integer 0.1.47 and num-rational 0.4.2, preserving
all prior registry identities/checksums. Actual official crate archives match
their lock SHA256s and contain MIT OR Apache-2.0 license texts. There is no new
root build.rs or build-dependency and no production Rust/formula/domain edit in
this extension. Private example/test files remain separately reviewed component
source, outside the conservative native selector's explicit qualification inputs.

PR132's represented-source exact predicate repairs five prior aperture artifacts
within its predeclared fixed corpus; original failed v2 fields and finite-error
budgets remain historical. PR134's eight aligned direct coupled-algebra cases
reuse the existing StaticObstacleGeometry/AlignedStrain and conditional exact-real
StokesWork identity. Neither selects a general cut/free-surface velocity space,
extension or physical traction/boundary lift. No PDE/physical/production-family
acceptance is granted here. Root observes the recorded component scientific
dispositions without rerunning their campaigns or transferring old receipts.

This explicit v3 map refresh authorizes **attempting fresh qualification only**.
The [frozen dependent protocol](../plans/fluid-accuracy-composition/reports/q2-admission-20261020/protocol.md)
requires independent composed-source/metadata review, clean committed-source
native/rational execution and a new real packet with all normal/optimized refusal
tests, plus separately retained focused composed and private-reference contracts.
Preserve all previous review text, v3 schema/base, immutable primary tree, frozen
v1/v2 bytes/digests, ancestry and complete selector/refusal checks. Commit this
bound review and inventory together. Fresh evidence binds the new exact source,
not 9c04278. Put outcomes in a separate report; do not change these bound bytes
after qualifying. Hosted, desktop/native-window, performance, model and physical
gates remain separate. Further cut/PDE method selection remains with its existing
numerical owner and requires its own reviewed admission before implementation.

## PR138 preliminary composition with reviewed PR142 source, 2026-10-21

The serial lead explicitly authorized isolated preparation against frozen PR142
`dbd49d3e41c482a753884abc8aaaff7d1b0d0653` in PR138 comment 6328266160,
without waiving final merged-main verification or authorizing a main merge. This
checkpoint retains PR138 `a6bfcc94481b1bafbe61be804a1abc3426927bbd`
and exact PR132/e6525a1 and PR134/f84f651 ancestry and tested reference bytes.
All prior review text above remains unchanged, including both PR142 histories
and the complete dependent Q2 admission suffix from PR138.

The unchanged conservative selector contains **152 regular files: 0 added,
2 changed, 150 unchanged, 0 removed** against the reviewed PR142 dependency.
Only Cargo.toml and Cargo.lock change; every product, thermal, UI, fixture,
workflow, toolchain, checker, qualifier and oracle selected identity remains
exactly PR142's. The complete sorted compact source-map SHA256 is
`9a10497158827e189fe505f65e1aa203cd8a28fe894c469408977071b055375f` (no trailing newline).
Cargo.toml SHA256 is `997fb098f13c60e4f85ead3f5f7f2527f87ad1cba04fdac82d0bfb24959fca3a`;
Cargo.lock SHA256 is `e64beca355b3963812b80d49ac5b17d0b1626f03f5d651b50e4dd25823dce96f`.

The two manifest requirements remain development-only num-rational=0.4.2 and
num-traits=0.2.19. The lock adds only num-bigint 0.4.8, num-integer 0.1.47 and
num-rational 0.4.2; all pre-existing registry identities/checksums remain unchanged.
Actual official crate archives and MIT/Apache license bytes match the retained
dependency review; no new root build.rs or build-dependency is added. The normal
no-default production graph excludes all three new crates. Private reference
implementation/test and scientific source pins remain exact; no sealed campaign
is rerun, relabelled or promoted to combined-source evidence.

The independently reviewed actual complete source and metadata are recorded in
`docs/plans/fluid-accuracy-composition/reports/pr138-integration-20261021/`.
This explicit v3 refresh permits attempting fresh qualification only after that
review approves the actual staged content. The old PR142 inventory refused the
changed Cargo.lock before numerical execution; its refusal log is retained.
Preserve schema/base, primary tree, v1/v2 bytes/digests, ancestry, complete selector
and all checker/qualifier/oracle/packet refusal contracts. No automatic pin
refresh, copied receipt, dirty override or omitted source is admitted.

Follow the PR138 prospective protocol: clean committed-source native/rational
qualification, a new real packet with all 26 normal and 26 optimized tests/no skips,
affected fixed ordinary contracts and hosted checks; retain failure/partial
outputs. Results belong in a separate report, not changes to these bound bytes.
Every receipt names its actual preliminary executed head. Final readiness still
requires verified PR142 merged-main identity, any necessary exact-source
revalidation, independent final review and the sole lead's normal merge.
No general cut/free-surface PDE, physical/production-water, graphical hardware,
learned/GPU or performance acceptance is granted by this reference/build stage.
