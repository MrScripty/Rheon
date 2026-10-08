# Exact compatible endpoint no-slip in a fixed slab

`ColumnShearWorkspace::update_with_boundaries` extends the independently
accepted finite-Navier operation with explicit tangential endpoint constraints.
It retains the same five normal-count f64 scratch arrays, liquid-only lumped
mass, interior viscosity stiffness, output publication gate and 64-epsilon
identity budgets. The old `update` and `update_with_walls` APIs remain available.

## Boundary data and limits

Lower/upper `ColumnShearBoundary` values select either
`Navier(ColumnShearWall)` or `NoSlip { velocity: [f32; 3] }`. Navier coefficients
remain finite and nonnegative in Pa s/m; zero is free slip. A no-slip velocity
is in m/s, representable in the stored f32 field, and constant for this step.
Normal motion, nonfinite/subnormal speeds and zero viscosity at a no-slip wall
are refused. Positive density and the fixed-flat-slab prerequisites still apply.

The incoming endpoint trace must equal its prescribed no-slip velocity. The
operation does not silently project an incompatible state or account for an
impulsive change in prescribed wall speed. A compatible initial field may be
nonzero at a moving endpoint; its initial energy and momentum must be retained.
Two no-slip walls on the same wet node are refused even when their speeds agree,
because their separate reaction impulses are nonunique. One no-slip wall and
one finite Navier wall on a single wet node have a unique reaction and are
supported. All refusals and cancellation leave the caller's output unchanged.

The retained basis extends the first/last wet-center value constantly to its
wall. No-slip is exact for that trace at both ends of the step. This does not
make the interior continuum representation exact: the basis still has spatial
endpoint error. The cap is confining, geometry stays fixed, lateral directions
are periodic, normal velocity is zero, and tangential fields are layer-uniform.

## Reaction impulse, momentum and work

Let f_j be the bulk plus finite-Navier force before constraint elimination,
m_j=rho A omega_j the lumped mass, and delta_j the exact-real increment. At a
free node, m_j delta_j=dt f_j. At a compatible constrained node, delta_j=0, so

    J_j = -dt f_j,                 m_j delta_j = dt f_j + J_j.
    W_constraint = sum_j w_j dot J_j.

J is reaction impulse on the liquid in N s, and W is delivered work in J. The
reported reaction force is J/dt, assembled directly as -f rather than through
an artificial large coefficient. Moving walls can supply or extract energy.
No-slip relative reaction work is J dot (u-w)=0; this is not zero bulk viscous
dissipation. A finite Navier wall keeps its relative slip dissipation separately.

The new report exposes each total wall force, total wall impulse and no-slip
reaction impulse. Total wall impulse includes finite Navier traction; reaction
impulse is zero at Navier walls. Actuator work includes all walls. The balance is

    P(v)-P(u) = sum_b I_b + stored-f32 rounding momentum.
    T(v)-T(u) + dt(P_bulk+P_Navier) - W_all
      - delta^T M delta/2 - stored-f32 rounding work = 0.

For mixed laws on a single node, the no-slip reaction also cancels the other
wall's finite Navier force. Their impulses and work remain separate. No-slip
rows have zero increment and are excluded from the explicit admission maximum.
Free rows retain dt(sum adjacent stiffness + incident Navier beta A)/m <= 1,
including an edge connected to a constrained endpoint. A fully constrained
two-node slab has no explicit free row; its admission number is zero. No hidden
substeps, coefficient changes or automatic numerical retries are introduced.

`proofs/Rheon/NoSlip.lean` proves four exact-real contracts: compatible reaction
preserves the trace, relative reaction work vanishes, finite momentum balances,
and the constrained finite work identity. The latter two assume the per-node
step equation; the energy statement additionally assumes assembled bulk/Navier
power and compatible reaction work. These algebraic statements do not prove
Rust/IEEE behavior, assembly, stability, spatial convergence or temporal order.

## Native interactive laboratory

Build the example and select a fresh directory outside Git:

```sh
cargo build --release --no-default-features --example column_no_slip
target/release/examples/column_no_slip /tmp/rheon-no-slip-lab
```

Open `index.html` beside `results.json` and the three CSV files. The case selector
chooses 8/16-layer two-wall no-slip or 16-layer lower-Navier/upper-no-slip. The
time control selects five actual native snapshots, including the compatible
initial field. A prominent snapshot-playback note beside the controls states
that the three recorded cases are fixed and provide no live recomputation or
general liquid solver. It neither interpolates nor recomputes a trajectory in JavaScript.
The ledger displays cumulative reaction and total impulses, signed work,
bulk/relative-wall losses, initial energy, explicit and rounding corrections.

Exactly three cases run 8192 steps each through 96 s. All use a unit height/area,
rho=3 kg/m³, mu=0.15 Pa s, lower wall speed zero and upper speed 1 m/s. The mixed
case uses lower beta=0.3 Pa s/m and ell=mu/beta=0.5 m. The upper endpoint starts
at 1 m/s; other wet nodes start at zero, so initial energy is rho A h/2.

With H=1 and h=H/L, the continuum and discrete steady references are

    both no-slip:     u(y)=y/H,       u_j=j/(L-1).
    mixed lower slip: u(y)=(y+ell)/(H+ell),
                      u_j=(j h+ell)/((L-1)h+ell).

The mixed continuum denominator matches Chapter 24's original one-wall oracle;
it does not use the symmetric two-wall Navier denominator. Native states at
wet centers compare against y=(j+1/2)h. Distance from the discrete equilibrium
includes unfinished relaxation, explicit transient error and f32 rounding.
It is not an isolated integration error or a temporal-order measurement.

New engineering gates are declared before the run: two-no-slip continuum max
deviation must decrease 8→16 and be <0.032 m/s at 16 layers; mixed deviation
must be <0.021 m/s; all three discrete-equilibrium deviations must be <5e-5 m/s.
These bounds acknowledge the retained endpoint basis and do not alter any
original finite-Navier gates. Endpoint no-slip is checked at every native step.
Generated data, plots, logs, ZIPs and compiled artifacts stay outside Git.

## Exclusions

This is neither wetting nor adhesion. It does not qualify contact-line physics,
arbitrary collision-mesh fluid topology, moving normal walls, time-varying
prescribed speeds, variable-density inertia, general 3D stress or evolving
interfaces. Historical failed assertions, pressure selection refusal, E2
evidence and PR22 refinement outputs remain separate and preserved. No E2
roster or new reference integration is part of this feature.
