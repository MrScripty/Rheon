# M0 source reconciliation: published AI comparison

2026-10-15. Source/design reconciliation is complete for the pinned material below.
This report supersedes the earlier **source-access** blocker and the assumption
that no diagnostic viscosity/continuation owner exists. It does not admit production
implementation, merge the integration, qualify a reliable learned solver or claim a
fresh model/lifecycle execution. The original [research](research.md) remains the
unchanged historical review; [plan](../plan.md) owns the updated handoff.

## Exact source and evidence boundary

| Material | Observed identity / result |
| --- | --- |
| Rheon main | Fetched main remains `43ed26b645dd42de906d0fac7d1ec7d5ab6e351a` |
| Published integration | Remote `integration/ai-pressure-comparison-20261015` fetched at `0fbadbe733b5b521e192dc8a2aa980d6d69a1e01` |
| Previously unavailable original | Exact fetch of `66e50388ec7a788dbda0ad1227e330996edf506e` now succeeds; ancestry check proves it is retained in the integration, not replaced |
| Intervening content | `6c38078` adds comparison, `66e5038` lifecycle test, `fe9d58648f15a5aefd9e33fe47d4e6bc5f855b82` training viewer/shared voxel renderer, `0fbadbe` handoff/receipts and stronger lifecycle assertions |
| Matching lab | Remote `feat/sealed-water-pressure-training` remains `a8426061380e82951b4dba1bbfa38ff93dc7ebb2`; bridge/export entered at `d6a92920bd5d9bbcc693709182d9cf35f8a910db` |
| Training provenance | Published handoff identifies deployment training source `11408e93995a9f3c3f59e171ccd0889906531027`; this is distinct from adapter/export and integration revisions |
| Frozen compared checkpoint | `aa82bd122fde9406d9e6b92114b3e6ba77281f5ee8b9c726aa168dacbd93a62d`, 2,000 updates, deployment descriptor digest `be4eef1f051206205598e6876c83edaf5246138e17613f1aa787f3df183a761f` in retained receipts; actual bundle not present in this task |

The old `feat/live-ai-pressure-comparison` name still is not a published head;
the accessible integration branch is the authoritative review target. Main's
`session.rs`, `solver_provider.rs`, `recording.rs`, `simulation.rs`,
`provider_step.rs` and `liquid_step.rs` are unchanged by the integration, verified
by an empty scoped diff. No AGENTS.md or repository `.agents/skills` is in that
tree; workspace `.agents` is empty. Source was inspected from an isolated archive
of the Git object, without changing the checked-out documentation branch or local
implementation agents' files. The [published handoff](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/docs/ai-pressure-comparison-integration.md)
also identifies unrelated dirty local work; it is outside this task's ownership.

## Existing owners to retain

| Concern / source at integration revision | Established behavior / reuse disposition |
| --- | --- |
| [Comparison UI](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/mod.rs) | Start/cancel/restart, retained immutable paired frames, shared camera/time/slice/probe; close waits for worker. Extend these owners rather than build another app/supervisor. |
| [Worker](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/worker.rs) | Joinable thread, single-thread Rayon pool, fresh two-track engine, shared cancellation/child slot, terminal record; existing lifecycle is reuse input, not a missing implementation. |
| [Model process](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/model_process.rs) | Specific v2 CPU package decoder/protocol, direct trusted child and reader ownership. Generalize only the model-specific mapping/contract with actual consumers. |
| [Engine](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/engine.rs) and [Scene](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/scene.rs) | Existing experimental numerical owner uses native advection/viscosity/pressure/correction, retains its own histories and bounded diagnostic continuation. Extract/promote a typed boundary with this owner when admitted; no competing solver loop. |
| [Record](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/record.rs) | Leased project, distinct run directories, scene/model/initial-and-step frame/completion JSON plus atomic observations. These are existing raw diagnostic artifacts, not StepRecorder accepted publications or managed training Runs. Preserve them unchanged; recording-owner integration must keep diagnostic meaning. |
| [View](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/view.rs) | Signed pAI−pNative in Pa, shared pressure ranges, symmetric difference range, physical picking and slice. Reuse renderer/camera; add validated cross-run alignment and reductions rather than infer quantitative completeness from colors. |
| Existing public core and SQLite | PressureProvider/SimulationSession acceptance and StepRecorder/RecordingReader remain main's separate unchanged contracts. A local experimental engine does not grant viscous accepted-session composition or diagnostic-v1 archive eligibility. |

The training viewer/shared voxel renderer in `fe9d586` is a sibling consumer,
not a replacement numerical or experiment owner. Do not fold its source/asset
custody into the pressure contract or modify shared renderer files in this docs task.

## Physical and model contracts established by source

The scene fixes rho=1000 kg/m³, mu=0.001 Pa·s and dt=1/60 s. Engine preparation
performs native velocity/tracer advection, prescribed acceleration, then
`ViscosityWorkspace::update` with that mu, then builds the pressure RHS and
`r=b−A*p_previous` from that track's staged velocity. Thus the **diagnostic**
viscosity mismatch is resolved by actual composition, while the unchanged
production provider/session still does not implement that viscous plan.

Grid counts are independently 5–128 with product ≤65,536; spacing is cubic
`h=1/cells_per_metre`, integer cells/m 1–128, origin exactly zero. Unequal counts
produce rectangular domains. Stationary sealed free-slip walls, zero normal wall
velocity and constant density are the enforced scope: no inlet/outlet, obstacle,
free surface, variable density or mass emitter. Bridge scalar h must lie in
[1/128,1], rho and dt must equal the fixed profile. Training coverage (5–16 per
axis and 8/16 cells per metre) is narrower than permitted execution; larger inputs
are extrapolation. Nonzero origin and anisotropic spacing remain unsupported.
[Scene](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/scene.rs),
[engine preparation](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/engine.rs#L260-L300),
[lab bridge](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/cpu_bridge.py).

Native solves use selected Jacobi/SGS PCG, max 3,000 iterations, relative residual
1e-8, absolute 1e-12. AI comparison uses relative 1e-6, predicted divergence
≤1e-7 s⁻¹ and actual corrected-f32 divergence ≤1e-5 s⁻¹. These are this diagnostic
engine's thresholds, not an assertion that production full-step acceptance is
identical. Nonfinite/length/gauge output errors stop; Courant>0.5, nonzero normal
walls or net-flux violation hard-stop the trajectory. Pressure-equation/divergence
qualification failures alone remain visible **and advance the AI diagnostic
history**; native reference must qualify. No native fallback replaces the AI.
[Engine::advance/finish](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/engine.rs#L159-L389).

The v2 deployment contains source/weight/normalization/runtime inventory and
hash-bound training context. Rust checks schema/profile/physical declarations,
interpreter and bridge; the Python bridge checks the inventory/runtime before
inference. This is trusted-local measured execution with host libraries and no
mutation during use assumed, not authentication or a sandbox. The per-request
message has step ID, counts, h, rho, dt, prior/base pressure and native residual;
it has no viscosity or origin field. Their policy is in the selected scene and
deployment, not inferred from tensor dimensions. [Host decoder](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/model_process.rs#L174-L258),
[export](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/deployment.py).

The bridge reshapes x-fast f64 fields to `[1,1,Z,Y,X]`; model inputs become float32
`[1,7,Z,Y,X]`: residual/RMS, prior pressure/RMS, physical XYZ centers
`(index+0.5)*h`, rho/scale and dt/scale. Normalization is the frozen train-only
artifact. Model output is a same-grid correction, converted/scaled to f64 and
gauged at cell zero, then **added once inside the adapter** to prior pressure.
The bridge returns full Pa pressure, not a correction to add again. Zero residual
returns prior pressure exactly. [Model](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/cuda-pressure-v1/model.py#L154-L188).

The measured split `[15,63]` establishes 15 native steps, first AI inference at
physical step 16 using solved step 15 pressure. Warmup clones qualified native
state, does not call inference and contributes no AI quality count. Afterwards
each track uses its own pressure, velocity and tracer, including marked AI
diagnostic failures; no recurrent hidden memory or 15-frame input exists. The
bridge checks early IDs but trusts the caller's actual history. A paired frame
has common fixed physical time, not common RHS after trajectories diverge.
An initial zero-pressure frame is marked Initial, not a solved pressure.
[Engine history](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/engine.rs#L129-L259).

## Complete observed lifecycle and its limits

Start validates scene, allocates fresh history/cancellation/child slot and creates
a joinable worker. Model startup bounds descriptor to 1 MiB, hashes interpreter/
bridge, spawns Python directly with `-I -S -B`, clears inherited environment and
sets CPU/single-thread values. A second cancellation check protects normal
startup-slot installation. Owned stdout/stderr threads drain pipes; stdout records
are ≤8 MiB with a two-reply queue and finite pressure length ≤65,536; stderr
exceeding 64 KiB triggers immediate-child kill. Handshake binds checkpoint/training
identity, seven float32 channels, CPU, zero optimizer updates and no CUDA.

Cancel sets the flag and attempts immediate `Child::kill`; normal ModelProcess
Drop closes stdin, takes the child, attempts kill/wait, then joins readers.
Successful completion explicitly drops the process before terminal publication;
error unwinding runs the same drop path before recording failure/cancellation.
Worker join diagnoses panic; Worker Drop cancels/joins and makes a final
immediate-child cleanup attempt. UI defers close until worker completion/join;
restart creates a fresh worker/run under the project lease. This is observed
normal ownership order, not independently instrumented reader-custody evidence.
[Process/Drop](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/model_process.rs#L259-L445),
[worker](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/worker.rs#L45-L237),
[UI close](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/mod.rs#L82-L108).

There is no process-group/subreaper or descendant inventory. Kill/wait/join
results are best-effort, with weaker poison/panic/startup-failure reporting.
Startup receive has a 90s deadline; prediction receive's 30s deadline begins
**after blocking write_all**; the 600s worker limit is checked between advances.
Hashing, writes, native work, OS wait and reader joins have no independent hard
shutdown watchdog. A descendant holding pipe descriptors could obstruct reader
completion. Retain these limits; do not describe this trusted immediate-child
implementation as generic malicious-code containment or guaranteed bounded Close.

## What the test and retained receipts prove

The ignored real-deployment test is not part of the default seven passing
comparison tests. At the original revision it waits for step ≥17, cancels the
resident child, joins, asserts slot=None, cancelled shared/terminal JSON and
≥18 retained frames. Restart uses the same project directory, a distinct run,
16 steps and exactly 17 frames. Publication adds both immediate PIDs absent from
Linux `/proc`, plus **exactly one** restarted AI prediction at step 16 and no
AI quality in the initial/prefix frames. It is not a long restarted rollout.
[Current test](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/src/bin/desktop/ai_comparison/worker.rs#L261-L364).

This task independently checked the current worker SHA256
`b4f4e2fc0ef2dad94279a70e67015b6f8caa4e5e62dc0a612b38a44cd9d146ec`
against the receipt and all five log-wrapper and decoded UTF-8 text digests.
All match. The retained original log reports 1 lifecycle pass in 11.96s; stronger
publication log reports 1 pass in 11.30s. Receipt reports first 17 steps / 18 frames,
restart 16 steps / 17 frames / 1 prediction and Linux PID absence. Other retained logs
report 7 comparison tests plus 1 ignored, 47 desktop plus 1 ignored, 5 training-viewer,
4 bridge tests and selected-bin/test Clippy success. These are **producer-retained
local observations**, not fresh execution here. [Receipt](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/docs/ai-pressure-comparison-tests/integration-receipt.json),
[publication log](https://github.com/MrScripty/Rheon/blob/0fbadbe733b5b521e192dc8a2aa980d6d69a1e01/docs/ai-pressure-comparison-tests/lifecycle-publication.log.json).

Digests establish internal consistency of worker/log material, not authenticity,
the full executable/dependency build closure or the unavailable deployment/scene/
completion payloads referenced by hashes. The measured bundle/installed runtime
are not supplied in the published Rheon snapshot, so this task did not rerun the
ignored lifecycle test, inference or GUI. Fresh reproduction uses the published
command with an actual exported measured deployment and a fresh output path.
No required model dependency is installed implicitly.

The matching lab's retained 64-step result has 49 post-warmup AI checks and 0 passes;
the checkpoint remains accuracy-failing under that diagnostic experiment. Its
local native-window/field-import observations are recorded provenance, not a new
held-out benchmark or a generic saved-run-set/SQLite workflow acceptance.
[Lab retained acceptance](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/docs/plans/live-ai-pressure-comparison/reports/acceptance.json).

## M0 disposition and next bounded handoff

Source/owner/envelope reconciliation is satisfied. Concrete existing envelope:
the fixed-dt, zero-origin, cubic-cell sealed viscous-water **diagnostic** plan
above, with native prefix and separate raw AI verdicts. It resolves the earlier
missing-source/diagnostic-viscosity/no-fallback ownership questions. It does not
turn that engine into an accepted SimulationSession provider plan.

Retain the existing process/worker/UI/engine and lab adapter; introduce no duplicate
loop, supervisor or raw diagnostic store. M1's candidate change is a typed context/
adapter seam at these actual composition points, with the public API owner
serializing necessary exports and deciding whether/how a production viscous plan
is qualified. Existing diagnostic JSON stays raw evidence; recording owner alone
defines an additive diagnostic path under SQLite without relabeling accepted v1.
Teacher-stage bounded request capture remains new work; the engine's transient
Prepared input is not a saved equal-request fixture. Existing paired pressure/tracer
frames do not retain all native face velocity/operator fields needed for every
proposed metric. General saved specs/sets and cold quantitative comparison are
still proposed work, despite the existing local Scene JSON and live synchronized UI.

Initial generic comparison policy remains stop-on-reject. Current experimental
continuation is an existing **different diagnostic policy** to preserve and label;
promoting it requires explicit typed continuation/hard-stop/history semantics and
independent qualification, not another application-owned reimplementation. No
new implementation is admitted by completing this read-only M0 reconciliation.

At inspection, integration-head CI exposed only successful **Bounded dense3D
producer metadata** run [38751946726](https://github.com/MrScripty/Rheon/actions/runs/38751946726).
It is not CPU-model lifecycle CI. Prior PR84 head `4176435` had successful scoped
metadata checks and CodeRabbit status. Final documentation-head checks are recorded
in the ledger after push; retained local receipts remain separately identified.
