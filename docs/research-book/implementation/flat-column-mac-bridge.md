# Fixed-flat column momentum in the accepted MAC owners

This prerequisite connects the conservative column parcel representation to
Rheon's existing velocity, pressure, liquid-volume and reconstructed-column
owners. It materializes a prescribed tangential field, projects it with matching
liquid masses and flux measures, and publishes both owner revisions together.
It does not advance physical time or move the interface. Ordinary and viscous
steps refuse the resulting mode until a compatible coupled advance exists.

The dependency is: conservative parcels → mass-consistent MAC transfer →
compatible pressure → shared publication → moving-geometry transport and strain.
The preceding remap already supplies within-column overlap and explicit cap
exchange. Establishing its connection to the accepted MAC owners comes before
varying-height strain: different collapsed cap slabs produce nonconforming
lateral interfaces, which need a separate flux/dual-mass derivation. Fixed flat
columns have conforming lateral slabs and permit an independently assembled
pressure graph. The existing periodic flat-shear prototype has different wall
conditions and a P1 trial basis; it is not silently attached to this bridge.

## One geometry and three mass partitions

Take constant density ρ, normal spacing h, common height H=(k+f)h,
0≤f<1, and tangential column footprint A. The existing column contract requires
h/2 < H ≤ (N−1/2)h. There are L=k+1[f>1/2] wet pressure nodes. Parcel dual slabs
are [jh,(j+1)h] for j<L−1 and [(L−1)h,H] for the final node. Write their lengths
ω_j and define V_j=Aω_j, m_j=ρV_j. The cell-lattice parcel components are means
on these slabs, not P1 nodal samples. The phase fraction array retains its
original grid-cell representation; an air-centered partial cap is folded into
the last wet pressure slab. Compensated phase mass and parcel mass are compared
explicitly with the unchanged 64ε extensive budget.

For each tangential component, interior MAC face mass is m_j and each fixed
outer wall face owns m_j/2. Its interior flux area is h_other ω_j, and wall/dry
flux is zero. Equal adjacent slab masses justify averaging their two parcel
values. Wall velocity is prescribed zero; wall impulse and removed kinetic
energy are explicit boundary terms, even though wall dual mass is retained.

For the normal component, the bottom wall mass is ρAh/2; interior face masses
are ρAh. The top pressure face has mass ρA[H−(L−1/2)h]. Interior and top flux
areas are A. Above the top face, mass and flux vanish. Each component's complete
face mass partition, including the zero-velocity wall masses, equals Σ_j m_j
per column. The top slot is a **virtual free-surface pressure degree of freedom**
in existing MAC storage. It does not make ordinary sampling or phase transport
compatible with this control-volume geometry. That is why dynamics are refused.

`FlatColumnMacGeometry` borrows the accepted `ColumnSurfaceView`; it owns no
second geometry or mass cache. All columns must have exactly the same stored
height decomposition. Geometry, density, coefficient scale and active support
are checked before transfer. No small-cell floor, fraction clipping or damping
coefficient is introduced. Crossing a cell-center threshold changes L and the
collapsed slab layout; just above that threshold the top normal mass can become
small. This static qualification supplies neither a continuity argument nor a
moving-domain treatment for that transition.

## Transfer, adjoint restriction and explicit losses

For an interior tangential face with parcel velocities a,b, set
v=(a+b)/2. Equal values copy the existing f32 value exactly, retaining the
constant-field correction in the preceding remap successor. For a two-cell row
with cell masses m, the output face mass is m and the two wall masses are m/2:

    P_MAC − P_parcel = −m(a+b)/2 + P_round
    E_parcel − E_MAC = m(a²+b²)/4 + m(a−b)²/8 − W_round.

The first energy term is prescribed wall removal; the second is interior
averaging loss. These are not viscous dissipation. Restriction computes each
cell's average of its two bounding tangential faces. It is the mass adjoint of
lifting with wall velocities fixed, and preserves weighted tangential momentum
up to measured f32 rounding. It is a lossy averaging operation, not an inverse.
Its report explicitly records the normal momentum and energy omitted from the
two-component output. Re-prescribing an exported field also records the change
from the full previous accepted three-component state, including omitted normal
energy; the repeated demo cannot be interpreted as unforced liquid evolution.

All transfers use a declared 3F f64 workspace, nominally 8F bytes where F is the
sum of the three MAC face counts. Actual vector capacities are charged. Outputs
are copied only after every arithmetic, ledger and cancellation gate. Restriction
reuses the first N entries of two workspace vectors; it allocates no per-call
snapshot. Inputs and stored candidates require normal-or-zero finite values.
Nonzero subnormal results, nonzero product/division underflow and invalid f32
stores are rejected in the bridge's checked arithmetic. This does not establish
such a guarantee for every internal PCG intermediate.

## Matching pressure graph and energy accounting

Let B take signed integrated face flux, lower minus upper, on each wet slab.
Let M contain the face masses above. The operator and correction are

    A_pressure = B M⁻¹ Bᵀ,     b = B u / Δt,
    v_raw = u − Δt M⁻¹ Bᵀ p,  p_air = 0,
    κ_face = q_face² / m_face.

Here Δt scales the pressure; it does not advance a clock. Tangential cap
coefficients scale with ω_j. The top normal coefficient corresponds to the
actual pressure-node-to-interface distance. RHS, matrix application, Jacobi and
SGS neighbor coefficients, correction and true residual divergence all use the
same mass/flux geometry. Divergence is integrated flux divided by the wet slab
volume, not by the original full cell volume. Legacy operator modes retain their
original coefficients and scaling.

With δ=v_raw−u, D=½δᵀMδ, residual work W_res=−Δt pᵀBv_raw and f32 store work
W_round, the measured identity is

    E_stored − E_before + D − W_res − W_round = 0.

The allowance for energy increase contains positive measured residual work,
absolute rounding work and the existing 64ε extensive budget. Momentum changes
are checked against pressure impulse plus rounding; sealed walls and atmospheric
pressure can exchange momentum. This is not a closed-system momentum claim.
True linear residual, predicted volume divergence and actual stored-f32
divergence retain their existing settings; none is relaxed for this evidence.
The independent oracle assembles B separately and solves small cases with a
dense SPD matrix. It also reconstructs the native paired/compensated flux
accumulation when comparing a tiny reported residual-work budget; its separate
six-term incidence calculation remains checked against a flux-magnitude roundoff
bound. This changes no native acceptance tolerance.

The MAC placement, discrete gradient/divergence relationship and atmospheric
pressure boundary direction follow the primary [Bridson and Müller SIGGRAPH
course notes](https://www.cs.ubc.ca/~rbridson/fluidsimulation/fluids_notes.pdf),
sections 4.1–4.5. The collapsed-slab mass and flux construction above is the
specific restricted model qualified here; those notes do not validate it.

## Publication through the existing facade

`LiquidTransportSimulation::materialize_flat_column_profiles` takes an exact
`ColumnMacStateStamp` pair, borrowed two-component profiles, a pressure scaling
Δt and a cancellation callback. The first materialization requires the fresh
rest carrier; subsequent materializations require the same static mode. An
already advanced legacy carrier is rejected rather than reinterpreting its
full-cell inertia. Both density owners must agree and the accepted fractions
must match the current reconstructed-column stamp.

Preparation reuses `Simulation` candidate velocity and `PressureWorkspace`.
Previous accepted momentum/energy and prescribed changes are reported. After
all transfer, pressure, actual divergence, energy, phase-mass and cancellation
gates, one final publication barrier copies held pressure, swaps only the
accepted velocity vectors, increments carrier generation and liquid revision,
and updates both end and held-pressure geometry stamps. Fractions, heights,
tracers and physical time retain their accepted values. No accepted snapshot is
allocated for rollback. Export takes the actual revision pair, borrows those
accepted MAC fields, and transactionally restricts into caller-owned output.

On a 64-bit target, the existing column facade has nominal array storage
16F+80N+48C bytes, where C is the number of columns. The bridge workspace adds
8F, for total 24F+80N+48C. Reports charge actual capacities. This accounting is
for these arrays, not a whole-process memory cap; native evidence generation
also owns output snapshots outside the production API.

Eight native contract tests cover an independently assembled partial-mass
matrix and discrete adjoint, hand-computed transfer losses, all three normals
and both PCG methods, exact paired revisions, every callback occurrence from a
nonzero accepted state, output rollback, export normal omission, stale stamps,
nonflat/paused/overflow/scale failures, late actual-divergence failure and exact
retry, single-node and decimal-spacing phase-mass identity, and legacy advance
refusal. Accepted velocity, pressure, phase, tracer, clocks, geometry and stamps
are compared bit for bit on cancellation/failure.

## Measured behavior and proof scope

The [native packet](../../../evidence/column-mac/README.md) has 22 cases and
76 static publications. Its [measured figure](../../../evidence/column-mac/static-transfer-and-projection.jpg)
shows prescription/transfer/pressure energy and both refinement measures. Eighteen pulse cases
cover X/Y/Z normals, cap fractions 0.25/0.5/0.75 and Jacobi/SGS. Each subsequent
prescription is exactly the preceding accepted export. JSON holds every source,
lifted, accepted and exported f32 field, accepted pressure and identity/ledger;
initial/final grayscale PNGs are generated from actual MAC velocity. Independent
verification checks every stored candidate, phase mass, epoch and pixel.

Four instantaneous circulation cases use
u_x=sin(πx)cos(πz), u_z=−cos(πx)sin(πz), u_n=0 over a fixed 1 m square.
This is a kinematic manufactured transfer/projection reference with no stress
boundary or material calibration. Its mass-weighted velocity error uses the
sum of all three component masses (3 times liquid mass):

| Tangential cells | Velocity RMS (m/s) | Actual stored divergence (1/s) |
|---:|---:|---:|
| 8 | 0.007844378735 | 2.365010e−7 |
| 16 | 0.001965831582 | 4.766929e−7 |
| 32 | 0.000491752074 | 1.432405e−6 |
| 64 | 0.000122956731 | 3.814684e−6 |

Instantaneous transfer error decreases approximately fourfold per refinement;
stored-f32 derivative roundoff grows with resolution. All cases satisfy the
unchanged 1e−5 stored divergence limit. This is not temporal convergence,
moving-interface accuracy, a viscosity benchmark or general fluid validation.

The 13 `Rheon.ColumnMac` Lean theorems establish conditional finite exact-real
mass partitions, constant transfer, wall momentum, a two-cell mass adjoint,
energy partitions, positive pressure weights, pressure power, divergence/
gradient adjointness and an energy implication given residual work. Pinned
Lean/mathlib builds and transitive axiom audits pass; injected sorry and extra
axioms are rejected, and restored source passes. They do not prove arbitrary
geometry or basis integration, Rust/IEEE execution, PCG convergence, transaction
atomicity or moving-surface dynamics. These depend on native tests and measured
evidence only within their stated scope.

Still missing are compatible three-component moving momentum transport,
varying-height lateral flux/mass geometry, moving-interface pressure publication,
full strain on that geometry, adhesion and density constitutive laws, capillarity
and general reconstruction. The preceding frozen fixtures, including the known
f32 viscosity refinement accuracy failure, are retained without altered bounds.
