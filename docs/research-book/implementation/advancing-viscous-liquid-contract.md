# Advancing viscous liquid: bounded goal and current obstruction

The end goal is one **genuinely advancing, narrowly supported viscous liquid
step**: accepted liquid geometry, all required momentum components, compatible
pressure and strain, and one physical clock advance. It is not achieved by the
static MAC materialization milestone. The constant-field repair and static MAC
bridge source/evidence heads remain frozen on their review branches. This
companion is a research checkpoint identifying missing contracts; it adds no
production stepping mode and claims no advancing rendered simulation.

## Minimal remaining sequence

1. Select one physical liquid domain and discrete velocity/pressure spaces for
   both the old and candidate geometry. Derive face masses, wet face flux areas,
   surface normals, active support and all six strain samples from that same
   domain. Restrict initial work to constant density/viscosity, bottom-attached
   columns, stationary free-slip solid walls, no adhesion/capillarity and a 2D
   slice extruded in the third direction. Keep height/thickness/slope and arithmetic
   rejection. A first bounded implementation can refuse support changes as well.
2. Advance volume and **every momentum component that can become nonzero** with
   the same routed mass transfers. Account for phase/momentum boundary exchange,
   wall forces, remap mixing and store rounding. Surface motion must determine
   changing normal-face support and masses, rather than admit unrelated cap
   donors. Conservative momentum does not follow from conservative phase alone.
3. Assemble compatible pressure and Newtonian symmetric strain on candidate
   geometry. Derive the free-surface stress condition and its discrete boundary
   terms. Solve/gate the composed momentum/pressure equations, not merely the
   old inviscid atmospheric projection followed by unqualified damping.
4. Verify candidate phase mass, all momentum components, measured work/energy,
   true linear residual, actual stored divergence and common physical time.
   Publish existing velocity, pressure, phase and both geometry/time identities
   only after one final cancellation barrier. Keep scratch capacities explicit;
   reuse existing candidate/geometry owners without accepted-state snapshots.
5. Qualify repeated advancing execution using an analytic or controlled physical
   flow, independent numerical checks, cancellation/failure from nonzero accepted
   states, and actual rendered replay. Include refinement and any accuracy failure.
   Until these pass, the advancing end goal remains pending.

This is one dependency chain toward an advancing step, not a sequence of isolated
operators relabeled as completed liquid simulation. The concrete obstruction is
currently at the first three links. The existing transaction machinery already
provides a suitable publication pattern, but cannot supply missing mathematics.

## A moving global-flat closed layer is not an invariant class

For an incompressible constant-density liquid under a common height H(t) and
fixed footprint of total area A>0, its mass is M=ρAH. With sealed solid walls,
no sources, and a material free surface, mass conservation gives

    ρ A [H(t+Δt)−H(t)] = 0,   hence H(t+Δt)=H(t).

Equivalently A Hdot=0. There is no nontrivial common-height motion within that
closed global-flat class. At a flat material surface the kinematic condition is
Hdot=u_n(surface). Different surface-normal velocities produce different column
heights; zero *total* surface flux only conserves the integral, not flatness.
This does not preclude viscous velocity evolution with stationary geometry.
Such a stationary-surface invariant flow is a distinct supported goal and cannot
be substituted for a requested moving-surface step without making that scope
explicit. Open inflow/outflow or volume sources could change a common height,
but require their own boundary momentum and stress work; none is silently added.

The exact native accepted field retained at
`evidence/column-mac/demo/pulse-y-0.25-jacobi/case.json` supplies a concrete witness.
Its physical height is 0.5625 m on four 0.25 m² columns. The virtual pressure
surface slots have accepted f32 normal velocities

    (−0.27636364102363586, 0, 0, +0.27636364102363586) m/s.

Their area-weighted sum is zero. Interpreting them as material surface velocities
would give a height spread of 0.0005527272820472718 m after an Euler interval of
0.001 s. This is a **hypothetical kinematic interpretation** of a frozen static
field, not a production advance or an accuracy claim. The current static API
correctly refuses to use it for transport. The witness establishes why globally
flat mass/projection support is insufficient for generic subsequent motion.
Its actual export also omits positive normal kinetic energy, so re-prescribing
only the two tangential components is not conservative three-component evolution.

## Geometry and normal momentum need a shared interpretation

The current phase reconstruction describes bottom-attached columns. Its old
inviscid pressure crossing uses interpolated center heights. The conservative
parcel remap interprets columns as stepped prisms with piecewise constant slab
means. The static MAC bridge defines pressure fluxes on a flat collapsed-slab
control volume and a virtual top face. These are declared restricted models;
they do not yet define one varying-height moving domain and strain space.

For unequal heights, adjacent final collapsed slabs have different endpoints.
A horizontal interface must be split into overlaps and exposed liquid/air
segments. A row can intersect more than one neighboring slab. Assigning the
old nearest-neighbor coefficient or a full-box strain weight does not derive
that geometry. Reconstructing a smooth graph instead changes volumes and normals
and requires explicit reconciliation with the authoritative phase fractions.
A bounded overlap graph is possible under the existing slope restriction, but
its face incidence, velocity basis and exposed-surface degrees of freedom must
be derived before implementation.

Normal momentum is not an optional export detail in a moving flow. The virtual
surface unknown needs a physical support and motion rule. For example, consider
the last normal interval [(L−1)h,H], of length ω=H−(L−1)h. The static bridge's
top-face mass is

    m_top^bridge = ρ A (ω−h/2).

If instead normal velocity were P1 between the physical face at (L−1)h and the
surface at H, the integrated row-sum top mass would be

    m_top^P1 = ρ A ω/2,
    m_top^bridge − m_top^P1 = ρ A (ω−h)/2.

The neighboring interior P1 mass would be ρ A(h+ω)/2, rather than ρAh. Both
partitions have the same total mass but differ whenever ω≠h. For the native
quarter-cap witness, the bridge/P1 top masses are 0.046875/0.0390625 kg per
column. This is not a defect in the bridge's declared pressure metric, nor a
proof that no compatible strain operator exists. It demonstrates that the
simplest proposed P1 basis cannot be assumed to integrate that metric exactly.
A chosen mass quadrature or a different basis needs its own consistency and
boundary derivation. The earlier tangential-only P1 shear basis does not answer
this normal-component question.

The existing within-column remap accepts explicit cap donors. It is not a
certificate of actual intercolumn mass routing. A moving successor must either
use shared physical phase/momentum fluxes directly or derive how their integrated
transfers determine donors for all three component supports. Changing height by
an unrelated prescription and subsequently remapping profiles would reproduce
another externally prescribed sequence, not the intended advancing step.

## Pressure and stress cannot be inferred from energy damping

With no capillarity and atmospheric pressure p_a, the material surface condition
is

    (−p I + 2μ D(u)) n = −p_a n.

For an axis-aligned flat surface this requires

    p−p_a = 2μ ∂_n u_n,
    ∂_n u_t + ∂_t u_n = 0  for each tangential component, when μ>0.

Setting p=p_a additionally requires zero normal strain. That holds for the
previous periodic tangential-shear restriction, with u_n=0 and no lateral
variation. It does not follow from divergence-free generic velocity. Independent
component damping need not satisfy tangential traction either. A positive
semidefinite matrix by itself proves discrete dissipation, not that the matrix
represents symmetric strain or the right surface boundary.

[Batty and Bridson 2008, Sections 4–5](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf)
derive coupled viscous surface terms and a variational split. Their later
[time-dependent Stokes paper, Sections 3–5](https://arxiv.org/pdf/1010.2832)
addresses pressure/stress coupling. These primary results motivate the research;
neither validates Rheon's collapsed-slab geometry. A split treatment would need
an explicit normal-stress approximation and its measured error. The current goal
should instead derive a compatible coupled treatment before claiming it complete.

One candidate finite formulation on a fixed candidate domain is

    (M + Δt Eᵀ W E) v + Δt Bᵀ p = q_adv,
    B v = 0.

Here q_adv is conservatively transported **momentum**, M is derived candidate
liquid inertia, B is integrated negative divergence, E is actual symmetric-strain
sampling and W contains nonnegative viscosity/geometry quadrature. Free boundary
velocity variations must encode the complete natural stress condition. This is
not already the existing nearest-neighbor pressure PCG: its Schur complement
contains the inverse of M+ΔtK, not M alone. A bounded solver needs true composed
residual checks and declared memory. Merely inserting an arbitrary E would
still leave geometry, rigid-motion consistency and boundary traction unqualified.

For a backward-Euler unforced fixed-domain candidate satisfying these equations,
pressure work cancels if Bv=0 and

    E_before−E_after = ½‖v−u_adv‖²_M + Δt vᵀKv.

Moving mass, advection/remap, gravity or prescribed-boundary work add separate
terms. They must be measured rather than attributed to viscosity. A sensible
first-order ordering is conservative phase/momentum transport from Ω_n, assembly
on Ω_(n+1), then coupled stress/pressure there; it still requires the missing
shared moving flux geometry and a complete work ledger. More accurate geometry
coupling is a later question, not assumed by advancing the clock alone.

## Precise research exit criteria and current status

The next research result must specify a bounded domain/basis/flux construction
for the 2D extruded varying-height slice; all face masses and surface strain
weights; shared three-component transport and boundary work; a pressure/stress
formulation; and the applicable stability/solver/moving-energy gates. Hand
matrices must test symmetry, true work adjoints, nonzero affine shear, constrained
boundary behavior and local admissible rigid-motion null modes. Then the pieces
can be implemented in one advancing owner transaction and qualified with repeated
physical time and genuine phase/momentum/energy evolution. A controlled gravity
relaxation is a possible eventual replay, but is not yet executed or validated.

`MovingLiquid.lean` contains seven conditional exact-real obstruction identities:
constant common height from supplied conserved mass; zero common rate from a
supplied zero-flux balance; nonflat Euler heights from unequal rates; balanced
two-column volume exchange; zero normal strain from supplied atmospheric normal
traction; positive energy of omitted nonzero normal velocity; and the algebraic
P1/bridge top-mass difference. These prove none of continuum conservation,
kinematic discretization, basis integration, geometry/strain assembly, solver
convergence, Rust/IEEE behavior, transaction atomicity or physical validation.

The advancing end goal is **blocked by the missing shared moving geometry and
full pressure/stress discretization**, not completed by this research checkpoint.
Production refusal and every existing tolerance remain unchanged. Old evidence,
including the recorded f32 viscosity accuracy failure, remains immutable.
