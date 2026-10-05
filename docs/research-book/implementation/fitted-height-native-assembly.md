# Native fitted-height assembly milestone

This implements the bounded instantaneous slice selected in
[fitted-height-liquid-formulation.md](fitted-height-liquid-formulation.md).
`FittedHeightWorkspace` assembles and inspects one immutable periodic 2.5D
polygonal strip. It owns no accepted velocity/pressure/liquid state, has no clock,
and supplies no advancement, geometry replacement, pressure solve or transfer
step. The existing advancing-viscosity refusal and prior owners are unchanged.

## API and one physical geometry

`FittedHeightGeometry` supplies ordered cap points, independently ordered fixed
bottom abscissae, extrusion width, constant density and dynamic viscosity. Both
boundaries share a period and the cap closes at the same height. This permits a
skewed snapshot after horizontal cap motion while keeping the bottom fixed.
Each strip cell has two macrotriangles with the declared diagonal; centroids and
shared-centroid/edge intersections define the six-way Powell–Sabin splits.
Reject graph/order/closure, intersection, thickness or triangle-quality failure.
Minimum quality and height are rejection thresholds. They never floor masses.

All three components use the same positive nodal median-dual masses. Periodic
node identification and the fixed cap/bottom affine trace map R are explicit;
bottom normal velocity is zero. `velocity_embedding` and `embed_velocity` expose
that map. The caller's field arrays are borrowed during inspection, rather than
stored as a competing accepted flow state. Midpoint trace values must equal the
checked endpoint average; bottom normal velocity must be exactly zero.

`inspect` accepts a full-vector periodic nodal field and coefficients in the
assembled pressure basis. Each basis field is a selected column of the reduced
horizontal velocity divergence, with units inverse length; pressure coefficients
are therefore not per-triangle pressure samples. `pressure_basis` exposes the
sparse reconstruction. No pressure gauge is pinned. Constant pressure remains in
the image, as the native constant-mode representation checks demonstrate.

Geometry, masses, gradients, pressure basis and face topology are immutable
views. `triangle_stiffness` returns the full 9×9 local matrix on raw geometric
nodes, so affine and rigid-rotation patches can be tested without mislabeling
those fields as globally periodic/bottom-admissible flows. Global diagnostics
apply the full symmetric-strain operator and integrated pressure transpose to
caller fields. No assembled force is applied to an accepted velocity.

## Shared mass/flux/pressure/strain contracts

Within each microtriangle, masses are ρ b A/3 and the exact P1 gradients determine
all velocity strain terms and the integrated negative divergence. The strain
operator uses weights μ b A (2,2,1,1,1) for
(∂x u,∂y v,∂y u+∂x v,∂x w,∂y w). Both out-of-plane shears are included.
The pressure force is the transpose of the same integrated negative divergence.
The native affine patch balances that full stress against independently
integrated prescribed edge traction. This qualifies its assembly, rather than a
solved homogeneous free-cap condition or pointwise stress convergence.

During `inspect`, bottom mesh velocity is zero and cap mesh velocity is the
caller's physical horizontal/vertical cap velocity. Centroid velocities are
averages of their macro vertices. Shared edge split velocities differentiate the
actual intersection construction with checked dual-number arithmetic. Area
derivatives and mass rates come from those same moving microtriangles. A native
central finite difference rebuilds cap/bottom geometry independently and checks
these mass derivatives. It is an infinitesimal geometry consistency test, not a
finite conservative material trajectory.

Dual-face pieces integrate the continuous physical velocity minus this mesh
velocity. Contributions are aggregated into one oriented flux f_ij per periodic
node pair before donor momentum and dissipation are formed. All three carried
components use that same aggregated flux. `audit_shared_flux` recomputes it and
requires each pair and flux bit to match; a circulation that preserves all mass
marginals is rejected. There is no externally supplied accepted new height or
independent cap-donor velocity.

The diagnostic ledger checks

    m'_i + Σ_j f_ij = ρ b Σ_{T incident i} A_T (div u_h)_T / 3,
    Uᵀ K U = Σ_T μ b A_T (2 exx²+2 eyy²+sxy²+sxz²+syz²),
    Uᵀ Bᵀπ = −Σ_T b A_T π_T div u_h,
    Σ_i U_i·c_i = ½Σ_i |U_i|²Σ_j f_ij + D_adv.

The first is the full Reynolds identity. Its right side is zero for a strongly
divergence-free input. `geometric_identity_error` measures that identity;
`continuity_defect_max` separately reports m'+Σf and `divergence_max` reports the
actual element divergence. A deliberately divergent native example has a small
geometry error and a large continuity defect. The API does not accept it as a
physical incompressible step. Internal momentum-flux and strain/pressure force
resultants are reported separately; wall reactions and physical external work
are not hidden in an advancing update.

## Pressure image and numerical limits

The strip declares 8C+1 pressure modes for C columns. Constructor elimination
selects independent divergence columns and must return that dimension, or reject
with `PressureRank`. It uses partial row pivoting and an explicit relative rank
threshold; dependent columns are not silently turned into a pressure gauge.
Sparse Q terms are reconstructed from the original divergence coefficients.
Native tests check selected velocity columns against Q, positive pressure Gram
pivots, the implied full row rank of integrated B, and constant pressure
representation for C=2,3,4,6,8,12 in the tested geometry family.

This is finite numerical rank qualification. The dimension formula, rank
threshold and elimination pivot ratio do not establish a uniform inf-sup bound,
a spectral conditioning estimate or a mesh-family convergence theorem.
Unresolved rank/geometry cases reject. The persistent dense rank scratch is
bounded and counted, while Q and the element/face operators remain sparse or
matrix-free. A stronger topology-derived pressure basis and stability study
remain appropriate before a pressure solve is qualified.

## Memory, failure and cancellation

`FittedHeightPlan` checks every capacity product before allocation. Constructors
also enforce maximum columns, pressure-term capacity and a total payload limit.
Every vector uses `try_reserve_exact`; actual capacities, not just logical lengths,
are charged to the limit. Persistent rank scratch is included. On this qualified
64-bit target the nominal managed payload is

    1056 C² + 9576 C + 208 bytes.

For C=4 this is 55,408 bytes. It includes geometry, macro edges, gradients, R,
dual-face pieces/pairs, reserved sparse Q terms, pressure-column IDs, rank scratch,
masses, moving-geometry scratch, candidate diagnostic rows and triangle scratch.
There is no dense transfer map, dense global stiffness matrix, per-inspection
heap allocation or hidden history snapshot. Fixed stack matrices and Vec headers,
allocator bookkeeping, caller input/output buffers and the example's serialization
buffers are outside this managed-array payload. Caller diagnostic output alone
is 96 bytes per periodic node on this target. The API exposes the managed payload
so an eventual simulation facade must include it in its own total budget.

The geometry never mutates after construction. Nonfinite/nonzero subnormal
inputs or checked intermediates, nonzero product/division underflow, overflow,
invalid trace/shape, identity failure and cancellation reject. Scratch views may
be incomplete after rejection and are explicitly named scratch. Caller output
rows are copied only after every gate and the final callback, so they preserve
all prior bits on failure. Native tests cancel every inspection callback and
verify unchanged output and an identical retry. Constructor failure drops its
partial workspace without replacing an existing object.

## Qualification and next slice

The native contract suite covers mass/gradient partition, full-vector affine
traction, rotation/translation null strain, pressure image/constant mode,
material graph translation, changing-mass constant transport, actual rebuilt
geometry derivatives, nonzero adjoint pressure work, density/width/viscosity
scaling, arithmetic/shape/trace errors, capacity limits, uncertain rank and
marginal-correct wrong-motion rejection. Full default/core/desktop Rust tests,
formatting and strict Clippy are required, together with focused release tests.
Actual debug/release example binaries produce static geometry/diagnostic arrays;
independent normal/optimized Python compares them with the frozen rational
research assembly and rejects altered mass, flux, third shear, pressure force and
advancing-simulation scope. The figure uses those actual native arrays and is
labelled instantaneous, not a temporal replay.

The frozen 15 Lean identities remain conditional exact-real algebra. Their prior
compiled and negative-audited bytes are preserved. They are not Rust/IEEE,
geometry-assembly, stability or physical-validation proofs. No new Lean theorem
or pressure/free-surface solver is claimed by this implementation.

The next slice must select and qualify a physical space-time geometry/transport
integrator. A semidiscrete pressure solve must differentiate B(t)RZ(t)=0 and
include B'RZ; an endpoint pressure projection on independently changed geometry
is insufficient. The same path must supply integrated dual fluxes and the GCL,
and full momentum transport and force work must pass their own residual gates.
Only then can candidate mesh, velocity, pressure and derived phase exports
publish through one facade with a common physical time and rollback tests.
Uniform pressure stability, pointwise traction, temporal accuracy, surface
reconstruction/export integration, native advancing/rendered dynamics, variable
density, surface tension and adhesion remain unqualified. The advancing refusal
therefore remains in place.
