# Proposed stage, model and saved-experiment contracts

Design notation only: these records are not implemented Rust APIs, executable
configuration or a new wire format. The implementation owner chooses a codec and
public API evolution strategy after phase 0. The [research](research.md) gives
source evidence and alternatives; [plan](../plan.md) owns current decisions.

## Authority and version boundaries

| Contract | Canonical authority | Consumers / compatibility |
| --- | --- | --- |
| Stage contract and qualified stage plan | Rheon numerical/API owner | Dispatcher, adapters, session; semantic change requires explicit version/admission |
| Model adapter manifest | Lab adapter owner | Rheon runtime admission; independent source/weight/adapter updates reference supported stage versions |
| Runtime messages | Runtime integration owner | Supervised process and host; closed versioned variants, no assumed cross-version compatibility |
| SimulationSpec | Application configuration owner | UI and run resolver; persisted revisions retain their schema and exact resolved inputs |
| Run instance and run set | Application experiment owner | Scheduler and cold comparison reader; references numerical archives, never owns field truth |
| Accepted/diagnostic records | Existing recording owner | StepRecorder/RecordingReader and separately typed diagnostic extension; accepted v1 interpretation remains strict |
| Comparison recipe/result | Analysis owner | Native numerical inspector, plots and export; analysis versions do not rewrite raw records |

Stage, adapter, runtime, spec, archive and analysis versions are independent.
Content digests identify exact bytes; they neither authenticate authors nor prove
numerical support. Unknown required versions are `unsupported`, malformed or
contradictory records `invalid`, missing required material `unavailable`.

## Typed stage values

Each name below denotes a validated bounded value or closed sum type, not an
unchecked string/map. Physical descriptors reuse existing grid/recording meanings
where they match. Optional fields carry an explicit absence reason.

| Value | Required members and invariant |
| --- | --- |
| FieldDescriptor | Semantic quantity/role, SI dimensions and scale, scalar kind, components, shape/strides/axis order, cell/face/particle/global association, sample offsets, coordinate frame, geometry identity, validity support, temporal stage and lineage |
| GridIdentity | Counts XYZ, spacing XYZ in m, origin XYZ in m, frame, boundary/domain/operator version; checked lengths and exact captured values, not counts alone |
| FieldTime | `AcceptedEndpoint(time,generation)` / `TentativeStage(interval,stage)` / `IntervalDiagnostic(start,end)`; actual dt, not requested dt |
| FieldLineage | Application run/attempt ID, source generation, producing stage, field content identity; never another track's implicit latest frame |
| PressureContext | Qualified sealed constant-density plan/operator identity, grid, rho kg/m³, actual dt s, full ungauged integrated RHS m³/s², explicit prior context, authoritative optional initial residual, cancellation/deadline and resource bounds |
| PriorPressure | `Absent(constructor-or-policy)` / `Accepted(field, interval, gauge, lineage)`; presence requires this track's committed generation and compatible geometry |
| InitialResidual | Named initial guess p0 Pa and host-computed r0=b−A*p0 m³/s² with operator/RHS/guess identities; distinct from RHS and final candidate residual |
| PressureCandidate | Full cell-centered x-fast f64 pressure Pa, exact required length, canonical p[0]=0, context/attempt identity; no mutable core state or operator replacement |
| QualifiedStep | Existing final accepted session publication and authoritative reports; pressure qualification alone cannot construct this value |

The first slot remains a full-pressure candidate, including when an adapter
internally predicts a correction. The host independently checks finite values,
gauge, full true residual and predicted divergence; existing full-step owners then
check actual corrected stored-f32 velocity divergence, Courant, transport, time
and liquid constraints. A later failure commits neither state nor prior-pressure
history. It does not silently retry the entire interval with another provider.

Conceptual transitions:

```text
ResolvedRun -> existing worker -> SimulationSession staged interval
  -> immutable PressureContext -> selected adapter/runtime -> PressureCandidate
  -> existing pressure qualification (or declared numerical fallback on same input)
  -> existing full-step gates -> final accepted publication
  -> commit explicit history -> existing recording adapter -> UI publication
```

Preserve the existing owner-specific publication ordering and recording-failure
contract: recording is after numerical acceptance and cannot roll back physics.
An unrecorded accepted tail is explicit. An uncertain commit stops appending and
requires reconciliation by a fresh reader before reporting completion/retrying.

## Model package and execution

`AdapterManifest` contains bounded IDs, required stage/runtime versions, immutable
artifact inventory/digests, named physical inputs, tensor ports/order/dtypes,
transform and reconstruction identities, history policy, scientific support,
runtime/device/dependency requirements, resource bounds, state/cancellation
capabilities, license/provenance and validation-evidence references. Discovery is
inert metadata reading. Source or weight execution is an explicit run/probe step.

`SupportResult = Compatible | Unsupported(reason) | Unavailable(material) |
Invalid(contradiction)`. Keep declared support, observed runtime capability and
numerical qualification as separate reports. No manifest checkbox admits new
physics. Reject unknown required keys/variants; optional extension fields require
an explicit versioned preservation/ignore policy, never accidental dropping.

`PredictRequest` binds runtime version, run/attempt/request ID, stage/context
identity, bounded descriptors and immutable input payload references. Structured
arguments select a supported runtime; no shell command string. Replies are closed
`Candidate` / `Rejected` / `ExecutionFailed` / `Cancelled` variants with matching
IDs and bounded payloads. Logs use a separate drained channel. Stale replies after
cancellation/reset cannot publish. Existing joinable worker owns supervision,
descendant shutdown and wait/reap; the process has no archive handle.

First runtime recommendation: explicitly trusted local code in a supervised CPU
subprocess, following review of the existing lab bridge. This isolates dependency
and crash failures; same-user code is not sandboxed against malicious behavior.
Trust-model confirmation is phase-0 admission, not an implemented security claim.
Model package copying cannot imply authority to install dependencies or execute.

Current lab profile at `a8426061380e82951b4dba1bbfa38ff93dc7ebb2` is an adapter
example, not the universal stage API. It packs residual/base pressure/X/Y/Z/rho/dt,
uses float32 inference and reconstructs full pressure as base + gauged correction.
Its 15 native steps establish prior-pressure context; it is not a recurrent
15-frame model. Scalar spacing requires cubic cells, permitting unequal counts;
the bridge's counts/volume/density/dt/spacing limits remain explicit. Nonzero origin
is unqualified. Deployment mu=0.001 Pa·s is incompatible with simply claiming the
current viscosity-free full-step provider plan reproduces that physical setup.
Phase 0 must resolve this before admission, not silently set viscosity to zero.

Stateless adapters with explicit context come first. Hidden evolving state needs
tested prepare/commit/abort or full snapshot/restore including RNG and caches;
otherwise restrict it to a documented diagnostic/nonretryable mode. Existing
PressureProvider does not provide rollback of external side effects.

## Saved scenarios, attempts and sets

`SimulationSpecRevision` is rerun intent: schema, scenario/initial-state assets,
domain/boundaries, physical parameters, source/forcing schedules, requested time
policy, qualified stage plan and provider bindings, tolerances, runtime selection
and recording/resource budgets. Freeze content before execution. Camera, slice,
colormap and playback live in a separate `ViewSpec`. Opaque current Rust Debug
configuration remains provenance, not a parser input or migrated runnable spec.

`ResolvedRunAttempt` binds a stable application ID, immutable resolved spec,
actual binaries/build/runtime/device/package identities, archive reference,
actual time sequence and terminal outcome. A retry creates a new attempt. Run-local
native IDs remain run-local. An unfinished archive retains active/incomplete state;
reading a valid last record does not prove a completed run. Only after close/finalize
may a whole-file archive digest describe an immutable completed artifact.

`RunSetRevision` has schema/revision, members (`SpecRevisionRef` or `RunAttemptRef`),
baseline member, intentional variant differences and `ComparisonRecipeRef`.
Start with a bounded manifest over existing SQLite files; add no numerical store.
Membership edits create a revision without deleting runs. Missing/corrupt/runtime-
unavailable members are visible; cold inspection requires no provider execution.

Example saved set: one common sealed-box scenario revision, native Jacobi PCG,
native SGS PCG and one qualified model+fallback binding, with the pressure binding
as the only intentional difference. A teacher-stage set references captured equal
requests separately. Independent rollout uses each track's own history and labels
native-prefix context. None of these archive frames is a restart checkpoint.

## Comparison and diagnostics

`ComparisonRecipe` declares mode, exact members/baseline, field roles, support,
geometry/association/unit/stage constraints, alignment policy and tolerance,
gauge policy, reductions, zero-reference policy, timing scopes and analysis version.
`ComparisonResult` binds the recipe and exact run/publication/field identities,
coverage/missingness, transforms, metric values/units, timing samples and outcomes.
Results are derived evidence; explicit recomputation creates a new identity.

Initial alignment requires compatible native grids and matching actual physical
times/intervals within a recorded tolerance. Ordinal equality is insufficient.
No automatic spatial/time remapping; future remapping records method, support,
source samples and error and cannot cross resets/events. Keep projection interval
pressure separate from endpoint/EOS/absolute pressure. Direct pressure differences
require matching gauge; optional offset-aligned diagnostics retain offset and
canonical-gauge failure. Never equate cross-run particle IDs by index.

First reductions: host true full residual L2/L∞ and named threshold/RHS/operator;
actual stored-velocity divergence/Courant; volume-weighted RMSE
`sqrt(sum(w*difference²)/sum(w))`, L∞, support count/coverage and units; matched-time
rollout error, first rejection and accepted/attempted horizon; fallback count over
all attempted intervals; separate inference/IPC/pre/postprocess/qualification/
fallback/full-step timing. Empty support is unavailable. Zero reference norm uses
an explicit undefined-relative result plus absolute error, not a hidden epsilon.
Sample distributions, warmup, sequential order, failures, device/build/precision
and timing scope accompany performance comparisons. No speedup claim is made.

Three mode variants remain distinct: qualified model+fallback publishes only
accepted full steps; teacher-stage evaluation records candidates against identical
captured requests; independent diagnostics use their own lineage. Initially stop
on rejection. Teacher fixtures require an explicit bounded capture at authoritative
stage preparation: current accepted archives lack RHS/residual/operator context
and cannot reconstruct equal-input requests. A diagnostic candidate-attempt path
must return rejection without native fallback and retain bounded candidate evidence:
current ProviderWorkspace always falls back and overwrites rejected pressure.
Observing that completed production call cannot implement stop-on-reject.
Rejected attempts require a versioned diagnostic schema/producer
inside the current recording owner with separate typed reader/append paths;
accepted-v1 decoders remain strict. Failed-state continuation additionally needs
a Rheon-owned diagnostic advance contract and hard-stop rules. Diagnostic payloads
never acquire accepted-publication meaning. Preserve bounded nonfinite rejection
details without normalizing bad values into accepted finite fields.
