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
