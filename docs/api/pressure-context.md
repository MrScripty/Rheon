# Pressure context and trusted CPU adapters, version 1

`rheon::solver_provider::context` is an additive borrowed physical request contract.
It does not execute a provider, advance a SimulationSession or certify upstream
advection/viscosity composition. Existing PressureProvider and accepted recording
contracts retain their owners. The desktop comparison is an independent diagnostic
consumer; its raw AI states are never accepted Session publications.

## Physical context

`PressureContext::new(PressureContextInput)` validates actual GridGeometry, positive
finite density/dt, finite nonnegative interval start and strictly advancing end,
finite full-cell RHS, nonempty bounded run identity and positive generation. The
stage identifier is `sealed-mac-pressure-context-v1`. Full sealed MAC operator
semantics, Cartesian geometry and x-fast cell ordering come from PressureOperator.
RHS and residual have units m³/s²; pressure has Pa and canonical first-cell zero.
`initial_residual(output)` computes `b − A*p0` with the authoritative full operator.
Absent history uses an explicit zero initial guess; it does not invent solved
pressure. A two-cell analytical stencil independently checks its sign and scaling.

Prior PressureHistory binds borrowed values, exact grid/density, run ID, solved
positive generation/end time and Accepted or Diagnostic { qualified } status.
History must be the immediate same-run predecessor with matching interval end.
Only generation one at time zero can omit it. AcceptedCandidate rejects diagnostic
history. Caller-declared lineage proves coherence, not authenticity or acceptance
by another owner. The diagnostic Engine supplies its own committed prior and
retains failed-quality status explicitly; cancellation before commit does not
advance history. Run IDs currently distinguish engines within the process; they
are not durable saved-run identities or authentication.

## Inert package selection

The desktop accepts existing closed `rheon-pressure-cpu-deployment-v2` bundles and
new closed `rheon-pressure-adapter-v1` JSON manifests through the same selector.
**Check package support** verifies material and scene support without executing
adapter code. Starting a run repeats preflight before spawning Python. Unknown
fields/versions, duplicate inventory/input keys, bad hashes, path escapes, special
files and unsupported physical contexts fail explicitly. Hashes identify bytes,
not authors, scientific quality or the complete interpreter dependency closure.

A v1 manifest requires these fields (the fixture tests construct complete examples):

| Field | Contract |
| --- | --- |
| schema, id, version, stage | Exact schema/stage; bounded identifiers |
| python | Absolute executable path, SHA256 and byte length |
| entrypoint, files | Package-relative entrypoint; unique relative inventory keys with SHA256/length for every declared file |
| support | Exact density_kg_m3, dynamic_viscosity_pa_s, dt_s; counts_min/max, max_cells, spacing_min/max_m; origin `zero`, cells `cubic` |
| context | minimum_native_steps and prior_pressure `required` or `optional` |
| inputs | Named ports mapped to rhs, previous_pressure, initial_residual, x, y, z, density or dt |
| output | role `full_pressure` or `pressure_correction`; dtype `f64`, layout `x_fast`, gauge `cell_zero` |

Correction requires prior pressure. Selecting a prior-pressure input also requires
available history. Support does not widen the existing scene envelope: sealed
free-slip water, rho=1000 kg/m³, mu=0.001 Pa·s, dt=1/60 s, zero origin, cubic cells.
Unequal axis counts are supported. New physical semantics require qualification;
changing adapter tensor channels, normalization and captured files stays in the
package when these declared contracts hold.

Budgets: manifest 1 MiB; at most 64 files, 256 MiB each/512 MiB total; interpreter
128 MiB; at most 32 ports/65,536 cells/512 minimum native steps. Canonical regular
files and inventory containment are checked. This is trusted local execution:
the operator must exclude concurrent package/interpreter replacement between
verification and use. Canonical paths and isolated Python flags are not a sandbox.

## One supervised protocol owner

Existing ModelProcess launches the pinned interpreter with `-I -S -B`, a cleared
CPU-oriented environment and the declared entrypoint/manifest. The adapter owns
model loading, tensor preparation and normalization. It sends a closed ready line:

```json
{"kind":"adapter_ready","id":"analytic-full","version":"1","manifest_sha256":"<64 lowercase hex>","device":"cpu","optimizer_updates":0}
```

Each newline-delimited request uses schema `rheon-pressure-context-request-v1`,
kind `predict`, a correlated integer id and the stage identifier. Its context has
run_id, generation, mode, interval_start_s, interval_end_s, dt_s, grid
(counts_xyz, spacing_xyz_m, origin_xyz_m) and nullable prior lineage/status.
Each named input has semantic, units, association (`cell` or `global`), dimensions,
scalar `f64`, layout `x_fast` and finite values. Density/dt are one-element globals;
XYZ are physical cell-center coordinates. The adapter replies:

```json
{"kind":"candidate","id":16,"role":"full_pressure","pressure":[0.0,1.0],"device":"cpu","optimizer_updates":0}
```

Role must match the manifest; pressure length/finiteness and reply identity are
checked. The host adds prior pressure exactly once only for pressure_correction.
Numerical qualification still runs after protocol decoding. Analytic full and
correction stdlib fixture packages prove real independent process/port/output-role
selection without rebuilding host code; they are not trained-model qualification.

One existing worker owns the child, bounded writer queue and I/O threads. The
30-second prediction budget includes queued stdin writing and reply waiting;
cancellation is polled during both. Drop disconnects the queue, attempts immediate
child kill/wait and joins owned I/O. Real tests cover a blocked stdin writer and
cancel/restart after predictions. OS cleanup remains best effort; no descendant
custody, sandbox or guaranteed hard shutdown bound is promised.

## Recording and qualification limits

Legacy checkpoint identities retain raw JSON frame-v2 for the pinned lab reader.
Generic packages emit frame-v3 with provider identity and null checkpoint; no
checkpoint/training provenance is invented. Binding metadata is versioned
`rheon-provider-binding-v1`. Neither is accepted SQLite archive-v1 or a typed
saved-run-set implementation. Cold accepted analysis and diagnostic persistence
remain separate contracts.

Fresh automated tests use actual stdlib CPU children and headless egui rendering.
The measured trained deployment/Torch is absent here; its explicit runtime gate
remains pending. A subsequent real X11 window on workspace-local Xvfb software
rendering exercised inert support, visible failed diagnostics, cancellation,
restart completion and Close during prediction with immediate-child liveness
observations. This is fixture/software-display evidence, not trained-model or
hardware/GPU qualification. Retained trained
checkpoint accuracy failures are not converted into passes by protocol tests.
