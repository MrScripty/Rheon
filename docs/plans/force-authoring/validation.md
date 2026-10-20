# Force authoring validation and review record

2026-10-20; Rust 1.92.0; isolated branch based on main fd7c0f0.
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

## Prepared integration tests: not executed

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
