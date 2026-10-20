# Fixed P1 necessary-screen protocol (v1)

Frozen before numerical campaign execution. Authoritative full acceptance remains
[PR120 verification](https://github.com/MrScripty/Rheon/blob/8422f77bb45e60b95c5694ed0b39e01377b62976/docs/plans/fluid-accuracy-composition/reports/verification.md#particle-wall-and-equilibrium-gates).
No pass here admits production; failures are retained and return to selection.
This screen cannot establish the full plan's 3D native pressure/shear/wall gates.

## Versions and sources

Compare `aha2012-linear-wendland-planar-screen-v1` and
`mayrhofer2013-k0-si-mu-planar-screen-v1` against identical water/geometry.
[Adami–Hu–Adams 2012](https://doi.org/10.1016/j.jcp.2012.05.005),
eqs 6–10, 22–23 and 27, motivates dummy wall extrapolation.
[Original article text](https://www.academia.edu/4728278/A_generalized_wall_boundary_condition_for_smoothed_particle_hydrodynamics)
is the inspected primary article reproduction. [Mayrhofer et al.](https://arxiv.org/pdf/1304.3692),
eqs 37–38 (k=0), 18 and 20, motivates analytical wall integrals and the
native near-wall linear viscous law. Neither adaptation reproduces the whole
published method: kernel, EOS and integration differ; artificial viscosity,
density diffusion, damping and background pressure are absent. Source references
justify comparison, never Rheon equivalence.

Use binary64 SI: rho0=998.2 kg/m³, mu=.001002 Pa s, g=9.80665 m/s², c=32 m/s.
Wendland C2 support **R=2h**, W=21/(2 pi R³)(1-q)^4(1+4q), q=r/R.
No lattice renormalization. Kernel/moment source:
[Dehnen–Aly 2012](https://arxiv.org/pdf/1204.2471).
Its application evidence does not validate water. Also compare frozen preview
poly6 density/spiky viscous stencil in the 3D instantaneous consistency study.

The necessary planar reduction is a column per unit tangential area, mass kg/m²,
volume m, force/area Pa and energy/area J/m². Exact tangential kernel marginal
K(s)=3/(2R)(1-|s|/R)^5(1+5|s|/R+8s²/R²),
K'(s)=-2 pi s W(|s|); phi=-K'/s=2 pi W(|s|).
Analytical gamma and its derivative are exact integrals of this kernel over the
admitted half-space/channel. Gamma is not the discrete particle partition.

AHA fluid V_i=m_i/rho_i. Boundary quadrature volume per area stays its initial
spacing; pressure/density are extrapolated, the source freezes this volume in continuity. Fixing it also in
pressure and viscosity here is an explicit adaptation, not a source-exact claim. Boundary extrapolation uses K weights, physical prescribed velocity
in continuity, 2*u_wall-mean(u_fluid) in viscosity. Pressure uses symmetric
(V_i²+V_j²)*pbar. Record pressure/continuity EOS work defect; pair momentum
alone does not prove energy. SA k=0 uses V-weighted pressure extrapolation,
exact gamma surface terms in pressure and continuity, and dynamic-mu viscosity
divided by rho_i for SI acceleration. Wall slope is (u_i-u_wall)/distance,
measured from native particles; no analytic profile slope is supplied.
Record SA pair momentum/adjoint defects, including gamma asymmetry. Wall
pressure/viscous reaction is minus each actual wall contribution. Stationary
wall mechanical work is zero; extrapolated ghost velocity is not wall motion.
All density, pressure and viscosity use the same support and planar side.

## Predeclared measurements and stopping rule

Hydrostatic: H=1 m, free atmospheric top, bottom wall, infinite periodic tangent
planes (no vertical/free-surface-wall junction). n=16/24/32, h/dx=1.5/2/2.5,
ordered and seeded 10%-spacing displacement, dt=1e-4/5e-5/2.5e-5, T=.1 s.
Parcel mass integrates rho0 exp(g(H-y)/c²) over its source interval exactly;
initial rho is that continuum value at the parcel center. This is not a
quadrature correction. Measure discrete acceleration before stepping. Kick
velocity, then update continuity and positions with kicked velocity (declared
symplectic-Euler adaptation; no universal stability claim). Stop/refuse on
nonfinite state, nonpositive rho, bottom crossing or Mach>.1; retain last state.
Research dt is externally fixed, **not production-admitted**. Record density RMS,
pressure RMS/(rho0*g*H), maximum speed over horizon, fixed mass ledger and full
EOS internal+kinetic+gravity energy. Record integrated instantaneous pressure-work
and viscous power plus independently accumulated kinetic/gravity kick, kicked-continuity and
nonlinear EOS chain remainders, and their final closure residual. Never infer
these contributions from the remaining energy difference. Targets unchanged: rho RMS<=1%,
pressure<=2%, speed<=.01 m/s, mass<=1e-12, no crossing and decreasing resolution
error. Full unexplained energy threshold is 1e-8 of rho0*g*volume*H; explicit
measured defects are diagnostics, not a substitute pass.

Shear: height .001 m, n=16/32/64, all support choices, T=.1 s,
requested dt=1e-4/5e-5/2.5e-5. Start exact Couette (upper .001 m/s) or
Poiseuille (stationary walls, body acceleration .01 m/s²). Also free-slip
constant velocity .001 m/s. The native assembled constant-density viscous
matrix includes actual dummy extrapolation or SA slope law. Before stepping,
equal parcel masses make the Euclidean norm exactly proportional to kinetic
energy. Check its explicit contraction matrix
`-(L+L^T)-dt L^T L` for positive definiteness, reduce dt to .9 of the numerically
computed contraction boundary, and shorten final step. This floating-point
matrix check does not prove nonlinear acoustic/geometry stability. Free-slip
has a constant null mode; its constant-profile screen uses the native
nonnegative row/Markov bound and checks the invariant directly. If no
positive contraction interval exists, record refusal. Targets: finest RMS<=2%,
finest two pairwise spatial orders>=.9, traction<=5%, momentum/source/reaction
ledger<=1e-8 of nonzero rho0*height*.001 m/s and wall work closure at same scaled
tolerance. Report absolute SI defects, measured viscous exchange and the independently
computed explicit-kick energy term. Do not call exchange physical dissipation
until traction, conservation and nonpositivity gates pass.

3D consistency: periodic L=.001 m cubes at n=16/24/32, h/dx=1.5/2/2.5,
ordered/seeded 10% perturbations, vx=.001 sin(2 pi y/L), other velocities zero.
Seed fixed integer mixer; actual vector displacement normalized to norm<=.1dx and independently checked.
Evaluate density partition, gradient/Laplacian quadratic moments, actual neighbor
count, native initial viscous decay Rayleigh rate vs nu*(2pi/L)^2, pair momentum
and kinetic power. Pair enumeration is symmetric; self density included. This is
an instantaneous operator screen, not the full trajectory/decay gate. Finest
rate<=2%, consistency improves with support/resolution, closed momentum<=1e-10
and no positive viscosity power. Fixed-support dimensionless biases are retained.

Planar isolation: side-labelled infinite resolved slab, angles 0/17/31 degrees;
use independent segment/slab intersection oracle. Identical excluded-pair
predicate must remove density, pressure and viscosity support factors exactly.
Thin diagnostic queries retain nonzero unmasked support; they test the query
only and do not admit the refused volumetric wall formulation. Refuse thinner
than 2R and zero-thickness volumetric dummy support in this limited screen;
that conservative experimental domain does not establish a general physical
thickness limit. Imported meshes, edges and free-surface-intersecting finite
walls remain untested, explicitly unavailable for production.

Stop selection at this bounded campaign. Any failed required screen or missing
3D/pressure timestep gate prevents production admission. Passing reduced
screens only identifies the next candidate for independent 3D qualification.
Do not silently retune kernel, wall law, dt, EOS or thresholds during evaluation.

## Execution and evidence

`source /workspace/rheon-setup/env.sh`; `cargo test --locked --no-default-features --example particle_planar_selection`;
`cargo run --locked --release --no-default-features --example particle_planar_selection -- FRESH_DIRECTORY`.
Retain all raw CSV fields and summary measurements outside Git. Commit only
source, protocol, results/review and compact hashes. Binary/source/compiler/command
binding in manifest; previous failed runs retained separately. Program exit 0
means campaign execution completed, not physical acceptance. Focused Clippy,
format and unchanged-preview regression checks are supporting gates.
