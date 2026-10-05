# Prescribed normal box flux

This slice supplies a pressure/velocity boundary interaction for one connected,
all-fluid, fixed rectangular control volume. Each complete side has a constant
outward normal speed (metres/second): negative is inflow, positive outflow.
Lower-side stored Cartesian speed is minus that value; upper-side speed equals
it. No pressure is simultaneously prescribed on those faces. It follows the
book's compatible incidence and boundary-work accounting, without changing the
accepted book sources or PDF. It is a standalone library projection primitive;
the existing closed-box `Simulation` and passive tracer stages are unchanged.

## Affine constraint and pressure

Let C be integrated outward interior-face incidence (entries ±face area), Q the
known outward boundary flux per cell, M diagonal interior-face masses rho*cell
volume, and u the provisional interior speeds. The constraint is C v + Q = 0.
Pressure solves K p = -(C u + Q)/dt with K = C M⁻¹ Cᵀ, then
v = u + dt M⁻¹ Cᵀ p. This uses the preserved `PressureOperator` coefficients and
`PressureWorkspace::solve_rhs`, including both original solver identities,
gauge cell zero and the all-cell true-residual acceptance gate. External input
samples are validated but replaced by prescribed values in RHS and output.

Known total flux is checked independently of the provisional field: a
compensated sum of six side-area/speed products must lie within
64*epsilon*sum(abs(products)). There is no mean subtraction, pressure opening,
or provisional-RHS-based relaxation. Nonzero products that underflow to zero,
nonfinite arithmetic and invalid masses are rejected. This rounding budget is a
numerical acceptance convention, not a certified exact compatibility predicate.
After correction, direct divergence uses every stored f32 face value, including
outer values and the gauge cell; a separate physical 1/second limit must pass.

## Work and ownership

Only interior unknown faces have kinetic degrees of freedom. Prescribed samples
are excluded from that kinetic sum. With R = C v + Q and
eta = M(v-u)-dt Cᵀp, the reported ledger is

E(v)-E(u)+E(v-u) = -dt pᵀQ + dt pᵀR + etaᵀv.

Boundary pressure work can raise energy. The 3×1×1 through-flow example, rho=2,
dt=0.5, speed=±0.25, starts with zero interior energy and ends at 0.125 joules;
correction energy is 0.125 and boundary work is 0.25. Declaring this projection
unconditionally dissipative would be incorrect. Stored f32 correction and
floating coefficient error are included in eta; actual divergence supplies R.
The ledger error is diagnostic and is not an additional acceptance tolerance.

The workspace owns its immutable grid/density, six original f64 pressure arrays
and one extra f64 cell array. Actual Vec capacities are capped (56 bytes/cell
when exact-sized); caller input/output arrays, allocator overhead and RSS are
excluded. It copies one immutable boundary value with caller-owned unique
identity/version per request. No allocation occurs in the solve. Inputs are
borrowed read-only; output is caller-owned disposable scratch. Cancellation or
failure after correction starts may leave output partial and pressure scratch
unaccepted. Publish output and consume pressure only with a successful report.
Retry overwrites scratch and is checked against a fresh workspace.

## Verification and limits

Eleven contract methods test signed flow on all axes and both methods, unequal
side areas against an independently assembled dense system, the affine adjoint
and gauge invariance, work accounting, zero-flux legacy bits, net-flux rejection
under enormous provisional values, actual stored divergence rejection,
cancellation/retry, iteration failure, singleton/capacity and invalid/extreme
arithmetic. A real red fixture exposed a nonzero boundary product disappearing
through underflow; rejecting that product fixes the demonstrated violation.

[AffineProjection.lean](../../../evidence/box-flux/AffineProjection.lean) contains
seven exact finite algebraic theorems and five definitions for the supplied C,
masses and residuals. It proves the work identity, compatibility, gauge-invariant
boundary work and conditional zero-flux energy nonincrease. It does not certify
Rust assembly, IEEE arithmetic, continuum convergence or geometric predicates.
See the [executed evidence](../../../evidence/box-flux/README.md).

This is uniform normal inlet/outlet data on a fixed box, with no tangential
no-slip rule. Mesh solid classification, cut cells/connectivity, translating
wall swept volume, moving-domain work, free surfaces, pressure openings,
conservative tracer transport and full time-step boundary integration remain
outside this slice. No native graphical run or isolated performance benchmark
is claimed.
