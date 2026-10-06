# Coupled discrete-work refinement: arithmetic repair and native qualification plan

The prior source `31f8614fe169153f92f7d75889330b3504e083ea` and evidence
`aa89d1af95f84d86e4fb58f09c523035187b2006` remain frozen, including their failed
fourth refinement, pressure-state coarse diagnostic, every aborted trial and
all preceding numerical/proof limitations. The owner resumed this research;
no acceptance allowance is raised. The temporal method remains the explicit
first-order endpoint donor momentum discretization using physical path-integrated
face mass. It does not become exact continuous momentum transport.

## Reproduced Newton floor and algebraically identical repair

Observation-only instrumentation of the frozen solver reproduces refusal at
fine step five with `h=.00625`: after two corrections the rate norm is about
`2.01e-13`, then oscillates above the unchanged `1e-13` stopping target. Merely
replacing `M1 z1-M0 z0` by

    R^T [m1 R(z1-z0)+(m1-m0) Rz0]

does not resolve that floor; this failed trial is retained. Both expressions
are still independently measured against the original `1e-11` physical gate.
The stable inertia form remains in the successor to avoid subtracting two large
momenta, but its success is not claimed as the cause of the repair.

The reviewed six-coordinate chart prescribes cap `(ux0,ux1,uy0)` and selectors
`(z[0],z[1],z[4])`. On this embedding those are the exact one-hot coordinates
`z[2],z[3],z[12],z[0],z[1],z[4]`. Integrated strong divergence plus constant
volume implies `z[13]=-z[12]`, the other cap normal velocity. Eliminate these
seven coefficients exactly before solving the remaining fifteen. Select fifteen
independent strong rows of the initial restricted matrix and solve

    D_unknown(q) z_unknown = -D_known(q) z_known,
    D_unknown(q) a_unknown = -(Ddot z)_selected-D_known(q) a_known.

Here `z_known` is assigned from eta without solving for already-known values;
`a_known` is assigned from alpha. The remaining velocity/divergence/cap
requirements are not discarded: **all 24** strong velocity and differentiated
rows, actual material cap motion, geometry and pressure image are checked at
every recorded quadrature evaluation and both endpoints. The symbolic frozen
row identities and volume relation justify the exact known values. The new
fifteen-row solve remains conditional on nonsingularity and those full checks;
no mesh-family or IEEE theorem is imported. The six accelerations and sixteen
pressure unknowns still solve the same 22 finite momentum equations.

The repaired lift completes the exact frozen refused input and the full sixteen
fine steps with the original stopping/iteration/resource bounds. Thread count
is recorded; the frozen refusal also reproduces with the same single-thread
setting used in the repeated qualification. Pressure gauge, mass provenance,
positive/negative face splitting and all work terms remain unchanged.

## Pressure-state coarse rate diagnostic

Let `delta=C_endpoint-C_actual_path`. With smooth local motion and stable donor
signs on faces with `f(0)!=0`,

    delta = h^2 L + O(h^3),
    L = R^T route(f(0),Udot(0))/2.

A face with zero initial flux contributes zero to L, including when its sign
changes later. Initial shear has **exactly zero** relative physical flux and
zero L, explaining the leading cubic difference. On the pressure-state field,
L is nonzero; its exact maximum component is approximately `5.64708e-5`.
The old `.05/.025` ratio `4.629` includes a substantial cubic contribution and
still fails the originally declared 15-percent band around four. That frozen
coarse flag stays false. New refinements measure `delta/h^2` approaching the
independently derived exact coefficient, and test the **same** band on finer
ratios. This is a conditional asymptotic explanation and measured consistency,
not a retrospective pass for the failed coarse test or a change in the physical
momentum equations.

## Complete temporal qualification and remaining proof scope

Run five intervals `.05,.025,.0125,.00625,.003125` to common time `.1`, on both
the initial shear and preceding nonzero-pressure field. Preserve every accepted
research step incrementally in a bounded output packet, so a failure retains
its preceding replay. Two independently refined coupled ODE references resolve
the measured errors. Replay every finite equation, physical mass balance,
separate BE/mixing/viscous loss, endpoint pressure, geometry, clock and stamp;
corrupt those quantities meaningfully. Repeat cancellation/failure after a
nonzero accepted state and resume the same owner. Normal/optimized execution
and independent rational/high-precision checks supply numerical evidence.

The host owner retains one accepted tuple and one candidate; exported histories
are explicit bounded evidence, not hidden simulation snapshots. Python allocator
bytes, native arithmetic/rollback and production memory limits are not proved.
Numerical sign discovery plus 16/32 refinement is not a sign-isolation theorem.
Full constraint/rank checks on evaluations are not a whole-interval or arbitrary
mesh theorem. Continuous momentum and actual integrated varying-donor momentum
remain measured against their old gates and reported as different equations.
No new Lean statement, continuum spatial/pointwise traction convergence,
component positivity, variable density, adhesion, capillarity or reconstruction
claim is introduced.

## Selected native owner/workspace slice, before implementation

After freezing the qualified temporal checkpoint, implement a bounded opt-in
native counterpart on this two-column periodic height graph, fixed bottom,
constant density/viscosity and invariant zero third velocity, with full xy
pressure/strain. Every component that can become nonzero in this slice is solved.
Reject unsupported topology, field support, geometry/path or arithmetic rather
than extending the model silently. General legacy liquid stepping keeps its
existing refusal until its own model and publication contracts are satisfied.

Reuse `TranslatedViscousFlow`'s accepted fitted frame, full nodal velocity,
candidate buffers, common clock/stamp and final `accept_ale` publication pattern.
Add only a validated crate-private initialization path for full xy accepted
fields; public translating-flow restrictions must stay intact. One additional
fitted frame is candidate scratch, as in `FixedBottomAleFlow`. The coupled
wrapper owns pressure coefficients tied to the same accepted frame/time and
publishes them at the same final barrier, never as a separately advancing
carrier/phase authority. Derive cap coordinates and chart coordinates from the
accepted frame/velocity rather than retaining another geometry state.

Allocate bounded fifteen-row chart, 22-row nonlinear Jacobian, face quadrature,
pressure and force arrays once; count every vector capacity and fixed scratch
payload in the memory budget. A step must allocate no heap arrays or accepted
snapshots. Use the existing nonfinite/nonzero-subnormal/checked-division guards,
bounded linear solves and true residual checks. Rank/full-divergence, actual
material mesh derivative, physical face mass, finite GCL, original/direct
momentum, all discrete work terms and endpoint pressure adjoint are final gates.
Cancellation and any failure, including overflow, resolution, quadrature or
iteration failure, must preserve accepted geometry, masses, velocity, pressure,
clock and stamp bit for bit.

Qualify actual Rust formatting, feature test matrix and strict Clippy, focused
release arithmetic/transaction tests, numerical replay against the qualified
host method, repeated temporal refinement and rendered accepted endpoints before
claiming a public advancing step. The native counterpart remains implementation
work until these checks pass. No paid service, credential or network bypass is
required. Parent coordinates reviews, PRs and merges; source/evidence checkpoints
remain separate and frozen.

## Observed successor results

Both fields complete 2/4/8/16/32 accepted steps to `.1` for the five intervals,
62 steps per field. The velocity error ratios approach two:
initial shear `1.9772,1.9885,1.9942,1.9971`, pressure state
`1.9764,1.9881,1.9940,1.9971`. Finest lumped velocity errors are respectively
`2.20120e-6` and `2.31627e-6`; the two reference refinements differ by
`8.13e-14` and `7.09e-15`. Geometry ratios likewise approach two.

Across these actual normal trajectories the largest discrete work/allowance
ratios are `.00943` and `.01325`, local GCL ratios `.01809` and `.01946`, and
finite momentum rate norms below `9.88e-14`. These observations are numerical
qualification against the same spatial DAE, not continuum spatial accuracy.
Clock drift from repeated floating additions is retained, never clamped.

Each first `.05` path has an actual exact-real 512-box rank/geometry certificate.
Initial/pressure lift Neumann bounds are at most `.044227/.044106`; pressure
basis bounds at most `.552603/.551069`. Microareas stay above `.0381404/.0381381`
and quality above `.0750682/.0750709`. The certificates cover these two first
coarse polynomial paths only, not every repeated path, IEEE evaluation or an
arbitrary geometry family. The attempted wrong-wrapper certificate input and
render implementation failure are retained separately from successful runs.

The replay render shows the 32 actual accepted finest endpoints of each field,
with separate first-order error, BE/mixing/viscous loss and gate-ratio plots.
It is explicitly a host research render. The native implementation starts only
after this source/evidence checkpoint is frozen.
