# Implementation guide: the bounded material preview

The desktop can emit physical material parcels into an initially empty box.
This is the role of [`ParticleFluidSimulation`](../../../src/particle_fluid.rs).
It is a bounded SPH preview with its own density and force model. The original
carrier solver, pressure methods and recorded replay contracts remain available
in the other desktop mode. The browser's 24³ concentration lesson still uses
that carrier model; it does not run this particle simulation.

Read [Chapter 11](../chapters/11-particle-methods.md) for the wider particle-method
design space, [material scenes](../../material-fluids.md) for controls and units,
and the [desktop owner](../../../src/bin/desktop/README.md) for live history and
thread lifecycle. No PBF, PIC, FLIP or APIC implementation is implied by this
preview's use of particles.

## What one parcel represents

A parcel stores position in metres, velocity in m/s, mass in kg, temperature in
K and a material identity. The numerical spacing determines a reference parcel
volume. Emitted parcels receive mass equal to source reference density times
that volume. Initial filled scenes distribute the declared material mass over
their seeded parcels. The emitter supplies mass from an external reservoir;
it is not a closed-system mass-conservation experiment while injection is on.

For a filled cube of side \(L\) and reference density \(\rho_0\), the intended
initial mass is \(\rho_0L^3\). Doubling \(L\) therefore multiplies this mass by
eight at fixed density. Changing a display unit or camera scale does not do so.
The native fixture checks physical mass at several domain sizes. It does not
establish that geometrically scaled moving scenes share the same dynamics:
gravity, acoustic limits and source pressure also set physical scales.

## Density and pressure are declared approximations

Let \(W_h\) be the compact poly6 kernel and \(\rho_{0,j}\) the reference density
of neighbor j's material. The current estimate is

\[
f_i=\sum_j\frac{m_j}{\rho_{0,j}}W_h(x_i-x_j),\qquad
\rho_i=\rho_{0,i}\max(f_i,0.05).
\]

Self density is included. Bins identify candidate neighbors, and an exact
support-distance test chooses contributors. The reference-volume factor
prevents a denser material's parcel from being interpreted as extra volume of
the receiving material. The floor keeps estimated density positive; it changes
the formula and must be included in an implementation comparison.

The implemented pressure laws are

\[
p_i=\max\bigl(c^2(\rho_i-\rho_{0,i}),0\bigr)
\quad\hbox{for liquid},\qquad
p_i=\rho_i R_iT_i\quad\hbox{for gas}.
\]

Here \(c\) is a deliberately reduced liquid acoustic stiffness, and \(R_i\)
is the gas-specific constant. The gas reference density is chosen to reproduce
its specified reference pressure and temperature. A liquid can compress in this
model; a zero tensile pressure does not impose exact incompressibility. Gas
expansion into vacuum is an approximate parcel behavior, not a qualified model
of shocks, nozzle choking or rarefied gas kinetics.

## Pair forces: momentum and energy answer different questions

With the native density estimates held fixed, define

\[
A_{ij}=\frac{p_i\rho_{0,i}}{\rho_{0,j}\rho_i^2}
       +\frac{p_j\rho_{0,j}}{\rho_{0,i}\rho_j^2},\qquad
F^{p}_{ij}=-m_im_j A_{ij}\nabla W_h(x_i-x_j).
\]

Exchanging i and j leaves \(A_{ij}\) unchanged and reverses the kernel gradient.
Thus the pair forces cancel in exact arithmetic, even when masses differ.
Their accelerations need not match: each force is divided by that particle's
own mass. The pressure term uses the gradient of the same poly6 kernel used
in density. Replacing it with a different kernel gradient would require a new
consistency argument.

For the viscous pair, the code corresponds to

\[
b_{ij}=\frac{m_im_j(\mu_i+\mu_j)}{2\rho_i\rho_j}L_h(r),\qquad
F^v_{ij}=b_{ij}(v_j-v_i),
\]

where the supported kernel Laplacian \(L_h\) is nonnegative. Positive densities,
nonnegative viscosities and masses give \(b_{ij}\geq0\). Pair power is
\(-b_{ij}|v_i-v_j|^2\), but an explicit timestep still needs a size restriction.
For one isolated coordinate with frozen \(b=b_{ij}\),

\[
E^+-E=-\Delta t\,b
 \left[1-\frac{\Delta t\,b}{2}\left(\frac1{m_i}+\frac1{m_j}\right)\right]
 (v_i-v_j)^2.
\]

This is nonpositive when \(\Delta t\,b(1/m_i+1/m_j)\leq2\), with positive
masses and nonnegative timestep. For 1 kg masses, velocities ±1 m/s and
\(b=1\) kg/s, an interval of 1/4 s gives velocities ±1/2 m/s and energy
1/4 J instead of 1 J. An interval of 5/4 s gives energy 9/4 J. These exact
teaching witnesses isolate viscous drag; they are not proposed timesteps for
the complete native scene.

Gravity, injection, pressure and wall contact can change a full scene's energy.
The liquid cohesion term is a symmetric attraction approximation for matching
liquid materials. It does not calculate surface curvature or contact angle.
Wall drag damps tangential contact motion, and wall heat exchange relaxes
contact temperature. There is no bulk conduction field, vacuum cooling model,
phase change or elastic slime model.

## Source credit explains intermittent emission

Each end of the cylinder has its own accumulated volume credit. The source
uses a nonuniform disk profile, a pressure-head speed and an approximate 0.43
profile-average factor for volume flux. If the current credit plus supplied
volume is smaller than one reference parcel volume, no particle appears that
step. The credit is retained rather than discarded.

Write \(C\) for previous credit, \(Q\) for prescribed volume flux, \(V_p\) for
parcel volume and \(n\) for emitted parcel count. The update is

\[
C^+=C+Q\Delta t-nV_p,\qquad
n\rho_0V_p+\rho_0C^+=\rho_0(C+Q\Delta t).
\]

For a hand calculation, measure volumes in units of \(V_p\): credit 0.6 plus
new supply 0.7 permits one parcel and leaves credit 0.3. Credit is future source
volume, not mass already inside the box. This identity accounts for the
prescribed source; it does not validate the approximate flux coefficient or
prove how many parcels a floating implementation must select near an integer
threshold. The visible cylinder is a source illustration, not a rotating
collision solid.

## Prepare, retry, then publish

`step` may shorten the requested interval using acoustic, advective, gravity,
viscosity, cohesion and emitter-rotation limits. It prepares candidate particles,
source credits, neighbor bins, density and forces. An additional acceleration
or damping check may restart preparation at a shorter interval. At most 16
preparations occur in one call; unresolved stability returns an error.

The accepted state includes the particles, clock, generation, two source
credits and deterministic disk-sequence state. A failure or cancellation must
preserve them together. Candidate scratch may be overwritten. A successful
commit publishes one coherent state, then the desktop worker captures its
immutable preview. The report records the actual interval, particle count,
emitted and total mass, and speed/pressure/temperature diagnostics. Its
`max_pressure` comes from the force-evaluation densities and temperatures,
before positions and contact temperatures are updated. In contrast,
`max_speed` is measured after the accepted velocity/contact update. Those two
maxima are not simultaneous samples of a freshly recomputed accepted pressure
field.

The native tests in [`material_contract.rs`](../../../tests/material_contract.rs)
exercise vacuum/fill mass, scaling, source accounting, cancellation, capacity
failure, retry and thread-count replay. Unit fixtures beside the implementation
exercise gravity, pressure against an independent compression-energy gradient,
unlike-material inertia, viscous energy, cohesion and contact temperature. Their
scope and numerical tolerances matter; a finite successful trajectory is not
a continuum validation or a global stability theorem.

The particle thread-count fixture compares one source-free filled-scene step
at 26 samples per axis using one and four workers. It compares time and parcel
state directly. That is a bounded replay witness, not a claim that every
material scene, compiler or hardware configuration replays identically.

[`material_reference_contract.rs`](../../../tests/material_reference_contract.rs)
adds a small independent route: an unordered all-pairs reference compares
every parcel's full step for liquid and gas fixtures; a separate compression
energy finite difference checks each pressure-acceleration coordinate,
including unlike materials; and a first-emission witness checks actual disk
sampling, midpoint rotation, prescribed head and retained credit. These
fixtures avoid reusing the production linked-cell traversal. Their small
scope makes errors easier to locate, rather than replacing larger scene tests.

## Memory and display are different budgets

The numerical owner reserves accepted and candidate particles, thermodynamic
and force scratch, linked neighbor indices and bin heads at construction. Its
array budget uses actual compiled element sizes and capacities. It does not
silently cap neighbor contributions or remove particles to keep a picture
running. A particle or buffer limit is an explicit failure with accepted state
retained.

Separately, the desktop splats accepted parcels into an eight-bit voxel
preview. Surrounding-material parcels may participate in forces while being
excluded from the visible emitted-material surface. The image therefore does
not show all the simulated material or calibrated optical density. Changing
the surface threshold or preview resolution does not change particle sampling.
The planned history can lower its stored resolution to retain every accepted
step within its own budget. Renderer meshes, allocations and process RSS lie
outside the payload estimate.

## Three bounded experiments to read alongside the desktop

| Experiment | Expected comparison | Measurable limit |
| --- | --- | --- |
| Vacuum versus filled initial contents, source disabled | Initial parcel mass is zero versus density × volume | This checks initialization, not interface accuracy |
| More preview voxels at fixed physical samples and inputs | Stored image detail and payload change; numerical parcel spacing stays fixed | Voxel resolution cannot recover missing physical samples |
| More physical samples at matched physical time | Parcel count, neighbor work and accepted intervals can change | Compare mass, density/pressure and shape; do not infer convergence from a smoother image |

Record material, domain size, source, gravity, accepted time, physical sampling,
stored preview size and worker count with every comparison. A run's settings
are immutable; use a new run for changed physical parameters. Source injection
and adaptive timesteps make equal step counts an unreliable matching rule.

## Proof and execution boundaries

[`MaterialPreview.lean`](../../../proofs/Rheon/MaterialPreview.lean) gives
exact-real statements for the declared formulas. Representative names are
`pressure_pair_coefficient_symmetric`, `pressure_pair_action_reaction`,
`viscosity_pair_work_nonpositive`, `viscosity_pair_momentum`,
`viscosity_pair_energy_nonincrease`, `nozzle_profile_bounds`,
`nozzle_pressure_head_identity` and `reservoir_mass_ledger`.

The pressure-pair symmetry statements tolerate the declared positive density
regularization. They do not prove that the pressure force is an energy
derivative across that floor or the liquid pressure kink. The viscosity energy
statement is for an isolated pair with frozen coefficients and its stated
timestep restriction. Source identities do not certify gas discharge physics,
the 0.43 approximation or floating parcel-count selection.

The independent native energy-gradient witnesses stay away from the active
density floor, the liquid-pressure kink and the near-coincident pair exclusion
at distances below \(10^{-10}h\). Those hypotheses must remain visible when
interpreting a successful finite-difference comparison.

Source statements are not a proof-run receipt. Publication must bind these
modules to the current reviewed source inventory, pinned normal Lean build and
allowed-axiom audit. Native execution tests separately connect the code to
independent equations and exercise ownership failures. Neither evidence layer
establishes a physically calibrated multiphase solver, full-scene convergence,
IEEE equivalence to Lean or a performance guarantee.
