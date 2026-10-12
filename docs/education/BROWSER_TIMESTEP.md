# Native timesteps in the browser: scope and teaching plan

The Rust core already implements fluid timesteps. The missing part at the
audited teaching checkpoint
[`24573051aa6768d253cec40aeeb576cf34a058ec`](https://github.com/MrScripty/Rheon/commit/24573051aa6768d253cec40aeeb576cf34a058ec)
was a browser adapter, build path and runtime qualification. The existing
JavaScript sampling/transport lessons and strain operator did not call those
timesteps. This is an integration boundary, not evidence that the native
physics is absent.

The next lesson uses the existing four-cell force/projection fixture through a
separate [adapter](../../browser/solver/src/lib.rs) and
[adapter manifest](../../browser/solver/Cargo.toml), built by
[`build_solver.py`](build_solver.py). Its scope is one accepted step from rest,
plus cancellation and retry. This document states the source contract and
qualification requirements; it is not a browser execution receipt. The
[coverage ledger](COVERAGE.md) records integrated chapter placement separately.

## Existing core and integration boundary

| Existing source | Concrete capability | Boundary that the browser must preserve |
| --- | --- | --- |
| [GridGeometry](../../src/geometry.rs) | Checked `new([u64; 3], spacing, origin)`, named component indexing, singleton dimensions and `BufferPlan::for_grid` | Counts convert to target `usize`; WASM32 has a smaller index domain. Validate before allocation. |
| [Simulation](../../src/simulation.rs) | `new`/`with_implementation`, `step`, `step_with_forces`, `state`, `set_paused`, `reset`, `allocated_bytes` | The owner starts at rest. Accepted fields cannot be seeded or mutated through `StateView`. `reset` clears fields/time and increments generation. |
| [StepReport](../../src/simulation.rs) | Actual `dt`, time/generation, pressure diagnostics, independently checked stored-velocity divergence, Courant number, tracer integral and kinetic energy | Requested time and accepted time can differ. `StateView` contains f32 x/y/z velocities and tracer, but no pressure field. No pressure API change is needed for this lesson. |
| [BodyForce and ForceReport](../../src/forces.rs) | Component-lattice acceleration or force density, optional half-open support, external-stage work from the actual f32 velocity change | Force density is divided by constant density. External-stage work is not the energy change of the complete projected step. Normal wall velocities stay fixed. |
| [PressureWorkspace](../../src/pressure.rs) | Existing Jacobi-PCG and symmetric-Gauss-Seidel-PCG implementations, residual/acceptance reports and typed failures | A successful solve/report is not an iteration history. The fixture does not qualify arbitrary grids, condition numbers or convergence rates. |
| [LiquidTransportSimulation](../../src/liquid_step.rs) | Owns a carrier, represented-volume state and accepted pressure; `step`, `step_with_box_flux`, restricted `step_viscous`, immutable `state` | Carrier, pressure, fractions, clocks and stamps publish together. Full-box fractions do not classify pressure. Opt-in surface modes have separate, restricted admission rules. |

`Simulation::step` delegates to `step_with_forces` with no external forces.
The staged implementation transports velocity, applies declared forces, solves
pressure, corrects velocity, checks actual divergence and Courant number,
transports tracer, then publishes. Errors and cancellation preserve accepted
field bits, time and generation; candidate scratch may change. The
`BeforeCommit` callback occurs after preparation and before publication.

The [core manifest](../../Cargo.toml) has no external dependencies with default
features disabled. PNG export and the desktop GUI are optional and are not
required by the adapter. The unchanged library can be checked with:

```sh
cargo check --lib --no-default-features --target wasm32-unknown-unknown
cargo test --no-default-features --test forces_contract \
  --test simulation_contract --test liquid_step_contract
```

The three native contract suites passed all **25 tests** during this audit.
A target check establishes compilation, not a callable JavaScript interface or
browser behavior. The core manifest alone supplies neither an exported WASM
interface nor JavaScript glue. Transitive `wasm-bindgen` entries associated with
optional GUI dependencies in the lockfile do not supply such an interface.

The adapter must own the Rust simulation and expose bounded commands and
copied accepted snapshots. Borrowed slices and generic Rust cancellation
closures are not a JavaScript API. Values must cross the boundary with explicit
units, errors and integer semantics; general u64 identities cannot silently
become imprecise JavaScript numbers. Retaining views into WASM memory across
allocation or memory growth also needs an explicit lifetime policy.

The separate adapter's `CycleSimulation::new(method_id)` selects either
existing pressure implementation. `snapshot()` returns copied JSON state;
`step(dt, cancel_before_commit)` returns an outcome, optional report and
accepted snapshot; `reset()` follows the native reset semantics. The fixed
geometry, force and settings match the source fixture. The wrapper rejects any
`dt` other than 0.25 and a second accepted step before reset. It includes f32
field bits, hexadecimal time bits and an exact decimal generation string for
comparisons. Its additional dependencies are pinned `wasm-bindgen 0.2.105`,
`serde 1.0.228` and `serde_json 1.0.145`; these belong to the adapter, while the
core dependency remains `default-features = false`.

This small synchronous step can demonstrate cancellation selected in advance
at `BeforeCommit`. That does not demonstrate interrupting a running solve from
a later browser event: an event handler cannot run concurrently on the same
blocked JavaScript thread. General interactive cancellation, long simulations
and larger allocation limits require separate integration and timing checks.

## One accepted force/projection step

The oracle is the existing test
`localized_force_projects_to_the_independently_derived_four_edge_cycle` in
[`tests/forces_contract.rs`](../../tests/forces_contract.rs). It uses a 2×2×1
unit-cell grid at the origin, density 1 kg/m³, initially zero fields, and
requested `dt = 1/4 s`. An x acceleration of 2 m/s² acts in the half-open region
`[0,2) × [0,1) × [0,1)`. There is no smoke source. This excites one internal
x face; wall-normal faces remain zero.

Order the four internal edges as x-bottom, x-top, y-left, y-right. With unit
face masses, the forced field is `u* = (1/2, 0, 0, 0)` and the divergence-free
cycle is `c = (1, −1, −1, 1)`. Exact projection onto that one-dimensional space
gives `c (u*·c)/(c·c) = c/8`.

| Accepted face sample | Prediction, m/s |
| --- | ---: |
| X at `[1,0,0]` | +0.125 |
| X at `[1,1,0]` | −0.125 |
| Y at `[0,1,0]` | −0.125 |
| Y at `[1,1,0]` | +0.125 |

The force-stage work is `1/8 J`; accepted kinetic energy is `1/32 J`.
Projection removes the gradient part while retaining circulation. All z
velocities and tracer samples remain zero. The native test checks the four
velocities and final energy against their hand values within `1e-8`, and runs
both pressure implementations. Its pressure settings are relative residual
`1e-11`, absolute residual `1e-12`, predicted-divergence limit `1e-9`, maximum
1000 iterations, actual-divergence limit `1e-6`, and maximum Courant number 1.
An adapter that narrows these settings or selects one implementation must
state that narrower scope.

The useful interaction is **predict → step → compare**, with a separate
**cancel before commit → inspect unchanged state → retry** path. Compare all
accepted f32 field bits, time bits and generation on cancellation, then compare
the retry with a fresh successful run. The supporting native tests are
`force_cancellation_and_pressure_exhaustion_preserve_state_and_retry` and
`cancellation_at_every_stage_preserves_all_accepted_bits_and_metadata` in the
[force](../../tests/forces_contract.rs) and
[simulation](../../tests/simulation_contract.rs) suites.

Draw returned face samples with a fixed arrow scale and show actual time,
generation, force work, energy, residual diagnostics and independently
recomputed divergence. A diagram of `u*` is the labelled analytical prediction,
not an exposed intermediate Rust state. Scrolling may change explanation or
view, but must not invoke stepping or reset; live state remains intact between
reading stops. Camera motion changes only the view.

The fixture exercises a real Rust timestep, but starts with zero velocity, so
its first advection stage transports zero. It does not measure transport
accuracy. Repeating the step invokes nonzero advection and invalidates `c/8` as
a repeated-step oracle. A reset or new owner is required before rerunning the
one-step comparison; creating a new owner starts generation zero, whereas
`Simulation::reset` increments the existing generation.

The thin closed box is not a free-surface liquid, a resolved vortex experiment,
a turbulent flow or a moving-wall model. The force-stage work and final energy
should remain separate readouts. Passing this fixture does not establish
cross-runtime bit equivalence for arbitrary inputs, whole-browser memory
limits, general solver performance or physical calibration. No new Lean claim
follows from compiling the Rust code to WASM.

## Existing liquid follow-on, not included in the first adapter

`signed_axis_transport_publishes_hand_pressure_and_synchronized_states` in
[`tests/liquid_step_contract.rs`](../../tests/liquid_step_contract.rs) is a
small next candidate. It already calls
`LiquidTransportSimulation::step_with_box_flux`, with an independently owned
`BoxFluxStepWorkspace`, rather than requiring new physics.

Three unit cells start with fractions `[0,1,0]`. Carrier density is 1000 kg/m³;
represented density is 800 kg/m³. With prescribed speed `+0.25 m/s`, zero inlet
fractions and `dt = 0.5 s`, the accepted fractions are `[0,0.875,0.125]` and the
accepted pressure is `[0,−500,−1000] Pa` in the test's gauge. Reversing speed
gives `[0.125,0.875,0]` and reverses the pressure slope. Represented volume stays
1 m³ and mass 800 kg; both state clocks reach 0.5 s. The existing test covers
each axis, both signs and both pressure implementations.

This is conservative represented-volume transport in a full-box carrier.
Fraction is an amount; drawing a filled subcell would introduce an interface
placement that this mode does not provide. It has no two-phase inertia or
general interface reconstruction. Surface/column modes are separate, and the
static flat-column MAC mode explicitly refuses dynamics. The existing
`accepted_actual_dt_controls_volume_interval_and_closed_step_is_coherent`
test also demonstrates requested 20 s becoming accepted 0.125 s when cell
width is 0.5 m and imposed speed is 2 m/s. Browser admission would need its own
snapshot, boundary-workspace, stamp and failure comparisons before this could
be called an implemented lesson.

## Bounded visualizations for the remaining theory

These are recommendations, not implemented coverage or new numerical results.
The preceding sampling/transport edition had 12 chapters without a linked
example: 1, 2, 8, 9 and 11–18. The fixed Rust lesson adds chapter 17 and the
subsequent bounded 3D volume adds chapter 14, leaving 10 chapters without a linked
example. None of these counts implies complete topic coverage. The first column
below preserves all 12 proposals; chapters 14 and 17 now have partial additions.

| Chapter and concept | Control and visual | Prediction or measurable check | Limit and source |
| --- | --- | --- | --- |
| 01 Evidence and accepted state | Select accepted, cancelled and retried one-step snapshots; align field values and state IDs | Cancellation changes no accepted bits; retry matches a fresh run | Lifecycle comparison only; no guidance or long-horizon accuracy claim. [Simulation contracts](../../tests/simulation_contract.rs). |
| 02 Units and nondimensionalization | Change positive length, speed, kinematic viscosity and density; show units, `Re = UL/ν`, `L/U`, and force-density/acceleration conversion | Equivalent `f = ρa` inputs give equal accelerations; changing units must preserve dimensionless groups | Algebraic scaling, not a flow prediction. Native force-unit equivalence is separately tested in [force contracts](../../tests/forces_contract.rs). |
| 08 Affine boundary work | Toggle the prescribed flux vector Q in the existing three-cell metric fixture; show both sides of the adjoint identity | The homogeneous identity matches; omitting the affine work term leaves exactly `pᵀQ` (0.21 for the source fixture) | Fixed finite matrices, not cut-cell assembly or moving geometry. `affine_boundary_adjoint` in the [depth companion](../research-book/companion/depth_experiments.py). |
| 09 Amount versus interface location | Move an axis-aligned filled interval inside a unit cell at fixed fraction; show its overlap with an outgoing slab | At fraction 0.3 and Courant 0.5, left/right placement gives outgoing amounts 0/0.3; uniform-fraction flux is 0.15 | Exact interval intersection, not arbitrary PLIC reconstruction or surface dynamics. `plic_axis` in the [depth companion](../research-book/companion/depth_experiments.py). The native liquid follow-on above is a separate next step. |
| 11 PIC/FLIP invisible mode | Two equal-mass particles with velocities ±1 deposit to the same grid sample; vary PIC/FLIP blend β under zero grid update | Grid velocity is zero; returned particle velocities are ±β, so retained energy fraction is β² | Frozen transfer only; no particle trajectories, pressure, APIC or SPH dynamics. [Complete PIC/FLIP sequence](../research-book/chapters/11-particle-methods.md#complete-pic-and-flip-update-sequence). |
| 12 Temporal error | Select Euler or midpoint and 10/20/40/80 steps for a prescribed rotation over the same final time | Compare the computed endpoint to `(cos 1, −sin 1)` and report endpoint error as step count changes | ODE backtracing, not spatial interpolation error or a coupled fluid step. `rotation` in the [depth companion](../research-book/companion/depth_experiments.py). |
| 13 Allocation arithmetic | Select bounded grid counts and payload cap; draw named velocity/tracer/pressure allocations | `8F + 56N` bytes for the base plan, where F is total face samples and N cells; reject a cap below the required payload | Distinguish the plan from actual retained Vec capacity and exclude JS copies, WASM pages, renderer, allocator and RSS. [BufferPlan](../../src/geometry.rs), [capacity contracts](../../tests/simulation_contract.rs). |
| 14 Guidance as measurement | Change constant extinction, ray length and subdivision count; draw segment contributions to optical depth | With segment lengths preserved, subdivision leaves `α = 1 − exp(−ΣσᵢΔsᵢ)` unchanged | Analytic opacity measurement, not scattering, liquid depth or normals. [Chapter 14](../research-book/chapters/14-guidance-rendering.md) and the bounded [+Z exporter](../../src/export.rs). |
| 15 Finite arithmetic | Toggle binary64 calculation and explicitly rounded binary32 replay on the same pressure-identity fixture | Report absolute discrepancy, stored-velocity divergence and normalization denominator separately | Rounding a pressure vector is not running binary32 CG; historical values retain their source identity. `projection_experiment` in the [companion](../research-book/companion/experiments.py), [chapter 15](../research-book/chapters/15-floating-point.md). |
| 16 Hypotheses and counterexamples | Vary the upwind Courant number while showing the applicable theorem premises | For left value 0 and current value 1, `c = 3/2` produces `−1/2`; the positivity premise `c ≤ 1` is then false | Numerical illustration and exact source link, not browser Lean execution or proof of a runtime kernel. [Transport.lean](../../proofs/Rheon/Transport.lean). |
| 17 Ownership and publication | Run the fixed Rust step, cancel before commit, retry, and explicitly reset | Compare four returned faces, work/energy, unchanged cancellation snapshot and generation behavior | The one-step scope above; scrolling never advances or resets the owner. [Simulation source](../../src/simulation.rs). |
| 18 Validation at a fixed endpoint | Compare existing transport refinements over one full period, with analytic reference and error bars/readouts | Report RMS error and observed error ratios using equal physical travel; state when a zero error makes a ratio undefined | Empirical evidence for this prescribed scalar problem, not universal order or production performance. [Transport kernel](transport_reference.js), [independent comparisons](test_transport_reference.py). |

Appendix A can reuse the actual cancellation/publication trace while keeping its
pseudocode distinct from the compiled API. Appendix E can align its exact cycle
arithmetic with the four returned Rust faces, preserving the different fixture
identity of the older 27-cell blend. Appendices B and D benefit from inspectable
source/build identities and units; C needs source links, not a simulator.
Appendix F can map a selected claim to its exact hypotheses and source without
turning the map into a proof-coverage claim.

The smallest coherent progression is therefore the existing Rust cycle and
accepted-state controls first, then finite boundary-work, amount/placement and
particle-transfer mechanisms with independent arithmetic checks. A native
three-cell liquid lesson would follow only after its separate browser
qualification. General pressure interfaces, surrogate training, moving meshes,
arbitrary liquid surfaces and new solver architecture are outside this plan.
