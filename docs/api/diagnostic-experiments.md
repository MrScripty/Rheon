# Saved pressure diagnostic experiments

The existing desktop comparison owns three explicit execution policies. **Legacy
raw continuation** remains the default and retains its existing JSON/lab-reader
behavior. **Teacher stage** evaluates a candidate on each authoritative native
request; only the qualified native state and pressure history enter the future.
**Independent, stop on first rejection** uses its own diagnostic trajectory after
the native prefix; the first failed candidate is retained without advancing either
state or pressure history. These are diagnostic experiments, never accepted
`SimulationSession` publications or accepted SQLite ordinals.

In the comparison sidebar, choose the policy, CPU package, physical scene, native
prefix and horizon, then start a fresh comparison. The standalone comparison
binary additionally accepts `--mode legacy|teacher|stop-on-reject`:

```sh
cargo run --locked --features desktop --bin rheon-ai-compare -- \
  --deployment /absolute/trusted-local/adapter.json \
  --project-dir /absolute/fresh-diagnostic-output --steps 32 \
  --mode teacher --verify-run
```

The CLI uses the existing default scene/prefix (15 native steps); edit other scene
values in the UI. `--verify-run` prints terminal observations and returns an error
on execution or persistence failure. A declared stop on candidate rejection is an
expected diagnostic outcome, not evidence of qualified model accuracy.

## Contract and physical ownership

Model-specific inputs, preparation, weights and full-pressure/correction output
are selected within the existing declared manifest/stage envelope. New physical
semantics still require qualification. The host owns the physical context,
operator, initial residual, output reconstruction and actual pressure/transport
qualification. No additional solver loop or process/session owner is introduced.
The existing Engine, Worker, ModelProcess, Record lease and recording owner retain
execution, cancellation, child kill/wait, I/O joins and persistence responsibilities.

Typed requests retain native geometry, rho/mu/dt, actual interval, RHS, initial
residual and optional pressure history with exact run/generation/time/status.
Typed time starts the next request at the previous committed endpoint; it does
not regenerate that endpoint with a separately rounded step multiplication.
Teacher history carries `QualifiedNativeDiagnostic`, not fabricated `Accepted`.
Independent history carries its actual diagnostic status. A successful prediction
that is cancelled before commit remains a candidate with
`CancelledBeforeAdvance`; it produces no advanced frame or future history.
Runtime/unsupported/invalid/cancelled absence uses typed reasons and actual
`dispatched` facts. Complete invalid protocol lines retain their measured SHA256;
the engine never invents a protocol-line digest from reconstructed pressure.

The independent equal-request reference solves the **captured diagnostic RHS**
through the existing native workspace. It is not the paired independent native
track at the same step. The optional reference retains its actual qualification
and request digest. The host records actual residual/divergence/Courant/flux/wall
observations and the thresholds already used by `finish`; storage does not grant
physical qualification or introduce a second gate.

## Frozen intent and durable attempts

A fresh leased output directory has a create-new time/PID identity; a rerun always
creates another owned directory and records the parent set digest. It never
resumes a selected field or overwrites the original run. The versioned strict
`intent.json` captures policy, complete Scene, absolute deployment path, provider
identity/manifest/checkpoint pins, source revision declaration and recording
budgets. The archive header binds this entire canonical specification by digest.
Startup compares the inert loaded provider identity to the frozen pin **before
child execution**. Changed or absent runtime material is a failed rerun, not a
substitute model. A build without `RHEON_SOURCE_REVISION` says its revision is
unspecified; that string is not executable authenticity evidence. Saved execution
requires exact equality with this executor's declared stamp before output/process
creation. Any mismatch, including known-versus-unspecified, rejects the frozen
rerun; historical cold inspection remains available. Matching unspecified stamps
are allowed but establish no build parity. Retarget a new specification explicitly
rather than silently rewriting frozen intent or attributing a new build to an old
stamp.

`attempts.sqlite` is diagnostic RHDP schema **2**, separate from accepted archive
v1. The Engine advances only under the declared policy; its writer appends each
attempt and actual transition before JSON/live publication. Rejected/cancelled attempts remain distinct from
advanced frames. Produced and acknowledged durable counts are separate. Append
failure stops execution; a cold-reconciled Failed terminal may disclose an
unrecorded tail. Uncertain terminal writes leave an incomplete committed prefix.
Persistence errors take Failed precedence even when cancellation races them.
There is no terminal retry or claim that a failed append necessarily did not commit.

Only after the writer closes can `finished.json` pin the whole archive and its
verified terminal/counts. Publication uses the existing create-new run-set owner.
Cold reads recheck the pin, header/spec identity, terminal, native prefix and
recorded category counts. Observed totals beyond a declared unrecorded tail are
explicit producer observations. Missing, invalid, active/incomplete, closed
unpinned and pinned closed sources remain distinct. An intent alone is not a
completed run. The current UI creates one-member diagnostic sets; the durable
member contract admits bounded named selections without inventing comparisons
between incompatible requests.

The keep-all-then-stop defaults are at most512 requests, a1GiB database and16MiB
per request payload, additionally constrained by the existing Scene history
budget. SQLite's metadata/page overhead remains part of the database limit.
The actual native prefix is recorded even if cancellation occurs before the
requested prefix completes; a zero-attempt archive is meaningful.
Saved reruns publish the exact frozen intent and parent link before checking
mutable runtime package contents or support. Missing, changed or unsupported
packages retain a Failed finished observation with no invented archive, provider
or inference. Executor-stamp mismatches still refuse before output creation.

Cold app run-set verification binds every archive, including unpinned and
incomplete sources, to the frozen requested horizon. Any produced attempt requires
a completed native prefix. A nonzero prefix must have qualified native diagnostic
seed history at that exact generation and repeated-DT end time; prefix zero has
no seed. Generic diagnostic archives retain their broader observed-seed contract.
An honest within-horizon produced-but-unrecorded capacity tail remains valid.


## Cold inspection and synchronized quantitative pressure

Open **Inspect saved diagnostic attempts**, select an intent or finished document,
and explicitly **Open / verify**, then **Load attempt**. Reading imports no model
code. The inspector caches one selected attempt; namespace/member/load failures
clear old fields. The live attempt inspector is separate from the advanced frame
timeline. A fresh saved rerun replaces its owner before repainting any snapshots.

Two cell-pressure slices share Z index, physical probe, cell-zero gauge, units and
common full-volume range. Candidate-minus-native uses its own signed delta range.
The request's actual interval and history transition are visible; raw RHS,
initial residual, prior pressure and candidate/native values are available at the
probe. Missing candidate/reference remains explicit and draws no fabricated field.
These display tiles do not interpolate, remesh or imply accepted state.

`DiagnosticAttempt::pressure_comparison` requires the optional native reference
to name the exact request digest and both pressure arrays to retain cell-zero
gauge. It reuses the existing native finite-sample reduction for L2, RMSE, L∞,
relative L2 (None at zero native norm), support and common bounds. It applies no
mean-offset transform; `GaugeReport.diagnostic` is false for that direct canonical
reduction. The attempt itself remains diagnostic, including rejected candidates.
Arithmetic overflow is unavailable, not an infinity/zero metric.

**Export exact selected attempt summary** reopens the document and pinned archive,
rechecks the exact cached attempt, then creates a new
`diagnostic-comparison-MEMBERINDEX-ATTEMPTINDEX.json`. Existing outputs are refused.
The summary captures manifest/member/run/archive/request identities, physical
interval/support and the actual metrics or explicit absence. Available candidate
and native reductions can be checked independently from raw SQLite/IEEE arrays;
unavailable summaries preserve absence and are not numerical oracle inputs:

```sh
python3 tools/verify_diagnostic_pressure.py /absolute/run/diagnostic-comparison-0-0.json
python3 -O tools/verify_diagnostic_pressure.py /absolute/run/diagnostic-comparison-0-0.json
```

The stdlib oracle derives norms with a maximum-magnitude scaled `math.fsum`,
independently of the Rust streaming-hypot reduction. Scaling avoids premature
square underflow or overflow when the finite L2 norm is representable. It checks source/selection/time/gauge/units/bounds and
rejects deliberately changed metrics even under Python optimization. It is a
reduction/custody check, not a replacement for `DiagnosticReader::verify`, model
accuracy or a hostile-author authenticity proof.

Timing separates process startup and prediction-call wall time (including
protocol/waiting, excluding host qualification) from execution wall time up to
finalization. Execution includes the existing100ms UI pacing and child teardown;
archive pin/final JSON costs occur afterward. These are observations, not an
isolated model benchmark or speedup claim. Counts distinguish native prefix,
produced/dispatched/qualified/rejected/unavailable and durable attempts.

## Remaining qualification boundary

Stdlib CPU fixtures and manufactured pressure solves test these runtime/storage
contracts. They do not qualify the absent measured checkpoint/Torch deployment,
physically accepted learned-provider composition, descendant process custody,
hardware GPU rendering or a bounded shutdown deadline. The exact required
measured bundle and unchanged negative accuracy observations are in
[the measured handoff](measured-pressure-bundle-handoff.md). Accepted Session and
provider/session/recording owners remain unchanged; learned production composition
requires its separate A2 gate.
