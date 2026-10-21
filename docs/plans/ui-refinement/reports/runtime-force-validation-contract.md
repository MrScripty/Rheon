# T5 bounded desktop validation contract — proposed, not executed

Status: source integration is gated. This document specifies the next exact cases;
those two numerical/SQLite test bodies are **not yet implemented or enumerated**.
It is not an executable admission, workload authorization or runtime result. The
current executable probes are the seven pure tests in force_runtime_ui_tests.rs.
No force-enabled production launch is admitted by source-only evidence.

## Ownership and prerequisites

Use canonical backend core at43ec505218c43594641ecbb80873bec8c88d7d2c; fetched
backend a4326a78a747bc868874fd16e434d780fd5a6d35 differs only in its own admission
review/inventory. Preserve legacy hashes. UI owns Request/carrier dispatch,
Inspector begin/record, frozen desktop intent/binding and retained presentation;
backend remains owner of force evaluation and recording/accepted solver custody.

Before execution, implement the two separately ignored cases in a separate desktop
test module, independently inspect their complete helpers and enumerate exact names.
Seal the final commit/tree, complete affected-source SHA256 manifest, Cargo.lock,
Rust version/feature configuration, test-source bytes and exact compiled binary.
The current197-name binary does not contain these cases and must not be represented
as execution-ready integration evidence. Compile/list-only does not step a solver.
No tuning, retry or fallback suite is implied by this contract.

The execution owner must revalidate its existing supervisor and fresh bounded
recording filesystem. Do not provision mounts/cgroups, change delegation or reuse
a previous output root. If an existing suitable bound is absent, return the exact
missing prerequisite. Existing live cancellation stays parked; this contract does
not replace its containment requirements or supervise a native UI session.

## Common bounded execution envelope

One exact ignored case per disposable process, serially, libtest threads1. Desktop
features include rayon: use the production run_owned scope with Request.threads=1;
this makes one owned Rayon pool worker, rather than claiming parallel is disabled.
No Worker::start, GUI/event loop, model/provider, child command, emitter, collision
mesh, observer, progress/failure gate or public network input is present.

Require the execution owner's already supported controls before dispatch:

- wall30s, CPU20s, address space1GiB; TERM then owned process-group KILL after1s;
- per-file1MiB and aggregate fresh root8MiB, aggregate captured output1MiB;
- root created once without replacement by the supervisor, exclusive to this case;
- no automatic success/failure/panic/timeout teardown or cleanup; retain database,
  sidecars, manifests, stdout/stderr, source/binary seals and disposition;
- enumerate all retained files/bytes and SHA256 after the child terminates; check
  no owned descendants remain. Do not infer measured peak resources from limits.

The test helper must accept an explicit absolute supervisor-owned root with no
fallback to temp_dir or normal recording_directory. It must not delete existing
paths or use a Drop cleanup guard. Every path is inside that fresh root. Stop on
any unexpected selection, file, resource-bound failure or timeout; preserve evidence.
These are maximum requests, not promised observed consumption.

## Case1 — proposed exact name

`force_runtime_integration_tests::native_force_dispatch_records_and_cold_binds_v2`

A single JacobiPcgV1 native carrier track, grid2×2×2, spacing[0.5;3]m,
origin[0;3], density2kg/m³, zero initial velocity/tracer, one requested interval
0.000001s. Frozen tolerances: relative1e-11, absolute1e-12, predicted divergence
1e-9/s, actual divergence1e-6/s, Courant1, max pressure iterations100; retained-array
limit1MiB. No source/emitter. Request preview edge2, history budget16MiB,
recording max database1MiB/record65536bytes/records2, steps1/threads1.

Canonical scene version1 has one enabled force: id1, name“Desktop custody probe”,
translation[0.5;3]m, identity xyzw quaternion, Acceleration, Directional vector
[0.25,0,0]m/s², Global support, activation[0,∞). No configured gravity is inserted
by this test; native carrier has its existing zero-gravity inputs.

Freeze once. Build native FrozenSpec v2 and one-member RunSetRevision v1 with no
prior attempt; bind the same Some snapshot in Request. Permit only the private
cfg(test) force_validation flag, preserving every other preflight. Through existing
run_owned → run_simulations → CarrierExecution::advance and Inspector optional
constructor/record wrappers, request exactly ONE step (max100 pressure iterations).
No retry loop around the public request. Native saved-intent fixed-dt policy still
rejects a shortened dt and retains the exact accepted archive.

Expected evidence: one result row and progress1; two ordered in-memory frames at
constructor/accepted times; source binding, actual accepted pressure controls and
same force metadata on constructor and accepted records. Open final immutable member
through RunSetRevision/open_member, verify pin/hash/status Completed, count2,
generation1, exact stored force bytes and spec/declaration/run binding. Mutate a
separate draft after freeze; it must change neither archive nor retained snapshot.
Existing startup/finished manifests and database remain. This proves dispatch and
custody for this tiny case, not field physics, legacy equivalence or UI interaction.

## Case2 — proposed exact name

`force_runtime_integration_tests::material_force_dispatch_records_immutable_scene`

One material track with same2×2×2 preview/grid geometry in a1m cube. Water at
293.15K; fill EmittedFluid, samples_per_axis4, max_particles64, initially exactly64
particles, zero velocity, no emitter. Default water material and Air ambient,
ambient293.15K, gravity[0,-1,0]m/s², liquid sound speed20, wall heat exchange1,
retained-array limit1MiB. Freeze the same enabled canonical force document as Case1.
One requested interval1e-6s, at most16 internal preparation/retry attempts; no
external retry, pressure solver or max-pressure-iteration loop in this path.
Other Request/recording/preview limits match Case1. No saved native set is claimed
for material; immutable Request and version1 core force metadata own this path.

Through existing run_owned → run_material, exactly ONE public particle step, same
snapshot for begin/step/record. Expected progress1, one result row, constructor and
accepted previews using the actual accepted state/time, cold core reader verification and
RecordedMaterial force document matching exact run-bound bytes, accepted dt/emitter/
report controls and generation1. Assert preview/record time and actual dt association;
particle preparation may shorten dt, so do not assert a fabricated literal time or
necessarily the requested1e-6s. Both records must remain associated with that
snapshot after separate draft mutation. Preserve database/sidecars and metadata;
no automatic cleanup. Do not describe this as a material frozen-rerun format that
does not exist in the current application.

## Subsequent gates, deliberately separate

After the reviewed exact bodies pass once under separately admitted execution,
review their evidence before removing the temporary production gate. Native active
force launch, progress/Cancel/join, retained-versus-draft gestures, long status,
keyboard/accessibility, mixed saved members and frozen rerun still need their own
bounded evidence. Do not silently add those workloads to the above two-case budget.
The current no-force workerless synthetic harness cannot establish any of them.
Existing actual8148fa2 native validation remains owned by the current validator;
it does not cover this T5 source. JPEGs must come from actual rendered exact-source
states, labelled with revision and state, quality85, confirmed Library identity.
