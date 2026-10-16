# Configured accepted recording comparison

Configured accepted publications use the existing recording comparison owner.
`compare`, `align`, `compared_values` and `native_samples` retain exact Selection
identities, native samples, physical-time/stage/interval matching, units, gauge
policy, arithmetic refusal and shared scale reductions. Native carrier profiles
keep their original interpretation. The configured profile is admitted only
through `RecordedConfiguredSession::decode`, including any RCP3 association.
Unknown profiles and malformed configured fields remain unavailable; a profile
is never relabelled to pass a native eligibility check.

Comparisons use actual stored face velocities, tracer and interval pressure.
Divergence is derived from staggered native faces and recorded spacing, not a
rendered reconstruction. Configured pressure intervals retain their actual owner
start in `evaluation_time`; an absent start is not reconstructed as endpoint
minus dt. There is no interpolation or remapping. Equal ordinals need not be
aligned physical intervals, and equal physical time does not establish equal
RHS, operator or trajectory. Unmatched coverage stays visible alongside matched
results.

The additive `recording::provider::comparison` API exposes `inspect`,
`rhs_relation` and `summarize`. `inspect(&Selection)` returns configured
selection facts: the exact identity, optional accepted interval, owned core
receipt and separately available runtime observation. Constructor selections
have no accepted interval or provider receipt. RCP2 runtime absence is retained
as unavailable custody, not interpreted as no prediction or dispatch. RCP3
observations preserve the runtime owner's declarations separately from host
candidate acceptance, rejection and native fallback.

`rhs_relation` reports `EqualHashes`, `DifferentHashes` or `Unavailable`.
A known relation requires both actual authoritative configured RHS hashes;
carrier-only publications and missing context leave it unavailable. Equal hashes
refer to retained RHS payloads. They do not authenticate execution or prove
identical operators, requests or physical accuracy.

`summarize(&[Selection])` is bounded to 128 strictly increasing ordinals from one
identical run digest. Gaps are allowed and remain a sampled subset. It retains
exact selection identities and counts available constructor, accepted, attempted,
prediction-called, selected, rejected and runtime-observed facts. Those counts
are not dispatch counts or complete-run coverage. Core pressure durations use
checked sums of total, inference, qualification and fallback scopes. They are
pressure-call observations, not full-step wall time, isolated model compute,
startup, recording, rendering, throughput, RSS or a speedup claim. Missing
terminal history and unrecorded accepted tails cannot be inferred from selections.

The library does not resolve a frozen spec, member path, finalized artifact pin
or deployment. Existing application archive readers own readiness and typed
terminal custody. This API constructs no Session, model process, worker, store
or alternate metric engine, and archives remain inspection evidence rather than
restart checkpoints.

Contracts use real Session/SQLite publications, independent analytic/native-field
metric expectations and cold reader processes. Any authored runtime metadata is
labelled as such; it does not establish an actual ModelProcess observation,
measured learned-model accuracy, GPU execution, GUI behavior or performance.
