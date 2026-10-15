# Diagnostic pressure attempt recording

`rheon::recording::diagnostic` records bounded observations of the existing
`sealed-mac-pressure-context-v1` stage. It is a separate typed SQLite format under
the recording owner. It does not contain accepted simulation publications,
accepted ordinals, restart state, an executable configuration, or a model loader.
No numerical loop, runtime supervisor, background writer, catalog, migration,
eviction, automatic cleanup or writer resume is provided.

The current schema uses `application_id = 0x52484450` (`RHDP`), `user_version = 2`.
Both values are required by `DiagnosticReader`. These version numbers belong to
the diagnostic family; they do not change accepted archive version 1. The tables
are `diagnostic_run`, `diagnostic_lifecycle`, `attempts` and `pressure_samples`.
There is no `steps` table. Accepted archive readers reject diagnostic databases
as `Unsupported("archive kind")`, including when a diagnostic header claims user version one;
accepted selection/full verification also reject a changed application ID after open.
Diagnostic readers reject accepted databases. Cold inspection executes no Python
or other model code.
Diagnostic schema one is unsupported by this reader. There is no automatic
migration or modification of existing user data to add schema-two seed origins.

## Capture and append

Create a fresh caller-owned local path with
`DiagnosticRecorder::create(path, DiagnosticRun, RecordingLimits)`. Existing files
are refused. The caller protects the path and ancestors from concurrent
replacement. SQLite local filesystem locking/flush assumptions apply. Network
filesystems, hostile concurrent mutation and author authentication are outside
this contract.

`DiagnosticRun` freezes track/run ID, source revision, grid, density, dynamic
viscosity, fixed actual increment, diagnostic mode, provider binding identity and
bounded provenance. `native_prefix_steps` is a required caller-observed/declaration
of warmup length, bounded to 0–512. A nonzero declaration requires an observed
seed and cannot exceed its generation; its value is not inferred from that generation and does not certify
that preceding requests were recorded or physically qualified. The binding stores ID/version, manifest SHA256 and optional
checkpoint SHA256/training-run identity. Identifiers and digests are observations,
not proof of authorship, reproducibility, model trust or scientific quality.
Native-only readers do not resolve those references.

The optional `ObservedHistory` seed stores exact pressure values, generation,
interval end and `PressureHistoryStatus`. Its run/grid/density are those of the
immutable header. This is an explicitly observed starting state, including a
possible post-warmup state. It does not invent or certify the preceding warmup
history. A missing seed requires the first request to have no prior, generation
one and interval start zero. A seed requires the first request to use that exact
history. Diagnostic candidate output is never promoted to accepted history.
The mandatory `seed_origin: Option<SeedOrigin>` is paired with seed presence.
`AcceptedNative` requires observed `Accepted` history. `QualifiedNativeDiagnostic`
and `QualifiedDiagnostic` require observed diagnostic history with `qualified=true`.
Teacher mode permits only the two native origins; qualified model-origin history
cannot initialize its trajectory. Independent mode permits all three origins.
Unqualified diagnostic seeds are rejected in both modes: this format cannot
resume a failed diagnostic trajectory. A qualified experimental native engine
uses `QualifiedNativeDiagnostic` and retains diagnostic status throughout; it is
never relabeled as accepted Session state. Origins and qualification remain
explicit owner observations, not independently proven scientific facts.

`DiagnosticRequest::capture(&PressureContext, viscosity, payload_limit)` accepts
only `DiagnosticCandidate` contexts. It copies the authoritative RHS and optional
prior and computes `b - A*p0` through the existing context API. It checks the copy
budget first. Absence of prior remains absence; it is not solved zero pressure.
Pressure fields use x-fast f64 cell values and cell-zero pressure gauge. RHS and
residual use the stage contract's finite-volume equation units (`m3/s2`), pressure
uses Pa, density uses kg/m3, viscosity uses Pa s, grid distances use m and interval
values use s. Storage does not admit new adapter physics or physical support.
Request/header domain values must match by exact bits, including signed zero in
viscosity and grid origin. The writer rejects a mismatch before committing;
hashing and cold reconstruction never silently normalize these values.

`append(&DiagnosticAttempt)` records a contiguous zero-based **attempt index**,
physical request, dispatch fact, outcome, optional equal-request native reference
and actual history transition. Attempt indices have no accepted-generation or
accepted-ordinal meaning. The request preserves its track generation and actual
interval separately. No stability retries or request retries are supported in
this first format.

`PredictionOutcome` is either a finite, correctly shaped, canonical-gauge
`PressureSample` or an explicit `NoCandidate` with a closed reason, bounded detail
and optional digest of an invalid reply. Reasons are unsupported support,
unsupported context, cancellation, runtime failure and invalid output. A malformed
or nonfinite reply is not encoded as a canonical candidate or replaced with zeros.
Its optional digest preserves identity only; raw invalid reply bytes are not
retained. A candidate or invalid-output failure requires dispatch. Runtime
failure, support/context rejection and cancellation can occur before or after
dispatch. Dispatch is the host's observation of initiating the provider call,
not proof that a remote process consumed its bytes.

`NativeReference` carries the request digest, a pressure sample and the native
owner's explicitly observed `PressureHistoryStatus`. A mismatched
digest is rejected. This binds the reference to the captured RHS/prior/interval,
not merely to a matching frame index, shape or physical time. A pressure from a
separately evolved native track cannot substitute for a same-request solve.

`QualificationObservation` stores the closed `HostPressureGateV1` policy,
explicit thresholds, measured residual/divergence/Courant/flux values and the
producer's qualified/hard-stop verdict. Values must be finite, norm/threshold
quantities nonnegative, and qualified cannot coexist with hard stop. The recorder
also records `walls_closed`. A false wall observation requires `hard_stop=true`
and `qualified=false`, matching the declared gate's structural wall rule.
The recorder does not execute this policy or certify its verdict. In particular it does not
reproduce advection/viscosity, wall checks or actual-velocity diagnostics from
pressure alone. Prepared face velocities are not captured here. Schema coherence,
hash integrity and a self-declared qualified bit are not independent physics
qualification or accepted-native status.

The two supported transition policies are:

- `IndependentStopOnReject`: a host-qualified candidate must record
  `DiagnosticAdvanced`; an unqualified or absent candidate records `NotAdvanced`.
  Its next prior must be the exact preceding candidate, marked diagnostic. A
  nonadvance closes append admission for the run.
- `TeacherStage`: `NativeTeacherAdvanced` requires a same-request native sample
  reported qualified by the host. Its next prior is that exact native sample
  with its explicit native-owner history status, regardless of the model candidate's verdict or bias.
  An observed `Accepted` status is retained as such; the recorder does not infer
  or grant it from the sample's qualified bit. An observed diagnostic status is
  likewise retained exactly; a native history explicitly reported unqualified
  cannot advance the teacher chain.
  `NotAdvanced` stops the run. The candidate never becomes teacher history.

The recorder checks these declared transition facts; it cannot observe or mutate
the numerical owner's actual memory. The owner supplies the completed transition,
then synchronously records it before publishing its matching UI observation.
Recording failure stops the existing worker and cannot undo an owner transition.

`CancelledBeforeAdvance` records a dispatched candidate whose host qualification
finished before cancellation prevented state/history commit. It is legal in
either mode and retains the candidate and its actual observations, including a
qualified verdict. It closes append admission and creates no next history.
`NotAdvanced` still rejects a qualified independent candidate; cancellation is
represented by this explicit variant instead of relabeling output as absent.
Its terminal outcome must be cancelled, or failed if recording/terminal work
subsequently fails; completed and stopped-on-rejection outcomes are forbidden.

## Lifecycle and cold verification

Each append is one SQLite transaction, including metadata and both pressure roles.
Any append error poisons further admission, including validation, quota and SQLite
errors. Never retry an uncertain append. `recorded_count()` counts acknowledged
commits only. A cold reader reconciles the actual durable prefix. No destructor
marks completion; dropping a writer leaves active/incomplete status.

`finish(DiagnosticTerminal)` closes admission before validation or writing. It can
be attempted once, including after append error, to disclose completed horizon,
stopped-on-rejection, cancelled or failed status. Completed status requires a
nonempty fully recorded advancing prefix and an unpoisoned writer. A rejection
terminal requires an independent unqualified candidate with no history advance.
Both require produced-attempt count and advanced generation to exactly match the
recorded facts. Failed/cancelled status can disclose an unrecorded produced tail,
but cannot fall below the durable prefix. Any claimed new advanced generation
must exceed the observed seed generation and fit within seed plus produced
attempts using checked arithmetic. The durable advanced tail plus missing
attempt count supplies a further upper bound; a known stop cannot claim a later
advance. No new advancement beyond the seed is represented by `None`, not by
repeating its generation. These are chronological consistency checks, not proof
of the unrecorded tail. Terminal validation/write failure leaves
an uncertain terminal outcome and requires cold reconciliation. Active on reopen
means incomplete/unknown, never successful completion.

`DiagnosticReader::open`, `count`, `select`, `status`, `verify` provide indexed
access. Selection revalidates the immutable header, bounded inventory, request
context/residual, pressure samples, transition shape and hashes. `verify()` also
audits the full contiguous history chain, stop boundary, orphan samples, SQLite
integrity and terminal prefix within one database snapshot. Calling `verify()`
is required for a completed-archive or complete-lineage claim. An already returned
owned selection remains unchanged after later database mutation or append.

## Bounds and hash scope

The recorder reuses `RecordingLimits`, the existing recording connection's 250 ms
busy timeout, approximately 1 MiB page cache, no mmap and file-backed temporary
storage. It uses DELETE journals and FULL synchronous commits. The database cap
is floored to 4096-byte pages; active rollback journal/filesystem overhead are
additional. No whole-process RSS, hard-real-time or hard total-volume quota is
claimed. SQLite cache settings are targets rather than allocator caps.

At most 65,536 cells are supported. Metadata strings use the existing 16 KiB
text limit; track/run IDs additionally use the pressure context's 4096-byte limit.
There are at most two pressure samples per attempt, fixed schema fields and one
bounded header seed. Header and whole-attempt budgets conservatively include
fixed descriptors, text, every cell array and both samples; the existing maximum
record cap is 64 MiB. SQL length checks preflight aggregate selected array/text
bytes before field copies. A reader enforces both its supplied limits and the
recorded limits. Quota includes each diagnostic attempt, not the header seed.

The run hash uses `rheon-diagnostic-run-v2\0`; request, attempt and terminal
hash framing retains its separate `-v1\0` tags. Integers and f64 bits use little endian. Text/byte strings have LE u64
byte-length prefixes; f64 arrays have LE u64 value counts. Geometry has three
fixed count integers and six fixed f64 values. Optional values have an LE u64
presence value. Run hash
includes immutable geometry, binding, seed, limits and `keep_all_stop` retention.
Request hash includes physical stage/domain, lineage, RHS, prior and initial
residual. Attempt hash binds the run hash, index, entire request, outcome,
observations, equal-request native reference and transition. Terminal hash binds
the run hash and declared outcome/tail. The prefix declaration and native history
status, seed origin, wall observation and transition are included in their respective
run/attempt hashes. Hashes detect accidental mutation; a writer
able to replace both values and hashes can forge observations.

`tests/diagnostic_recording_contract.rs` exercises actual SQLite close/reopen,
teacher/native history versus biased candidates (including multiple requests
using actual accepted `Simulation` history), stop-on-reject, explicit missing
candidates, cancellation, incomplete Drop, kind isolation, tampering, real SQLite
FULL, quotas and terminal admission. These fixtures qualify storage semantics;
they make no trained-model, window, acceptance, speed or accuracy claim.

Timing/cost samples, complete warmup request sequences and runtime tensor/port
bytes are not captured by this schema. Their absence means unavailable evidence,
not zero cost, zero warmup or an inferred execution count. Produced diagnostic
attempt counts and the declared native-prefix count are separately explicit.

## Existing CPU runtime boundary

The desktop model process exposes `predict_typed(Input, cancel)` and retains
`predict(Input, cancel)` as the legacy string-error wrapper. `PredictionFailure`
preserves a closed no-candidate reason, sanitized bounded detail, whether the
existing writer accepted dispatch, and an optional invalid-payload SHA256. The
reason comes from typed support/context, protocol, pipe and deadline branches;
display text is never parsed to classify a failure. Dispatch does not prove the
child consumed bytes. Pre-dispatch failure stays false; cancellation, transport
or deadline failure after initiation stays true. Bridge `invalid` means invalid
request context; malformed/wrong-correlation/wrong-shape/nonfinite replies mean invalid
output. For complete invalid wire replies the digest includes the exact received
protocol line and its newline; an incomplete oversized prefix has no claimed full
payload digest. The existing child, bounded channels, readers, writer and Drop
remain the only runtime owners. Real stdlib subprocess fixtures exercise this
boundary without claiming trained-checkpoint qualification.

Frozen reruns call `ModelProcess::start_pinned` with the expected complete
`ProviderIdentity`. It shares the existing startup path and compares the inertly
resolved identity before `Command::spawn`; changed bindings cannot execute model
code merely to discover a handshake mismatch. The legacy `start` remains available.
The trusted-local no-concurrent-package-mutation assumption still applies through
validation and execution. The identity's serialized shape is unchanged.


Final serial-owner validation on f56a756: clean native/rational qualification and
fresh packet26 tests in normal/-O modes (zero skips),17 diagnostic+15accepted
recording+4carrier+20nativecomparison contracts, and scoped core Clippy all pass.
The final reviewed source closes signed-zero domain, rejected-seed and impossible
terminal-tail findings before admission. This remains a storage/API slice:
Engine/ModelProcess/worker capture, explicit policy execution, saved diagnostic
intent/membership and record-before-UI integration have not been implemented here.
It cannot supply missing trained-runtime, linked-field or complete A6 evidence.
