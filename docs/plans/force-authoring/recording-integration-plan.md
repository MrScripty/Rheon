# Prepared post-custody integration validation

Draft PR #145 publishes the core at 3e839cbf7c6aca56e8297ef740f7e48543de77b1.
This original preparation record is retained; the later authorized execution
on corrected head 7970a5d is documented in validation.md.
This follow-up initially added only ignored test source and this plan. It changes no core
or desktop source and executes no numerical/SQLite tests. The compiled cases
are proposals for separate review/authorization, not passed integration evidence.

## Independent change-impact review

A separate reviewer compared d7bffdd..3e839cb. Equations, acceleration bounds and
force-stage kernels did not change; success/error wrappers, Arc snapshot ownership,
accepted pressure/control custody and empty particle routing did. It recommends
one exact rerun of all five existing adapter tests against a newly sealed current
binary, under the existing supervisor, before claiming current adapter coverage:

- authored_mac_matches_legacy_bounded_directional_force
- authored_mac_cancellation_retains_accepted_state_and_retry
- authored_particle_density_conversion_adds_to_existing_acceleration
- authored_particle_cancel_retains_state_and_retry
- empty_snapshot_preserves_both_legacy_paths

No rerun has occurred. The published five passes remain pre-custody results.
No broad numerical/MMS suite is warranted by this change-impact review.

## Two smallest prepared recording cases

Both are individually ignored in tests/force_recording_integration_contract.rs.
Every helper was inspected independently without execution. Both have one frozen
force object: id=1, identity transform, global directional [2,0,0] N/m³, enabled,
start=0/no end. The alternate document changes only the name, verifying that
complete intent (including names) stays bound, not just the applied vector.

`carrier_force_archive_round_trip_and_failed_attempt_invalidation`

- One 2×2×1 grid, spacing=[1,1,1], origin=0, rho=2, initially zero fields.
- Solver retained-array cap=1 MiB, default Jacobi PCG, max pressure iterations=100,
  relative residual=1e-11, absolute residual=1e-12, predicted divergence=1e-9,
  stored divergence=1e-6, max Courant=1; no source/emitter.
- Exactly two public numerical calls, dt request=.001: one accepted authored step
  then one cancelled at BeforeCommit. No retries; <=200 pressure iterations total.
- Assert separate actual pressure/controls/work/scene custody, legacy accessor
  absence, wrong-scene wrapper rejection, and accepted state preservation with
  both pressure stamps invalidated by the cancelled attempt.
- Append constructor and accepted publications; deliberately change a third
  publication's metadata and require the exact run/document mismatch error before
  transaction. Finish failed with accepted generation1; read back durable prefix.
- Assert original run header, exactly two records, constructor pressure absent,
  full scene bytes, native numerical/pressure bits, dt/start and failed lifecycle.

`particle_force_archive_round_trip_and_rejected_control_custody`

- One 1 m cube (geometry2³, spacing=.5), exactly64 water parcels (4³), zero initial
  velocities; standard ParticleFluidConfig defaults except fill=EmittedFluid,
  samples_per_axis=4, max_particles=64, memory=1 MiB and gravity=[0,-1,0].
- No emitter/mesh. Exactly three calls: accepted dt=1e-6, changed-scene dt=1e-6
  cancelled on callback4, and invalid dt=0. At most16 preparations in the accepted
  call, at most2 entries for callback4 cancellation, and one invalid validation
  entry: <=19 prepare entries, <=17 density/force evaluations, one accepted step.
- Assert original requested dt/report/scene and evaluation-time custody, accepted
  particles/diagnostics/time/generation retained after failed calls, wrong scene,
  wrong caller dt and wrong report rejection.
- The same constructor/accepted append and third metadata-mismatch rejection
  pattern checks the durable two-record prefix, exact particle/diagnostic bits,
  evaluation time/accepted dt, original scene and failed lifecycle.

## Archive, process bounds and teardown

Each case owns one fresh temporary directory and one SQLite database; create_dir
fails rather than overwriting an existing path. RecordingLimits are database1MiB,
record64KiB, max_records=3 (two commits plus one attempted mismatch). One verify
scans the two recorded rows, followed by explicit selects0/1. No raw SQL mutation,
external fixture, model, subprocess adapter or parallel feature pool is used.

Normal teardown drops recorder/reader/simulation, audits at most3 regular files,
per-file<=1MiB and total<=3MiB, and asserts deletion of only the owned directory.
Drop retains/logs the owned directory on failure instead of deleting it.
KeepOnPanic retains outer writer/reader resources during assertion unwinding,
preventing test-owned SQLite destructors from changing the failure-boundary
recording before the child exits. Successful explicit drops/cleanup are unchanged.
A synchronous SQL operation may already have rolled back before returning an
error; this does not promise a pre-error or in-flight database image. These audits are post-run
checks, not peak resource enforcement. The owned path is logged for reconciliation
if process termination prevents cleanup.

Future execution must retain the existing process supervisor unchanged: 30s wall,
TERM then owned-child KILL after1s, CPU20s, address space512MiB, file/log1MiB,
test-threads1, no parallel feature; seal/enumerate the actual binary and select one
exact ignored test per child, serially, stopping on first failure/bound/selection
problem without tuning/retry. No execution is authorized by this plan.

## Compilation/listing only

Performed without executing test bodies:

```sh
cargo test --locked --offline --no-default-features --features force-serde --test force_recording_integration_contract -- --list
```

It enumerated exactly the two named cases above. Source review reported no
blocking defects. These cases deliberately do not cover successful legacy stamp
clearing after an authored call, historical archives, emitters, provider/session/
free-surface/UI paths, raw corruption, large workloads or scientific validation.
Existing source review and pure golden-hash evidence cover only their stated scope.

Original 1d44 preparation artifacts (superseded for authorized execution by the
later corrected-head seals):

Compile-only test source SHA-256:
`5943923df13e26acf03e9b1b0ac3d703158bd641d382471fd8e9217b7a17aea1`.
Compile-only binary SHA-256:
`28f5c0d3c398667a0049d276c132729ba4356f8371be9292aade9e4dc3c3354d`.
These identify preparation artifacts only; any future execution must recheck them.
Targeted Clippy (lib + this test target, warnings denied), formatting and diff
checks also passed without executing numerical/SQLite test bodies.


## Authorized execution preparation: failure preservation correction

The parent later authorized all five current adapter tests and both recording
cases once each under the existing process limits. Before execution, inspection
found that the original Archive Drop removed failed recordings. A reviewed
test-only correction now retains the logged owned directory on failure and
suppresses outer SQLite resource destruction only during panic unwinding.
There are no new solver calls, fixtures, process capabilities or changed limits.
The exact corrected commit/binary seals and outcomes will be recorded separately.
No cleanup of unrelated paths is permitted, and failed originals are not repaired
or rerun. This supersedes the original preparation-only execution hold for these
seven exact tests, not for other numerical/SQLite workloads.
