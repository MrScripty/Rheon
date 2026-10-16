# Configured runtime observations and termination

This contract adds bounded, owned runtime/application observations to the
configured Session recording profile. The existing Session owns numerical
acceptance and rollback; the existing `StepRecorder` owns SQLite transactions,
limits, poisoning and terminal publication. No process, model, numerical loop,
store or supervisor is constructed by these codecs or their cold readers.

## Accepted publication

Use `recording::provider::configured_publication_with_runtime(snapshot, ordinal,
limit, observation)` with an observation captured by the runtime owner. Passing
`None` produces the unchanged RCP2 publication. Passing `Some` writes an explicit
RCP3 supplement after the core acceptance receipt. RCP2 reads expose
`RecordedConfiguredSession.runtime == None`; absence does not mean that no
runtime was called. Existing SQL schema and native/diagnostic profiles retain
their meanings. Unknown versions and trailing bytes are refused.

The runtime owner calls `RuntimeRequestStamp::capture(request, binding)` on its
actual `ContextPressureRequest`, then supplies `RuntimeObservation { request,
outcome }`. The owned request stamp retains canonical run ID, geometry, density,
actual viscosity coefficient, generation, exact interval/dt, native prefix,
accepted-prior generation/end/pressure hash, RHS hash and exact package identity.
It stores no request, pressure or history arrays. Hashes use the same
little-endian IEEE value framing as the core context receipt.

`RuntimeOutcome` is `CandidateReturned` or `NoCandidate { reason, detail,
dispatched, invalid_payload_sha256 }`. Reasons reuse the existing closed
`recording::diagnostic::NoCandidateReason` operational vocabulary. They do not
promote diagnostic numerical history into accepted Session state. The runtime
owner records actual dispatch and outcome; neither core `Execution` nor
`prediction_called` can supply those missing facts. Dispatch means the writer
accepted a request, not that the child consumed its bytes.

A supplied accepted-row observation must match the current actual core
prediction, package binding, context, run, geometry, density and viscosity bits.
Runtime unsupported results require a core unsupported rejection; runtime or
invalid-output failures require core execution rejection and native fallback.
A returned runtime candidate may subsequently qualify or be rejected by the
host gate. Cancellation cannot be attached to an accepted interval. Raw rejected
numerical IEEE diagnostics remain in the separate core receipt; accepted fields
and diagnostics retain their finite-value requirements.

An invalid-payload hash is admitted only with dispatched `InvalidOutput`. It is
an owner observation of a complete payload, never evidence that a bounded prefix
was complete. The codec has no wire bytes with which to authenticate that claim.
Package/source hashes and all supplied observations are custody declarations,
not authentication or independent reconstruction of the RHS or force schedule.
An unaccepted request's zero coefficient alone does not establish whether a
viscosity stage was requested; accepted stage presence remains the RCP2 fact.

## Terminal publication

`RuntimeTermination::capture(snapshot, cause, detail, observation)` captures the
actual retained accepted generation/time and source/configuration association.
`finish(&mut StepRecorder)` uses the existing terminal boundary with an explicit
RCT1 payload in `RecordingStatus.detail`. It does not append an accepted row,
retry a poisoned publication, or resume a writer. Snapshot capture and terminal
storage are separate operations; if terminal storage itself fails, reopen with
a fresh reader to discover the durable outcome.

| Typed cause | Existing lifecycle outcome |
| --- | --- |
| `Completed` | `completed` |
| `Cancelled` | `cancelled` |
| `SessionDeadline`, `RuntimeFailure`, `StartupFailure`, `NumericalFailure`, `StorageFailure` | `failed` |

An observation can refer to the actual last accepted interval, including a
native fallback after runtime failure, or the unaccepted next attempt starting
at the retained accepted time. These associations remain distinct. Startup
failure has no invented prediction, request or dispatch. Completion cannot
claim an unaccepted attempt. The terminal cause is the owner's declared stopping
cause; a successful runtime candidate can still precede numerical cancellation
or failure. A cause alone does not establish completion of a frozen future
recipe.

`RuntimeTermination::decode(run, status)` refuses active/legacy generic detail as
typed runtime termination. Generic observations remain inspectable through the
existing status API. `audit_runtime_termination(reader)` first performs complete
configured verification, then checks available last-publication, request/prior
and terminal associations. It exposes `VerifiedLastPublication`,
`UnrecordedAcceptedTail { last_recorded_generation }`, or
`NoRecordedPublication` coverage. Verification describes retained association,
not authenticity or proof that external execution happened. A declared accepted
tail beyond the recorded prefix cannot independently establish its final field,
clock, runtime request or native-prefix association; the coverage result makes
that limitation explicit. Unknown generic terminal evidence cannot pass as a
typed verified result.

Both encodings use bounded binary metadata rendered as lowercase hexadecimal,
with closed tags/booleans, checked lengths and exact float bits. Existing 16 KiB
metadata limits and 1,000-byte individual string limits apply; aggregate or
per-record limits can refuse otherwise individually valid metadata. There is no
truncation, prefix-hash substitution, automatic eviction or second history cache.

## Verification scope

Focused contracts use actual Session/SQLite publications and explicitly authored
runtime/application metadata to challenge capture, association, limits and cold
inspection. A cold subprocess constructs only the recording reader/codecs;
packages, a model process and a fresh Session are unnecessary. Such tests do not
establish a real `ModelProcess` observation. The production bridge owner must
separately execute the actual runtime and demonstrate its mapping into this API.
Archives remain inspection evidence, not restart checkpoints. Cooperative
cancellation and terminal records establish no hard shutdown, descendant cleanup,
performance, model accuracy, GPU or GUI guarantee.
