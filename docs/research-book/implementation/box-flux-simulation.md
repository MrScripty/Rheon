# Prescribed box flux in accepted simulation steps

`Simulation::step_with_box_flux` advances the existing owned smoke fields with
one copied `BoxFluxStepBoundary` and caller-owned `BoxFluxStepWorkspace`. This
extends the [standalone affine projection](prescribed-box-flux.md) into actual
transactional time stepping on a fixed, connected, all-fluid rectangular box.
No mesh wall, moving domain, cut cell, free surface or tangential no-slip rule is
introduced. Boundary identity/version uniqueness remains caller-owned.

## Discrete boundary and transport choices

Velocity advection traces the previous accepted velocity with the unchanged
midpoint/clamped sampler. Its temporary external face values remain zero; only
interior provisional speeds enter forcing and projection. Projection replaces
external values with the requested end-of-step normal speeds and uses the
unchanged pressure operator and selected original PCG implementation. This is
an end-of-step boundary assignment, including when the boundary changes between
calls, not a time-continuous actuator history or moving-wall volume rule.

The pre-step travel policy includes both previous speed and the requested side
speeds; forcing shares the displacement budget as before. Actual stored face
divergence (every cell, including gauge) and final Courant limits must pass.
The independent net side-flux gate rejects incompatibility without mean shifts.
The box volume and density stay fixed. Reported dt*net outward volume flux is a
numerical imbalance diagnostic, not an evolved mass state or exact certificate.

Tracer transport uses the projected end velocity. Callers must explicitly select
`BoxFluxTracerPolicy::ClampedAppearance`: the existing bounded midpoint sampler
extends nearest sample values across the box/sample support at inflow, without a
specified reservoir concentration. It is appearance concentration transport,
not a finite-volume conservative scalar update. A source is added afterwards
with the existing saturation rule. The report separates transport integral
change and actual source integral change; no density or physical smoke mass is
silently inferred from concentration units.

In the 3×2×1 fixture, the source first seeds the left column at concentration 0.5.
A subsequent dt=0.25 step has balanced x through-flow speed 0.25 and localized
positive y acceleration 0.5. Its tracer transport integral grows by
0.0625246912240982, whereas an endpoint upwind boundary update using old adjacent
concentrations would add 0.0625. This measurable difference illustrates the
missing boundary-flux balance contract. Balanced fluid volume and bounded scalar
values alone do not imply scalar conservation.

## Complete step work and ownership

Energies on this opt-in path count only interior unknown faces, each weighted by
rho*cell volume. `StepReport.kinetic_energy` equals that interior accepted energy;
legacy methods keep their original diagnostic convention. Let A be the measured
advection energy change, S the stored legacy smoke-force work, F the external
stored force work, and use the affine projection's Q, residual R and momentum
error eta. The complete report checks the diagnostic ledger

Eaccepted-Eold+Ecorrection = A+S+F-dt pᵀQ+dt pᵀR+etaᵀv.

A is measured, without an advection-dissipation assumption. S uses actual energy
endpoints; F uses the existing finite-step force report. Pressure boundary work
can raise energy. Numerical reductions produce a reported budget error; it is
not an extra rejection tolerance or continuum energy certificate.

The independently capped workspace owns the preserved affine pressure arrays
plus three provisional face arrays: nominal capacity 56*Ncell+4*Nface bytes.
Actual Vec capacities are capped. The simulation retains its original closed
pressure workspace and allocation, reported separately; the combined retained
array payload is the sum of both reports. Caller budgets must account for both.
No step-time heap allocation occurs. Geometry, density bits and solver identity
must match before any stage callback. Immutable accepted inputs do not alias the
provisional or candidate arrays. Neither pressure nor provisional scratch is
published as accepted state.

All candidate velocities/tracer, time, generation and reports publish together
only after every force, pressure, physical divergence, Courant, tracer/source,
finite-ledger and before-commit gate passes. Cancellation can leave candidate and
caller scratch partial. Retrying overwrites it and is checked against fresh
execution. Workspace reuse, pause/reset, signed boundary changes and late source
failure are covered. Default force/tracer-barrier methods retain their original
closed-box path, capacities and callback sequence; calling those methods requests
that closed-box model. This slice does not combine mesh tracer barriers with the
new inlet/outlet method.

## Formal and executable scope

Twelve integration fixture methods exercise signed repeated steps on all axes,
boundary speed dt control, version changes, smoke/external work, appearance
integrals, all stage cancellations and retry, incompatible flux/iteration/late
source failures, mismatched workspaces, exact capacity/pause/reset, zero-flux
legacy fields, stored-divergence rejection and the scalar conservation limit.
Original deterministic demos are replayed through the legacy CLI.

The additive StepBudget.lean imports identical preserved AffineProjection.lean.
Seven exact finite theorems and two definitions compose stage work with the
projection ledger and prove conditional energy nonincrease. For uniform cell
volumes it separates row normalization (constant preservation) from column
balance (scalar-sum conservation), including a normalized two-cell counterexample.
These are algebraic hypotheses; they do not prove Rust interpolation weights,
IEEE execution, pressure convergence or continuum validity. The historical proof
inventory and accepted PDF inputs remain unchanged. See the
[executed evidence](../../../evidence/box-flux-step/README.md).
