# Force authoring validation and review record

2026-10-20; Rust 1.92.0; isolated branch based on main fd7c0f0.
The initial milestone record below is retained; the later authorized execution
and custody results at the end supersede its originally unexecuted status.
This record reports code-level checks, not scientific validation or performance.

## Executed pure checks

The helper paths in tests/force_field_contract.rs are local constructors,
freeze/evaluate calls and scalar assertions. No Simulation or particle solver
constructor is called there. Internal unit paths were inspected: the bound test
only evaluates fields; the three force-work tests update small local face arrays
through the lattice kernel and do not call advection/pressure/timestepping.

Tests were enumerated with:

```sh
cargo test --locked --offline --no-default-features --features force-serde --test force_field_contract -- --list
cargo test --locked --offline --no-default-features --features force-serde --lib -- --list
```

Each of these twelve tests ran separately with the exact command below,
substituting one listed name for NAME (all passed):

```sh
cargo test --locked --offline --no-default-features --features force-serde --test force_field_contract NAME -- --exact
```

- directional_units_superposition_and_density
- enabled_and_half_open_time_use_static_samples
- frozen_run_owns_original_identity_and_values
- invalid_disabled_objects_are_also_rejected
- invalid_schema_ids_and_count_are_rejected
- nonfinite_inputs_overflow_and_tiny_distance_are_safe
- radial_and_vortex_normalization_handles_subnormal_and_huge_vectors
- radial_sign_is_outward_and_centre_is_zero
- serialization_roundtrip_preserves_ids_and_requires_validation
- support_boundaries_and_smoothstep_weights
- translation_rotation_and_local_box_support
- vortex_handedness_axis_and_rotated_axis

Each of these four inspected unit tests ran separately with this exact command
(all passed):

```sh
cargo test --locked --offline --no-default-features --features force-serde --lib NAME -- --exact
```

- force_field::tests::rotated_subnormal_fields_respect_acceleration_bound
- forces::tests::authored_field_face_sampling_density_walls_and_work
- forces::tests::finite_step_work_can_be_negative_and_uses_both_velocity_endpoints
- forces::tests::work_accounts_for_actual_storage_rounding_and_closed_wall_faces

Thus 16 exact pure/unit tests passed. The last two are existing compatibility
checks. No broad execution filter or full suite was used. Local execution script
and full output: /tmp/rheon-force-pure-tests.sh and /tmp/rheon-force-pure-tests.log.
The initial offline enumeration lacked registry packages; one locked online build
downloaded dependencies using existing settings, then checks proceeded offline.
No credentials or network configuration were changed. Cargo.lock is unchanged.

## Independent source/equation review

A separate review agent, force_review, read the domain, both solver adapters,
existing force kernel and the new tests without editing or executing any tests.
It reviewed quaternion transforms, handedness, support/falloff, unit conversion,
snapshot ownership, MAC face boundaries/stored-velocity work, particle density,
force staging, retry/commit and legacy routing.

The reviewer identified direct hypot normalization of multi-component subnormal
offsets as capable of exceeding the declared magnitude bound. The implementation
now scales directions before normalization and includes relative/absolute
rounding allowances in the conservative bound. New pure regressions cover
subnormal offsets/strength and very large finite offsets. The reviewer rechecked
the correction and reported no remaining blocking correctness findings.

The reviewer also noted that hosts should cap JSON byte input before parsing;
freeze bounds validated data but does not bound deserializer allocations.

## Prepared integration tests: initially not executed

Every test in tests/force_field_solver_contract.rs is individually ignored with
a review/execution-authorization reason. They were compiled/listed only:

```sh
cargo test --locked --offline --no-default-features --features force-serde --test force_field_solver_contract -- --list
```

The independent reviewer inspected all helper paths and bounds. MAC helpers use
2×2×1 cells, constant rho=2, 1 MiB retained-array budget, pressure iteration cap
100, requested dt=0.001. Particle helpers initialize exactly 64 particles,
1 MiB, no emitter/mesh, requested dt=1e-6, and the existing at-most-16 retry loop.
Each simulation requests at most one accepted step; cancel/retry checks use
separate bounded instances. No files, subprocesses, native UI or model processes
are requested by these tests.

| Exact test | Planned assertion |
| --- | --- |
| authored_mac_matches_legacy_bounded_directional_force | Exact stored state/work agreement for equivalent local box and legacy region |
| authored_mac_cancellation_retains_accepted_state_and_retry | Accepted state retained at three cancel stages; retry agrees |
| authored_particle_density_conversion_adds_to_existing_acceleration | Pre-integration diagnostic difference equals evaluator with local density |
| authored_particle_cancel_retains_state_and_retry | Cancellation preserves accepted particles/time; retry agrees |
| empty_snapshot_preserves_both_legacy_paths | Empty snapshot agrees with legacy grid and particle paths |

These are ready for parent review, not an assertion that solver integration has
passed. Execution, if later authorized, must use one exact test name and
`-- --exact --ignored`, never a broad suite. No L1/MMS, native simulation, model
workload, scientific result or hardware/performance claim was produced.

## Static checks

Passed without simulation execution:

```sh
cargo clippy --locked --offline --no-default-features --features force-serde --lib --test force_field_contract --test force_field_solver_contract -- -D warnings
cargo check --locked --offline --no-default-features --lib
cargo fmt --all -- --check
git diff --check
```

The integration test's initial compile-only attempt exposed an invalid
SimulationConfig Default assumption in its helper. The helper now supplies the
existing configuration fields explicitly; subsequent compile/list and Clippy
passed. This did not execute a solver. Desktop/parallel builds and full suites
were not run as part of this bounded backend task.


## Later authorized bounded adapter execution

Parent explicitly authorized one execution of each of the five enumerated tests,
serially, from the already compiled binary under supported process limits. Source
and helper inspection reconfirmed the fixture/retry/iteration bounds before any
execution. The pre-custody source was d7bffdd5d6e80e022f15c59d3574739e3b39b352
(code identical to 19e0145; d7bffdd adds documentation only).

The existing binary was copied without rebuilding, hashed and enumerated. Cargo's
saved fingerprint reports only feature force-serde, with no parallel feature.
Binary SHA-256:
`0042e68eb0de3f63e3d4ef98506a88f417a4f965f63c41b64890f4ee9e9003cd`.
Full tracked source/Cargo/toolchain manifest SHA-256:
`29179208bf0def784bf22404d0cef6f8333e2481f5727a424cd372ba24d530d8`.
Test source SHA-256:
`9efdf712baaa8e662e77339a81074751ed10ef01b9675f99351fac2a662b41fc`.

Actual harness commands:

```sh
python3 /workspace/force-validation-pre-custody/run_bounded.py enumerate
python3 /workspace/force-validation-pre-custody/run_bounded.py execute
```

Enumeration executed `force_adapter_tests --list --format terse` and required
exactly the five expected names. Each actual execution command was the sealed
binary path `/workspace/force-validation-pre-custody/force_adapter_tests`, one
exact test name listed above, followed by:

```
--exact --ignored --test-threads=1 --nocapture
```

Each child checked/applied hard+soft CPU=20 s, address space=536870912 bytes,
file size=1048576 bytes and disabled core files before exec. Parent waited at most
30 s, with TERM then owned-child KILL after a 1 s grace on timeout. Stdout/stderr
went directly to a unique regular log subject to the child's file-size limit.
The harness rechecked the binary hash before each process, required exactly one
selected passing test and stopped on failure/timeout/signals/selection mismatch
or a full log. No tuning/retry occurred. No cgroups, mounts, security configuration,
models or adapter subprocesses were provisioned.

All five returned code 0, selected exactly one test and passed once. No timeout,
resource-limit signal, unexpected selection or bound failure was observed. Log
sizes in the planned execution order were 178, 183, 191, 173 and 167 bytes. These
are observed harness outcomes, not a performance or scientific validation claim.

Full source manifests, fixture seals, exact command/result JSON, raw logs,
feature fingerprint and the executed harness source are preserved under
`evidence/pre-custody-adapters/`. The large compiled binary itself is retained
locally under /workspace/force-validation-pre-custody, not committed.

These results apply to the pre-custody code only. No solver test was rerun after
accepted custody/recording changes, honoring the one-execution authorization.
No post-custody SQLite/native/desktop workload or full parity test was executed.

## Custody codec and source checks

Seven new tests in `recording::force_fields::tests` were source/helper inspected,
compiled and enumerated with:

```sh
cargo test --locked --offline --no-default-features --lib -- --list
```

Each then ran separately from the compiled library test binary
`/workspace/.rheon-tools/target/debug/deps/rheon-84cc08b87dcd124f`, with exactly
one name below and `--exact --test-threads=1`. All seven passed:

- recording::force_fields::tests::binary_rejects_truncation_trailing_unknown_versions_and_invalid_objects
- recording::force_fields::tests::binary_roundtrip_preserves_all_variants_ids_disabled_and_float_bits
- recording::force_fields::tests::field_schema_limits_and_accepted_custody_fail_closed
- recording::force_fields::tests::legacy_run_hash_and_absent_metadata_are_unchanged
- recording::force_fields::tests::malformed_run_digest_never_panics_and_is_not_rebound
- recording::force_fields::tests::maximum_valid_scene_fits_bounded_codec
- recording::force_fields::tests::run_binding_rejects_absent_mismatched_and_unbound_metadata

All helpers use in-memory documents/fields/hash fixtures only, with no solver,
SQLite, filesystem fixture, subprocess or native UI. A separate review agent
inspected these paths without executing them. It confirmed the writer/reader
association checks, success-only accepted controls, MAC stamp invalidation and
unchanged legacy hash/serialization branches. It identified an avoidable +0
addition for empty particle fields; that evaluation is now elided while accepted
metadata retains the empty document. No remaining blocking findings were reported.

Pure fixture golden hashes independently calculated from the historical byte
contract are run `781b179b9818a048d50183f33244e73968dc7ec02e35a258f5bba3463886986b`
and step `579f8cc614789a49c128f54c90ed449e6bd09a35fda8b7c6958b449ceef39b76`.
This is fixture evidence, not exhaustive historical archive validation.

Post-custody static checks passed:

```sh
cargo check --locked --offline --no-default-features --lib
cargo check --locked --offline --no-default-features --features force-serde --lib
cargo clippy --locked --offline --no-default-features --lib -- -D warnings
cargo fmt --all -- --check
git diff --check
```

The exact local pure invocation script/output are /tmp/rheon-custody-pure.sh and
/tmp/rheon-custody-pure.log. Remaining runtime/intent/archive UI work is specified
in custody.md. Desktop, request definitions and session/provider owners are
untouched; no publish/merge/deployment occurred.

Post-custody source/Cargo/test seal is evidence/custody-source.sha256; its SHA-256
is `d873dd98d5d59d1f1aac2221cb5f1f30ae574bd6dff91c95d95d82a0c927a9ca`.
It deliberately differs from the pre-custody runtime test seal; the runtime
results must not be attributed to this newer source.

## Authorized post-custody validation on corrected head 7970a5d

The parent authorized the five existing adapter tests plus the two independently
reviewed recording cases once each, serially, with the existing process bounds.
Before running, source inspection found cleanup-prone failure handling in the
recording tests. Test-only commit
`7970a5db8136c3771ea416db86c58753aea95566`, based on authorized
`1d44c8429701f205f678b602040d4f62ea6457b3`, corrects failure preservation.
No src/** or adapter-test source changed from 1d44. Independent review confirmed
unchanged helpers/call counts/limits and no blocking finding before execution.

Archive Drop now retains/logs owned directories on failure. KeepOnPanic suppresses
outer writer/reader destruction during assertion unwinding, while successful
explicit closes/cleanup remain unchanged. This preserves test-boundary failure
files, not a pre-error/in-flight image: API-internal rollback/constructor cleanup
may already have happened before returning an error. Signals also leave the
owned directory. No unexpected test failure occurred in this run, so there are
no failed originals to reconcile. No unrelated cleanup was performed.

Build only (no Cargo test execution):

```sh
cargo test --locked --offline --no-default-features --features force-serde --test force_field_solver_contract --test force_recording_integration_contract --no-run
```

Both binaries were copied and sealed; saved Cargo fingerprints report only
force-serde, with no parallel feature. Exact seals:

```
adapter_tests:
ceea9adb9d9023107e7211ca4792d87d6fb7f3d4cc0b6e06f60f313ad1dca773
recording_tests:
4ee715678545204a996b70121d26dcb930578e7e5f94cafc27d111372af69e5b
full source manifest:
26cce65fa43bab619cd0951dbc523945c37fd0bc955c06b3451448ca82cc5277
adapter test source:
9efdf712baaa8e662e77339a81074751ed10ef01b9675f99351fac2a662b41fc
corrected recording test source:
e9ca49fdeb0f474cdcddb40930c3773dc71762a709ac7dec6200fd5db31859a7
```

Artifacts are local under /workspace/force-validation-post-custody-7970a5d.
Source seals, feature fingerprints, exact executed harnesses, command/result
JSON and byte-preserved logs are committed under evidence/post-custody-7970a5d.
The large binary copies are local only. Binary hashes were rechecked before each
child. The supervisor's bounded() and run() function text is byte-identical to
the earlier supported harness; only paths, binary seals, exact names/count and
libtest output flag were configured. No supervisor redesign occurred.

Exact harness invocations (all inherit the explicitly owned scratch root):

```sh
TMPDIR=/workspace/force-validation-post-custody-7970a5d/scratch python3 /workspace/force-validation-post-custody-7970a5d/adapters/run_bounded.py enumerate
TMPDIR=/workspace/force-validation-post-custody-7970a5d/scratch python3 /workspace/force-validation-post-custody-7970a5d/recordings/run_bounded.py enumerate
TMPDIR=/workspace/force-validation-post-custody-7970a5d/scratch python3 /workspace/force-validation-post-custody-7970a5d/adapters/run_bounded.py execute
TMPDIR=/workspace/force-validation-post-custody-7970a5d/scratch python3 /workspace/force-validation-post-custody-7970a5d/recordings/run_bounded.py execute
```

Enumeration used each binary's `--list --format terse` and required exactly its
five/two expected names. Each adapter invocation was the sealed adapter_tests
path, one exact name and `--exact --ignored --test-threads=1 --nocapture`.
Each recording invocation used recording_tests, one exact name and
`--exact --ignored --test-threads=1 --show-output`. The latter keeps successful
name/result lines verifiable while retaining captured owned-path output.

| Exact test | Result | Log bytes |
| --- | --- | ---: |
| authored_mac_matches_legacy_bounded_directional_force | 1 passed; exit0 | 178 |
| authored_mac_cancellation_retains_accepted_state_and_retry | 1 passed; exit0 | 183 |
| authored_particle_density_conversion_adds_to_existing_acceleration | 1 passed; exit0 | 191 |
| authored_particle_cancel_retains_state_and_retry | 1 passed; exit0 | 173 |
| empty_snapshot_preserves_both_legacy_paths | 1 passed; exit0 | 167 |
| carrier_force_archive_round_trip_and_failed_attempt_invalidation | 1 passed; exit0 | 477 |
| particle_force_archive_round_trip_and_rejected_control_custody | 1 passed; exit0 | 471 |

Each child applied/checked hard+soft CPU20s, address space536870912 bytes,
file/log1048576 bytes, disabled core files, test-threads1 and no parallel pool.
Parent wall timeout was30s with TERM then owned-child KILL after1s. No timeout,
signal, unexpected selection or observed bound failure occurred; there was no
retry/tuning. All seven result/log SHA-256 and byte counts were verified after
execution without rerunning tests. No cgroup/mount/security setting was changed.

Recording cases used one fresh explicitly owned directory/database each, with
RecordingLimits database1MiB/record64KiB/max_records3 and two committed rows.
Each verified its expected mismatched third append rejection before transaction,
failed-prefix lifecycle, actual custody and exact readback. Carrier used four
cells, one accepted + one cancelled call, <=200 pressure iterations. Particle
used64 parcels, one accepted + one callback4-cancelled + one invalid call,
<=19 preparation entries. After all assertions passed, only those owned
recordings were successfully torn down. The task's scratch root was verified
empty afterward; no unrelated path was removed. Existing post-run file audits
passed; they do not replace process peak-resource enforcement.

A preflight Clippy collapsible-if finding in the new test guard was fixed before
sealing/execution; no runtime failure was tuned or retried. Targeted Clippy with
warnings denied, formatting and diff checks passed. This evidence establishes
bounded current adapter/recording behavior at 7970a5d, not scientific L1/MMS,
calibrated physics, desktop parity, emitter/provider/free-surface/session paths,
successful legacy stamp clearing after authored steps or exhaustive historical
archive compatibility. No additional numerical/SQLite tests were executed.
