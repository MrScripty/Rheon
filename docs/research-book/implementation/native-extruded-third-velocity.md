# Native third velocity on the reviewed extrusion

This is the implemented successor to the frozen
[domain and derivation](extruded-third-velocity.md). Its domain is
`x in R/Z, 0 < y < H(x,t), z in R/Z`, with both periods one and every field
independent of z. The full velocity is `(u,v,w)`. The bottom is impermeable with
natural zero tangential traction; the material cap has atmospheric-relative
natural traction in the restricted weak space. There are no extrusion end walls.
The nonconstant initial third field is admissible weak data, not a classical
pointwise traction benchmark. This is a two-dimensional, three-component family.

## API and one publication

`CoupledDiscreteFlow::new_extruded` and `step_extruded` use the existing
accepted frame, candidate frame, full nodal velocity, pressure and scalar
workspaces. The state returned by `state()` contains the jointly accepted
geometry, conservative nodal masses, all three velocity components, pressure,
time and stamp. `CoupledExtrudedReport` reports the unchanged planar solve,
the third solve, both third shears and the total energy ledger.

The cross-section is still the two-column Powell–Sabin family with bottom
abscissae `[0,.5,1]`, width one, density three, dynamic viscosity `.05` and
cap-height sum `2.25`. The third field has twelve independent coefficients with
the existing scalar trace embedding. It changes neither the cap's material
equation nor the planar pressure solve because `partial_z=0`.

On the qualified native target the conservative declared budget is 470992 bytes,
including fixed caller/callee scratch reservation. The legacy zero-third budget
remains 450320 bytes. The constructor checks the opt-in budget, and each step
reuses existing scalar buffers and fixed dense factors. There is no heap
allocation during a step, second accepted owner or rollback history. These are
declared bounded reservations, not measurements of peak process memory.

All candidate geometry, velocity, pressure and reports pass their checks before
the common `BeforePublish` barrier. Publication uses the existing accepted-frame
swap and infallible pressure assignment. The old `step` refuses nonzero accepted
third velocity; it cannot silently erase it. The opt-in zero-third step agrees
bit for bit with the legacy published state on the tested sequence.

## Equations and unchanged checks

The planar nonlinear path and actual integrated relative face fluxes are
unchanged. Using their `Fplus`, `Fminus` and endpoint masses, the added solve is

    Aw xi1 = Rw^T diag(m0) w0,
    Aw = Rw^T diag(m1) Rw
       + sum_ij (Ri-Rj)^T (Fplus_ij Ri-Fminus_ij Rj) + h Kw1.

`Kw1` is the third block of the existing full symmetric-strain assembly. Its
physical loss is `h mu sum_T |T| [(w_x)^2+(w_y)^2]`; neither shear has a factor
two. Pressure has no third force. Paired periodic z-face fluxes cancel locally.
Changing-mass inertia includes `m' w`, and the independent semidiscrete reference
integrates the same conservative equation, not a scalar equation on fixed mass.

The exact finite identity, with actual residual `rw` and local GCL defect `g`, is

    Tw1-Tw0 + D_BE,w+D_mix,w+D_mu,w + sum_i g_i w1_i^2/2 - xi1^T rw = 0.

The third and full-vector ledgers are checked separately. Both stable and direct
34-row momentum rates must meet `1e-11`. The old planar Newton gate remains
`1e-13`, at most seven iterations and 200 equation calls; root/quadrature bounds
remain unchanged. Local GCL and work retain the original 128-epsilon factors.
Arithmetic continues to reject nonzero subnormal intermediates and checked
division failures. Failed candidates preserve the accepted state.

Pressure is still a freshly solved algebraic endpoint multiplier, not an evolved
pressure state or a literal mean pressure. Its weighted force approximates the
interval impulse; the multiplier itself is not that impulse. The frozen
equation-consistency limitations, including the failed pressure-rate
interpretation and the original `.00078125` bounded-Newton refusal, remain valid.

## Actual tests, replay and rendering

The eight new Rust contracts inspect bitwise snapshots of all published
velocity, physical geometry, mass, pressure, time and stamp. They exercise all
six old and three added cancellation stages after an accepted nonzero-third
step, repeated quadrature/assembly callbacks, bitwise-identical retries,
constant-third motion with changing mass, invalid intervals, wrong-API refusal,
iteration failure, an actual checked-arithmetic third-solve failure, budget and
trace refusal, both physical shears, total work, and zero-third legacy parity.
These rollback tests use the original initial planar fixture. The pressure-state
fixture is advanced and independently replayed; separate pressure-fixture
rollback coverage is not claimed. The prior review's shared original rollback
finding remains shared.

`examples/extruded_third.rs` exports actual accepted owner states: twenty
constructors and 248 steps from the two planar fixtures, nonconstant/constant
third fields, and five intervals `.05` through `.003125` to common time `.1`.
Every physical field and the actual owner stamp are mandatory in `replay.py`.
The reader reconstructs 34 momentum equations, actual integrated GCL, both
shears, separate and total work, physical pressure and geometry from those
publications. It regenerates eight bounded DOP853 references at two declared
qualities and recomputes all errors and refinement ratios. The original bands
remain 1.8–2.2 for full/third velocity and 1.7–2.3 for geometry. Thirteen actual
malformed-input controls must be rejected, including renderer-unsafe float
indices that happen to compare equal to integers.

The nonconstant third-velocity lumped L2 error decreases from approximately
`.00365951` to `.000250116` in the initial fixture and `.00369077` to `.000252321`
in the pressure-state fixture. Its successive refinement ratios approach two
(approximately 1.909, 1.952, 1.975, 1.987). These are conditional temporal results
on this fixed spatial model, not spatial or continuum convergence.

`render_native.py` makes PNG/PDF figures, a 33-publication GIF and self-contained
HTML from actual exported geometry, periodic identification and velocity. The
HTML selects recorded fixture/interval/state, draws actual microtriangles and
computes total energy and third momentum from the selected publication. It runs
no second simulator. The Node VM checker verifies all 268 states, twenty cases,
canvas coordinates/colors, selected default interval and play/wrap/pause controls.
Browser rasterization requires the installed Chromium sandbox to work; a failed
sandbox attempt is recorded explicitly, without disabling sandboxing.

No new Lean theorem is claimed. Existing conditional algebra is not a proof of
the advancing IEEE implementation, root isolation or arbitrary meshes. There
is no validated general 3D motion, continuum cap traction, variable material,
adhesion, density variation, capillarity, surface reconstruction or full liquid
simulator claim. Old Jacobi fixtures, failed readers and numerical refusals remain
frozen and separately identifiable.
