# Submerged incompressible storage and generation contract audit

Planning report, 2026-10-14. No solver, archive, desktop, provider or dataset
implementation is delivered here. Repository subject is main
`06eb5daee9e77e2b0843a5ee0a4105f6950e9b1b`. Read-only dependency subject is
[PR76](https://github.com/MrScripty/Rheon/pull/76), head
`4bcdb7f49225cd645a0934953ba7d339ff5e168d`, branch
`integration/recording-compressible-s1-20261014`. Its actual head was checked with
`gh pr view`; its source was fetched into a separate temporary repository and
extracted for inspection. Existing workspace branches/refs were not changed.
PR76 already combines recording, carrier inspection and periodic compressible S1;
the plan must depend on that reviewed integration rather than duplicate PR69/71.

## Observed authorities and limits

| Subject | Actual symbol/path evidence | Implication for this lane |
| --- | --- | --- |
| Accepted session publication | Main `src/session.rs`: `SessionSnapshot::{state,report,provider_report,frame,stage}`, `SimulationSession::{step,step_with_pressure_provider,stop}`; `docs/api/simulation-session.md` | Immutable borrowed constructor/accepted publication, actual accepted dt, no retained history. A failed step retains published state/report and consumes no accepted-step quota. |
| Identity type | Main and PR76 `src/session.rs`: `SessionIdentity { run_id: u64, source_revision: String }`, `is_valid` | Full u64 run ID is caller assigned; only revision is constrained to 1..1000 UTF-8 bytes/no controls. The prose API document's grouped identity wording must not be read as making run ID a string. |
| Provider domain | Main `src/provider_step.rs`, `src/solver_provider.rs`; `docs/api/full-step-provider.md` | Sealed constant-density full-box projection only. Pressure qualification cannot qualify later velocity/volume/body gates or a moving hull. Frozen provider provenance is separately retained; dense3D does not serialize it. |
| Durable publication | PR76 `src/recording.rs`: `StepRecorder::{create,append,finish}`, `RecordedStep`, `RecordedField`, `RecordingRun` | One local, single-writer SQLite transaction per publication; generic named fields, opaque configuration/controls, independently hashed run/step/lifecycle. No resume or checkpoint. |
| Exact historical selection | PR76 `src/recording.rs`: `RecordingReader::{open,select,select_generation,verify}` | Indexed exact ordinal/generation; complete bounded publication selection and hashing; no latest-state substitution, efficient field-only read, spatial subset or optimized time-series scan. |
| Current carrier adapter | PR76 `src/recording/carrier.rs`: `carrier_run`, `carrier_publication`, `RecordedCarrier::decode` | Fixed sealed box with exactly constructor four/accepted five fields; no fluid/solid mask or moving geometry. Only successful `Simulation::step` and `step_with_emitter` receive the required stamp. |
| Carrier pressure lifetime | PR76 `src/simulation.rs`: `AcceptedIntervalPressure`, `accepted_interval_pressure`, `prepare_impl`, `commit_prepared`, `reset` | Pressure borrows workspace scratch. Every later prepare/reset/commit attempt clears inspection authority, including rejection. Capture immediately after success. Accepted velocity/tracer retention does not preserve this scratch-pressure authority. |
| Material timing | Main `src/material_session.rs`: `MaterialSnapshot::particle_diagnostics`; PR76 `src/recording/material.rs`: `material_snapshot_publication`, `RecordedMaterial::decode` | Accepted positions/velocities differ in time from post-emission/pre-integration density/pressure/acceleration. Wall response is excluded from acceleration. These SPH diagnostics cannot supply a hull traction ledger. |
| Interchange | Main `src/vtk_export.rs`: `write_session_vti`, `write_material_vtp`, `write_particles_vtp`; PR76 adds `write_recorded_carrier_vti` and `RecordedCarrier::write_vti` | Native face arrays, cell averages and diagnostic stages stay distinct. Existing carrier VTI invents no fraction; session VTI fraction is represented liquid volume, not solid validity. |
| Compressible S1 | PR76 `src/compressible.rs`: `ConservativeState`, `CompressibleView`, `CompressibleSimulation::{view,initialize,initialize_with,step}`; `docs/api/compressible.md` | Cell-average ideal-gas periodic Euler `[rho,mx,my,mz,E]`; no compressible session/recording adapter, solid/open boundaries, viscosity, moving body or aircraft qualification. |
| Current dataset authority | Main `src/dense3d_sequence.rs`, `docs/implementation/dense3d-producer-controls.md`, `tools/export_dense3d_sequence.py` | Existing dense3D v1/v2 is a bounded fixed transport case, not a generic submerged training schema. Proposed producer v2 does not prove Tuldok consumer adoption. |

The standards subject is `MrScripty/Coding-Standards` at `188beda1`, inspected in
`/workspace/Coding-Standards`: `profiles/boundaries/persistence.md`,
`topics/contracts.md`, `topics/licensing.md`. This report assigns source/destination
authority, version support, interruption behavior, strict decoding and licensing
gates. It does not claim standards-engine certification.

## Proposed acceptance and generation contract

The following is a future submerged profile, not a reinterpretation of current
carrier or compressible data. I1's experimental Stokes solve is a stationary solve
result with its own solve identity; it must not be presented as an accepted
time-stepping trajectory. I2–I4 add time/trajectory semantics only when their
numerical owners actually support them. Production session/recording promotion
belongs to I7 after numerical admission and reviewed PR76 integration.

1. Constructor has publication generation 0, simulated elapsed time 0 and no dt,
   interval pressure, impulse/work or accepted interval controls. Initial pose,
   velocities and admitted geometry are valid constructor data. A physical epoch,
   when needed, is separate metadata; do not seed archive time with an arbitrary
   nonzero epoch that contradicts its first-step rule.
2. Each successful full interval atomically publishes native fluid fields,
   body/grid pose and velocities, geometry publication identity, actual applied
   interval controls, reports/loads, time and generation. Generation advances by
   one after the complete numerical/geometry/force/work/cancellation gates pass.
   `requested_dt` remains separate from actual dt. Internal CFL/stability retries
   get attempt IDs and diagnostics, but no publication generation or ordinal.
3. Rejection/cancellation preserves all accepted fields, controls, body state,
   geometry state, clocks and reports; the scratch candidate has no export
   authority. A future accepted snapshot must retain owned accepted pressure/load
   authority across rejected calls, or explicitly require immediate capture before
   the next call. Do not inherit PR76 scratch lifetime accidentally.
4. Capture the successful borrowed publication before another mutable call.
   Recording happens after numerical acceptance and before its corresponding
   visual preview. Recording cannot roll back physics. A recorder failure stops
   capture and further stepping in that recording workflow, disclosing accepted
   tail and recorded prefix. Reconcile an uncertain append/terminal write with a
   fresh reader; never retry a poisoned writer from guessed state.
5. Preserve PR76 adjacency exactly: ordinal starts at 0 and is writer count;
   first record is constructor generation 0 or accepted generation 1 with
   `dt == time`. Later generation must be previous+1, time strictly increases and
   `abs(time-prev_time-dt) <= 8*EPSILON*max(time,dt)`. Gaps and duplicates fail.
   Constructor is recommended for submerged datasets. Dropping frames for visual
   playback does not authorize dropping canonical accepted archive records.
6. Reset/reinitialize/configuration change creates a fresh numerical owner, new
   run ID and fresh archive filename. PR76 `Simulation::reset` increments generation
   while zeroing time; it is not a new constructor, and is refused by canonical
   carrier capture. Compressible initialization also increments generation and
   clears report; treat it as a separate run boundary for future recording.
7. Archive completion requires explicit successful `finish("completed", ..., tail)`
   and full `verify()` with the completed lifecycle/tail matching the recorded
   prefix. `active` on reopen means incomplete/unknown accepted tail. Cancelled or
   failed prefixes may be inspected but are not automatically completed datasets.

PR76 evidence anchors are `tests/recording_carrier_contract.rs`:
`rejected_cancelled_and_unsupported_attempts_invalidate_pressure_without_publication`,
`emitter_actual_reduced_interval_and_controls_are_retained`; and
`tests/recording_contract.rs`: `canonical_session_rejections_and_cancellation_do_not_publish_or_relabel`,
`terminal_write_busy_closes_admission_and_requires_cold_reconciliation`,
`poisoned_writer_cannot_claim_completion_even_at_acknowledged_tail`,
`terminal_generation_cannot_contradict_durable_tail_even_with_valid_digest`.
These were inspected as source evidence, not rerun or claimed as new test results.

## Independent identities and strict codecs

| Identity | Proposed contents/invalidation | Authority |
| --- | --- | --- |
| Run | Caller u64 plus dataset-owned globally scoped run key; source commit, binary/build/toolchain hashes recorded separately | Session/run creation; caller ID alone proves neither uniqueness nor source authenticity |
| Accepted publication | `(run key, generation)` plus exact time bits and archive ordinal; attempts use separate IDs | Numerical transaction; ordinal is archive position, not a second physical clock |
| Grid | Exact counts/origin/spacing bits, native frame enum and layout version | Numerical grid owner; a body-frame grid can move in W without changing native grid identity |
| Geometry asset | Exact source mesh/config bytes and digest, coordinate units, body axes, COM datum, representation/tessellation policy | Geometry owner; a pose change does not change immutable hull asset identity |
| Geometry publication | Asset identity + exact body/grid pose + generation + active cut/aperture representation digest when applicable | Accepted geometry transaction; topology change requires supported new publication semantics |
| Trajectory | Immutable motion law/control policy, initial conditions, coordinate convention and content digest; actual pose/control history remains captured | Trajectory owner; requested trajectory is distinct from realized dynamic motion |
| Dataset family | Source ancestor/scenario grouping, trajectory grouping, splits, selected fields, normalization and producer schema | Verified Tuldok/project consumer; related resolutions/restarts/variants remain correlated |
| Versions | Session API, solver algorithm, numerical configuration/control codec, submerged field profile, SQLite archive, VTK, dataset and learned model each independent | Each contract owner; one version number cannot stand in for all |

The proposed solver tag is distinct, for example
`rheon-submerged-incompressible-v1/<method>`. Do not use
`rheon-carrier-full-box-v1` or an Euler tag. Reuse SQLite `user_version=1` only
while its tables/digest framing/type semantics are unchanged. Introduce an
independently versioned strict submerged profile decoder; unknown profile versions,
frame variants, stage variants, required-field omissions and unexpected fields
fail closed. Generic archive acceptance proves byte/shape integrity, not submerged
physical meaning. Old canonical `RecordedCarrier::decode` must continue to refuse
these records; generic field inspection may enumerate them.

PR76 `configuration`, `controls` and `provenance` are opaque text, currently
Debug-based in adapters. They cannot replay a run. For the new profile use a bounded
versioned exact configuration/control encoding with named keys, explicit units,
unknown-key policy, finite-value rules and semantic validation. Choose that codec
and its reviewed dependency at I7; do not silently parse old Debug as a codec.
An exact initial-state/configuration artifact, licensed geometry bytes, actual
interval controls and hash-bound source/binary provenance are required for
reproducible generation. A digest alone neither embeds missing inputs nor proves
authenticity. Bind external inputs with byte counts/hashes and fail if unavailable;
never resolve geometry by an ambiguous filename.

## Native frame, sampling and proposed field inventory

Store the numerical owner's native grid and values. `native_frame` is a required
enum: `world_fixed` or `body_fixed`. A body-frame solution must never be silently
resampled into world-grid cells. Store exact per-generation grid/body pose and the
velocity representation (`absolute` or `relative_rotating`). For the I2/I3
body-frame lane store relative body velocity plus the trajectory required to
reconstruct world velocity. I4 stores native world-grid velocity. Freeze right-handed
W/B axes and gravity direction in the run; proposed W +Z is upward and B +X is
forward, subject to numerical-owner admission.

Let `R_WB` map B components to W, `x_c,W` be body COM, and `r_B` the native sample
position relative to COM. The exact publication transform is
`x_W = x_c,W + R_WB*r_B`. For relative rotating velocity,
`u_W = V_c,W + omega_W × (R_WB*r_B) + R_WB*u_rel,B`.
For absolute B components, `u_W = R_WB*u_abs,B`. Transform a force/vector with R,
a tensor with `R*T_B*R^T`, and a scalar without changing its value. Frame rotation
does not produce interpolation to a new lattice. Interpolated/resampled views
must be labelled derived, with operator, target grid, masks and error meaning.
MAC components live at different face locations: a vector transformation requires
a declared co-location/reconstruction operator when R mixes components. Preserve
the raw face components; any reconstructed world vector remains a derived value
at a declared native sample position, without implying a new world-grid solution.

The prospective inventory below describes an admitted MAC/cut-volume profile;
I1 aligned stationary geometry may require a smaller separately identified profile.
Native precision remains the numerical owner's actual precision: freeze its
f32/f64 choice before profile acceptance, preserve bits, and reject type mismatch.
Body/geometry/load metadata is f64. No downcast for storage or display.

| Field | Shape, units and association | Time/stage |
| --- | --- | --- |
| `velocity_x/y/z` | Native staggered faces `(Nx+1,Ny,Nz)` / `(Nx,Ny+1,Nz)` / `(Nx,Ny,Nz+1)`, scalar m/s, offsets `(0,1/2,1/2)` etc.; required explicit native frame and absolute/relative convention | Constructor/accepted endpoint |
| `interval_pressure` | Native cells `(Nx,Ny,Nz)`, f64 Pa, offset `(1/2,1/2,1/2)`; projection gauge and physical pressure reference/reconstruction explicitly declared by solver | Accepted projection interval, start=`evaluation_time`, end=`RecordedStep.time`; absent constructor |
| `fluid_volume_fraction` | Native cells, f64 unit `1`, actual fluid volume/full cell volume in `[0,1]` | Constructor/accepted geometry; not liquid-phase fraction or tracer |
| `open_area_x/y/z` | Native face shapes, f64 unit `1`, actual open area/full face area in `[0,1]` | Same accepted geometry as velocity; do not invent for a solver lacking this representation |
| `body_position_world` / `body_orientation_wb` | `(1,1,1)`, components 3 m / 4 unit `1`; quaternion order, unit tolerance and sign policy explicit | Constructor/accepted pose; orientation normalized/validated by owner |
| `body_velocity_world` / `body_angular_velocity_world` | `(1,1,1)`, components 3, m/s / rad/s | Constructor/accepted COM motion |
| `pressure_force_body`, `viscous_force_body` | `(1,1,1)`, components 3 N, explicitly force **onto body** and component frame | Accepted interval-average load if numerical quadrature supports it; never infer average from one endpoint |
| `pressure_torque_body`, `viscous_torque_body` | `(1,1,1)`, components 3 N m, about declared COM | Same interval/frame as force |
| `pressure_impulse_body`, `viscous_impulse_body` | `(1,1,1)`, components 3 N s | Integrated interval impulse; support requires numerical ledger |
| `pressure_angular_impulse_body`, `viscous_angular_impulse_body` | `(1,1,1)`, components 3 N m s | Integrated interval angular impulse about COM |
| `body_work`, `actuator_work` | `(1,1,1)`, scalar J; separate sign/recipient conventions | Integrated accepted interval, absent constructor |

This maximal initial inventory is 22 fields (11 constructor, with pressure/load
fields absent); optional tracer or extra diagnostics require their own declared
inventory variant within 32 fields. Missing unsupported forces/torques/work are
unavailable, never zero placeholders. Solid-cell pressure values must have a
declared placeholder policy and an explicit geometry validity interpretation;
they cannot be inspected as physical fluid pressure. Gauge handling, disconnected
fluid components, physical datum/hydrostatic decomposition and force quadrature
must be fixed by the pressure/geometry owner before loading that profile. A
full-box cell-0 gauge is not automatically valid for embedded obstacles.

Surface traction fields, if later required, need indexed surface geometry/sample
identity, surface positions/normals/areas, component frame and evaluation time;
cell pressure alone cannot establish surface loads. Do not calculate physical hull
loads by sampling PR76's projection scratch. Existing material mass×acceleration
is a derived particle diagnostic and excludes wall response.

Each grid field is x-fast; tuples/components are interleaved and encoded as native
LE IEEE754/u64/u8. Body/sample tuple associations are separate from grid associations.
Strict profile checks bind dimensions, types, units, stages, frame, masks, field
inventory, quaternion/pose, pressure meaning and interval association to that same
accepted generation. VTK world coordinates may encode rotated native grids using
a separately admitted structured-grid/geometry representation; reusing axis-aligned
ImageData spacing/origin for rotated coordinates would misrepresent the grid.

## Events, loads and wake history

Accepted `controls` metadata binds start/end generation/time, requested/actual dt,
trajectory/control-law identity, actual evaluated values, geometry before/after
identity, boundary/control versions and a bounded event array. Events carry stable
run-local event ID, kind, exact simulated time, declared frame/units, actual payload
and interval ordering; events at an endpoint have a frozen ownership convention
(proposed `(start,end]`, constructor initial declarations separate). Motion-law
changes, ascent onset, actuator changes and geometry representation changes must
not be inferred from rendered motion. Unknown event kinds fail profile decoding.
Cap event count and combined encoded text at preflight; overflow rejects the
request before physics or terminates capture after an already committed physics
step with an explicit unrecorded tail. Never truncate accepted controls to fit.
Rejected requests belong to an optional bounded host attempt log, not fake accepted
archive steps. Host monotonic timings are separate from simulated event time.

Define interval force as `impulse/dt` only if the solver reports that interval
impulse, with conversion labelled derived. A rotating body's impulse cannot be
converted between frames by rotating one endpoint vector unless the integration
convention proves that operation; record the frame/integration convention or retain
native quadrature contributions. Body reaction and fluid impulse use opposite
signs at the same boundary; prescribed actuator work is external, dynamic exchange
is internal. Separate hydrostatic, pressure and shear contributions when supported;
record torque reference point and moving-boundary work. Stationary Stokes loads
are steady solve outputs and do not imply trajectory impulses.

Wake views select exact run/generation ranges, field names, ROI/probe positions,
reference frame and actual times. Compute requested histories lazily from verified
selected records, streaming one bounded publication at a time and retaining only
a caller-capped output/probe buffer. PR76 has no optimized field-only or spatial
read; every selected step still loads/hashes its whole bounded payload. Bound work
by requested record count and cancellation checkpoints; do not claim realtime.
Body-following probes transform at each generation; world-fixed probes remain at
fixed W positions. Vorticity/streamlines/interpolated velocity/wake deficits are
derived products with operator, mask and frame semantics. Neither visual rings nor
downsampled history replace canonical accepted data. Missing records, masked probes
and unavailable fields remain unavailable; no latest-state fallback or invented
interpolation across resets. Wake visualization cannot prove turbulence or drag.

## Bytes, lazy selection and durable completion

Reuse PR76 `RecordingLimits`: default main DB 1 GiB, record <=64 MiB, default
10001 records; constructor counts. SQLite uses 4096-byte pages, FULL synchronous
DELETE journal, 250 ms busy timeout, 1 MiB cache target/connection, no mmap,
file-backed temporary storage. Main DB cap excludes rollback journal and filesystem
overhead; budget approximately twice that cap plus overhead. Cache target is not
a hard allocator cap. Selection preflights aggregate blob/text size before field
allocation. Full audit is O(records), one selected publication at a time; it can
block the writer and should run with producer paused.

For cells C and total staggered faces F, velocity/pressure raw bytes are
`w_velocity*F + 8*C`; adding native f64 fluid fractions/apertures gives
`(w_velocity+8)*F + 16*C`, before bounded body/load fields, text and descriptors.
At 64³ this is 13,778,944 bytes for f32 velocity or 16,973,824 for f64.
At 128³ it is 109,641,728 / 135,004,160 bytes, both above 64 MiB. Therefore do
not admit 128³ to this full geometry profile by citing PR76's much smaller bare
carrier payload. Preflight exact checked dimensions and bytes for the admitted
inventory. A larger profile triggers a recording-owner limits/storage decision;
no silent chunking/compression/quantization or geometry omission.

Resident memory separately accounts for live numerical state/workspace, one
encoded record, selected raw record, decoded selected arrays, derived view/probe
buffer, SQLite copies and bounded GUI caches. PR76 may retain raw+decoded for
two comparison tracks and a cold inspector; limits are per track/connection,
not aggregate RSS. Cap simultaneous submerged tracks explicitly at I7. Keep
all/stop-at-limit is the current retention contract; no unbounded queue, eviction,
automatic expiry/deletion or model-output replacement. Lazy means selection on
demand, not existing support for loading only one field.

Datasets require their own atomic final manifest, exact inputs/control schema,
file hashes/byte counts, source/binary binding and independent consumer validation.
An archive with a valid prefix or opaque controls is not a completed Tuldok dataset.
Group whole trajectories and related families before train/validation/test splits;
frames are correlated. Frozen provider models trained for a sealed box have no
submerged eligibility. Tuldok parser/project versions and accepted field/frame/
association behavior must be observed by its own integration owner before export
schema acceptance; current fixed dense3D v2 must not be extended unilaterally.

## Numerical archive, visual archive and checkpoint truth

The native accepted archive is an inspection record of exactly published fields,
time and controls. Rendered frame/video/appearance preview is a separately identified
derived artifact, bound to its source run/generation, camera/render settings,
visual reduction and interpolation. It may omit fields and generations but must
not claim numerical conservation, solver restart or physical validation.

A numerical restart checkpoint would additionally need the complete integrator
state and stage, history/multistep data, boundary/geometry state, body/actuator
state, complete initial/control definitions, adaptation/closure state and exact
configuration/build compatibility, with independent restore-and-continue evidence.
Scratch must be reproducibly reconstructible or captured under that separate
contract. PR76 explicitly omits complete integrator/emitter state. No checkpoint
codec, restored accepted generation or reproducible restart is promised by I7
recording promotion; any later request is a separately admitted write set/version.

## Future write sets and dependency gates

These are prospective exact sets, not permission to edit protected source now.
Numerical I1–I5 use their own plan-owned experiment/test/example sets; in particular
I1 is `experiments/submarine_stokes.rs`, `examples/submarine_stokes.rs`,
`tests/submarine_stokes_contract.rs`. It does not touch recording or production
`src/simulation.rs`. Recheck exact PR76 OID/contract before I7 admission.

| Future sub-slice | Proposed exact owned writes | Required gate |
| --- | --- | --- |
| I7 numerical library promotion | `src/submarine/mod.rs`, `src/submarine/stokes.rs`, `src/submarine/flow.rs`, `src/submarine/geometry.rs`, `src/submarine/trajectory.rs`, `src/submarine/moving_boundary.rs`, `docs/api/submarine-incompressible.md`; coordinated removal/replacement of `experiments/submarine_stokes.rs`, `experiments/submarine_flow.rs`, `experiments/submarine_geometry.rs`, `experiments/submarine_trajectory.rs`, `experiments/submarine_moving_boundary.rs`; same-slice import changes in `examples/submarine_stokes.rs`, `examples/submarine_flow.rs`, `examples/submarine_translation.rs`, `examples/submarine_moving_boundary.rs`, `tests/submarine_stokes_contract.rs`, `tests/submarine_flow_contract.rs`, `tests/submarine_geometry_contract.rs`, `tests/submarine_translation_contract.rs`, `tests/submarine_moving_boundary_contract.rs`; if I6 was admitted before promotion, also migrate `examples/submarine_attitude.rs` and `tests/submarine_attitude_contract.rs` in that same slice | Numerical admission for exactly the advertised I1–I4 subset, reviewed integrated recording dependency and explicit I7 admission; I5 experimental status remains explicit and does not block labeled synthetic/laminar recording; relocate only accepted kernels once, leave unavailable modules absent, retain numerical semantics, migrate every named experiment consumer in the same slice, and leave no duplicate numerical kernel |
| I7 session adapter, same numerical/API owner | `src/submarine/session.rs`, `tests/submarine_session_contract.rs`, `docs/api/submarine-incompressible.md` | Numerical library promotion above complete; immutable accepted state/control/geometry/force clocks; shared API document has one serial owner; no source edits inside existing simulation/provider owners |
| I7 archive adapter/profile | `src/recording/submarine.rs`, `tests/recording_submarine_contract.rs`, `examples/recorded_submarine.rs`, `docs/api/submarine-recording.md` | Reviewed PR76 API integrated; native precision/frame roundtrip; no duplicate recorder, SQLite schema, carrier/material adapter or VTK implementation |
| I7 independent reload/frame evidence | `tools/verify_recorded_submarine.py`, `tools/test_recorded_submarine_verifier.py` | Independent LE-bit/transform/identity checks, real producer exit/cold reload, corruption/gap/type/frame failures; generated files outside tracked tree |
| I7 bounded desktop wiring, separately admitted | `src/bin/desktop/submarine_archive_view.rs`, `tests/submarine_archive_view_contract.rs` | Exact selected generation and bounded wake/probe views; real native-window workflow required for GUI claims |
| Later generic producer contract | `examples/submarine_sequence.rs`, `tools/export_submarine_sequence.py`, `tools/test_submarine_sequence.py`, `docs/api/submarine-datasets.md` | Actual verified Tuldok consumer schema/version and trajectory/family grouping; no changes to fixed dense3D writer/importer |

`experiments/outward.rs` remains a read-only dependency during these slices. Its
future placement/visibility must be coordinated with its owner at I7 admission;
there is no assumed public interval API and no authority here to move, copy or
alter its implementation. Missing suitable public access is an admission blocker,
resolved by an explicit owner-approved placement/write set rather than duplicated
interval arithmetic.

Serial parent/integration owner alone admits and applies additive wiring at these
exact paths: `src/lib.rs` (submarine module/public exports), `src/recording.rs`
(submarine recording adapter module/public exports), and `src/submarine/mod.rs`
(the five promoted numerical modules and admitted session module/public exports).
The numerical promotion owner may prepare its `mod.rs` declarations; parent applies
the final shared wiring serially, with no overlapping edits to that file. Desktop
routing, if separately admitted, is limited to `src/bin/rheon-desktop.rs` and
`src/bin/desktop/inspector.rs`. Original solver/provider source and existing PR76
files remain protected: neither worker edits nor final integration wiring into
those paths is admitted before the reviewed recording dependency is integrated
and the parent explicitly admits I7 wiring. Existing `src/simulation.rs`, carrier/
material adapters and recording schema/algorithms stay outside this proposed set.
New adapter files can construct the public `RecordedStep`/`RecordedField` types
under I7 admission without copying or changing the archive core. A rotated-grid VTK writer
would need a separately enumerated VTK-owner write set; this report authorizes no
change to `src/vtk_export.rs`. No current `Cargo.toml`, lockfile, CI, browser,
source-admission inventory, historical receipt, commit, tag or merge mutation.

Reuse PR76's pinned `rusqlite = 0.37.0` (`bundled`,`limits`) and `sha2 = 0.10.9`;
no new storage/compression/container dependency is needed for the proposed bounded
profile. Its `docs/plans/recording-compressible-integration/reports/dependency-licenses.json`
records MIT wrapper terms for rusqlite/libsqlite3-sys, selected MIT for SHA2,
public-domain bundled SQLite 3.50.2 with source digest, and MIT/Zlib terms for
the inspected added native validation graph. This is pinned prior licensing
evidence, not an audit of future code/targets or a new legal certification.
At integration recheck actual locks/features and authoritative license texts,
preserve notices for redistribution, and verify terms of any new codec, CAD asset,
benchmark data or learned model. No GPL-only dependency or copied GPL solver code.
Compression/HDF5/chunking remains deferred; if selected later it needs an exact
dependency/license/notice and versioned consumer/durability/bounds decision.

## Remaining promotion blockers

No planning action was denied or pending. The report is complete; implementation
remains deliberately deferred. Actual moving-body numerical/geometry/load contracts,
the choice of native precision/pressure reconstruction, reviewed PR76 integration,
the verified Tuldok consumer contract, and native-window workflow evidence must be
obtained at their milestones. Existing APIs do not supply these capabilities by
renaming fields. Preserve the distinct compressible jet roadmap and its S1 scope.
