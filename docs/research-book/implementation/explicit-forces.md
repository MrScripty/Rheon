# Implementation addendum: explicit body forces

The expanded book's Chapter 17 orders transport and explicit forces before
moving solids and liquid interfaces. Chapter 19 separates acceleration from
force density and derives finite-step work. This milestone implements that
bounded dependency in the Rust fixed-box core. The published chapter/PDF bytes,
root proof project, prior inventories and historical evidence remain unchanged.
This addendum is separate from the retained reading edition.

## Per-step API and units

`Simulation::step_with_forces(dt, smoke_source, &[BodyForce], cancel)` accepts
caller-owned force descriptions for one step. `ForceUnits::Acceleration` takes
m/s². `ForceUnits::ForceDensity` takes N/m³ and divides by the simulation's
constant density in kg/m³. This adds neither a liquid material model nor
spatially varying density. A force changes velocity, not tracer amount.

An optional `ForceRegion` is a finite world-space box wholly inside the grid.
It includes lower planes and excludes upper planes. Each vector component is
sampled at that component's own MAC face position; this is a piecewise-constant
prescribed field, not interpolated mesh data. Overlapping vectors add in slice
order before one stored update. The caller must express all values in the same
world frame. Normal wall faces remain at their prescribed zero speed.

The sequence is velocity advection, legacy smoke acceleration, explicit body
forces, pressure projection, tracer transport/source, diagnostics, publication.
The existing `step` delegates with an empty force slice, retaining its original
callback sequence and output format. Pressure implementation IDs and defaults
are preserved. The headless CLI and desktop comparison keep their existing
smoke controls; the reusable library and example expose this new feature.
`StepStage` gains BeforeForces/ForceSlice, and `SimulationError` gains
InvalidForce; downstream exhaustive matches must accommodate the new variants.

The old-speed travel budget and half-budget forcing policy now include a
conservative sum of absolute external accelerations divided by axis spacing.
Disjoint and canceling forces can reduce dt conservatively. The policy is not
a stability theorem. Reports identify accepted dt and time. Force descriptions
are borrowed for the call; no force arrays or simulation capacities are added.

## Work and publication

`ForcedStepReport` contains the unchanged `StepReport` and an optional
`ForceReport`. Empty forces have no external stage/report. A nonempty force
stage reports its input/output kinetic energy and applied work using the core's
existing uniform `density * cell_volume` face weights. These are the energies
after advection/smoke forcing and before projection, not complete-step energies.
Pressure and transport can subsequently change energy.

For stored old and new face speeds, let δu=new−old. Work sums
`mass * δu * (old + δu/2)`. It uses the actual f32-stored increment, including
rounding; a force below storage resolution can produce zero reported work.
The exact-real identity is ΔE=mass·δu·(old+δu/2). Signed work can be negative.
Using old-velocity power alone omits the finite-step squared-increment term.
Binary64 reductions remain approximate and are not a conservation certificate.

All force inputs are checked before candidate advection. Nonfinite vectors,
invalid/out-of-box regions, conversion/velocity overflow and unresolved time
increments return explicit errors. Cancellation at BeforeForces or a ForceSlice,
pressure failure, and later rejection leave accepted arrays/time/generation
unchanged. A retry overwrites candidate scratch. Reports publish only with an
accepted step; a renderer still reads the immutable accepted state.

## Executable acceptance and formal scope

The numerical fixtures use hand-computed finite-step work on anisotropic,
translated grids, acceleration/force-density equivalence, half-open face
support, and a four-edge unit-square graph. Its divergence-free cycle is
c=(1,−1,−1,1). From u=(1/2,0,0,0), projection is c/8, independently establishing
the expected nontrivial circulation for both pressure methods.

A fully filled stationary sealed box projects uniform acceleration back to
rest through a hydrostatic pressure gradient. This is a fixed-box check, not
a free-surface tank, liquid-interface, moving-wall or calibrated gravity test.
Additional fixtures exercise negative work, f32 rounding, forcing travel
limits, cancellation after partial force updates, rejection/retry and unchanged
managed-array memory. Original PNG/CSV fixtures are replayed with the legacy CLI.

The additive `evidence/explicit-forces/ForceWork.lean` module defines finite
kinetic energy and stored work, proves their equality, lifts Chapter 19's
acceleration work identity to finite fields, proves force-density momentum
scaling, and proves cancellation of signed acceleration by a prescribed
hydrostatic gradient. It is checked against the pinned Lean 4.19.0/mathlib
project with a separate explicit axiom audit; it is not added to the historical
root inventory. These real-arithmetic statements do not prove Rust face
assembly, IEEE execution, pressure convergence or continuum physical validity.

```sh
cargo test --locked --no-default-features --test forces_contract
cargo run --locked --release --no-default-features --example forced_smoke
cargo run --locked --release --no-default-features --example forced_smoke -- sgs-pcg-v1
```

The example emits step/time/force-work/energy/divergence diagnostics for uniform
gravity plus a localized three-component force-density drive and passive tracer.
Moving/collision geometry, variable material density, viscosity, liquid volume,
wetting, slip and capillarity remain later dependencies with separate acceptance.
