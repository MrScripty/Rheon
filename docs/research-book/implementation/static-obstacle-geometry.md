# One static obstacle geometry for collision and flux

`StaticObstacleGeometry` is a bounded geometry foundation. It consumes a
`GridGeometry` and the original immutable `TriangleSurface`, admitting a closed
axis-aligned box with eight unique corners and twelve outward triangles. Each
face must have two complementary triangles sharing a rectangle diagonal.
World-space vertices express translation and axis-preserving scaling in metres.
Open, duplicate, inward, non-box and unresolved geometry is refused. The existing
two-sided collision query API retains its broader, separate contract.

The owner retains that exact surface and stamp for collision queries. Cell
volumes, one opening per MAC face, connectivity and the conservative flux helper
derive from the same admitted vertices. Borrowed accessors do not reconstruct a
second mesh. This closes a bounded part of the collision-mesh requirement:
geometry and flux can agree about the stationary solid. Existing pressure,
transport and viscosity workspaces still require their original filled-box or
column domains. A [separate bounded follow-on](static-obstacle-flow.md) now borrows this owner for sealed pressure and reduced extruded shear; it does not retrofit the old carrier.

## Measures at represented endpoints

The physical cell boundaries are the evaluated world endpoints
\(x_d(k)=\operatorname{fl}(o_d+k h_d)\). Define the exact-real interval overlap
at those represented endpoints by

\[
L([a,b],[c,d])=\max(0,\min(b,d)-\max(a,c)).
\]

For cell intervals \(I_{i,d}\), the admitted solid intervals \(S_d\), physical
widths \(H_{i,d}\), and overlaps \(q_{i,d}=L(I_{i,d},S_d)\),

\[
V_i=H_{i,x}H_{i,y}H_{i,z}-q_{i,x}q_{i,y}q_{i,z}.
\]

For a face normal to axis \(d\) at coordinate \(x_f\), with tangential axes
\(a,b\),

\[
A_f=H_{f,a}H_{f,b}
-\mathbf1_{x_f\in S_d}L(I_{f,a},S_a)L(I_{f,b},S_b).
\]

The box is closed: equality on its boundary blocks that face intersection.
Volume has units m³ and area m². Outer faces report geometric openings; wall or
inlet speeds remain explicit caller data. A box touching the container can leave
its interior volume unchanged while blocking a boundary opening.

These formulas are exact-real specifications. Rust evaluates them in binary64.
Products, complements and flux products refuse nonfinite, subnormal or lost
positive measures, including a positive obstruction rounded away by subtraction.
There is no minimum-fraction clamp. Dyadic fixtures have zero comparison
allowance. The non-dyadic control compares rational measures at the represented
endpoints using an assembly allowance of \(64\epsilon\) times the corresponding
physical cell, face or domain measure. This is a declared fixture check, not a
universal IEEE error theorem or a temporal convergence result.

## Shared incidence and connected fluid

Each internal face stores one area. Its negative-side cell uses \(+A_fu_f\)
and its positive-side cell uses \(-A_fu_f\). Thus internal flux cancels in exact
arithmetic. `outward_flux` borrows the three velocity arrays and computes

\[
Q_i=\sum_{f\in\partial i}\sigma_{if}A_fu_f,
\qquad [Q_i]=\mathrm{m^3/s}.
\]

It performs no field mutation, allocation or pressure correction. The recorded
controls exercise one interior face at (1,1,1) on each axis with 2 m/s speed and
zero other speeds, checking the native neighboring flux pair against the shared
area. This checks spatial incidence, not a fluid time step.

Positive-volume cells receive deterministic component labels through positive
shared openings. Dry cells have no component. A box that spans a cell completely
in two directions and lies strictly inside its third interval splits that cell's
fluid into disconnected pieces. One cell unknown cannot represent those pieces;
construction refuses `UnresolvedCellTopology`. A whole grid-aligned separator
instead admits two distinct fluid components. No topology cleanup silently
removes a sliver or connects it across the solid.

For a later closed-domain pressure solve, each isolated component needs its own
compatibility condition and pressure gauge. Componentwise constant pressures
have zero weighted internal jump when every active edge joins equal labels.
That observation alone does not establish coercivity, the complete nullspace,
boundary compatibility or convergence of a future pressure operator.

If a future integrated residual obeys \(V_i D_i=\Delta t r_i\) in an active
cell, then

\[
D_i=\frac{\Delta t r_i}{V_i}.
\]

Tiny positive volumes amplify divergence for a fixed integrated residual.
Zero-volume cells cannot be pressure unknowns. The present feature assembles no
such pressure system and makes no residual qualification claim.

## Resources and publication

Construction accounts for actual retained Vec capacities of the consumed surface,
volumes, all face arrays and labels. The constructor peak adds its simultaneous
BFS queue, which is released before returning the owner. Checked arithmetic
plans these payloads before allocation; fallible reservation and actual capacity
checks can refuse the configured managed-buffer limit. An allocator can round a
reservation upward before its actual capacity is checked. Allocator metadata,
fixed stack, caller/callback storage and process RSS are excluded; this is not a
hard process peak bound or a whole-program memory certificate.

Cancellation is checked in cell, face and connectivity loops. A refused
construction does not modify a separately retained earlier immutable owner.
Publication is explicit: the caller replaces its owner only after success.

## Analytic controls and the recorded lab

The [native exporter](../../../examples/static_obstacle.rs) constructs nine
geometry controls: partial box intersections; X/Y/Z separators; dry, outside and
container-touching boxes; translated/scaled geometry; and a non-dyadic case.
Three thin subcell separators are recorded refusals. No time integration,
pressure solve or reference integration runs. Outputs must be fresh and outside
Git. The [rational checker](../../education/static_obstacle.py) independently
checks all cell and face measures, global volume, oriented closed edge incidence,
union-find components, analytic segment/box first hits and flux cancellation.
Its receipt binds the source, executable and records.

The [interactive geometry lab](../../education/obstacle-lab.html) selects those
native records, cells and face/collision axes. Cell boxes show labels, while the
original triangulated box shows the solid. Readouts expose physical measures,
the fixed native flux pair and the same-source collision hit. There are no live
arbitrary obstacle controls or fluid advances. A source-only edition without a
qualified packet states that the records are absent.

## Exact contracts and original research

[StaticObstacle.lean](../../../proofs/Rheon/StaticObstacle.lean) adds seven public
theorems for nonnegative, symmetric and bounded overlap; bounded volume
complement; shared flux cancellation; component-constant weighted jumps; and
residual/divergence scaling. Labels, active edges, nonnegative dimensions and the
integrated residual equation are stated assumptions. The geometry-stage local pinned
Lean build audited 64 public theorems and 87 declarations across that stage's
project with the existing axiom allowlist. Rust admission, triangulation,
binary64 predicates, BFS correctness and solver refinement are unproved. The
earlier PR24 source and 57/77 kernel qualification remain historical and separate.

Batty, Bertails and Bridson formulate pressure through kinetic energy and use
fluid mass in staggered velocity volumes; §2.1 also discusses face-area-based
approximations. This owner supplies primal cell volumes and shared face openings,
not their complete dual-mass projection or solid coupling. Their original paper
motivates retaining subgrid geometry instead of binary voxel masks.
[Original paper, 2007](https://www.cs.ubc.ca/labs/imager/tr/2007/Batty_VariationalFluids/variationalFluids.pdf).

Basilisk's original embedded-boundary source stores cell and face fractions and
guards interpolation against coupling disconnected regions. Its topology cleanup
can change fractions; Rheon's bounded owner instead refuses unresolved subcell
topology. This source supports the need for consistent geometry and topology,
not an implementation-equivalence claim.
[Original embedded-boundary source](https://basilisk.fr/src/embed.h).

The [bounded operator follow-on](static-obstacle-flow.md) adds stationary sealed pressure assembly/correction with component compatibility, gauges, rest and field controls, plus a separate reduced shear traction/work ledger. General viscous embedded-wall traction still needs additional interaction geometry. Arbitrary closed meshes, multiple fluid pieces within a cell, moving or
two-way solids, variable material coefficients and contact-angle evolution
remain unresolved. See the [requirements map](requirements-roadmap.md).
