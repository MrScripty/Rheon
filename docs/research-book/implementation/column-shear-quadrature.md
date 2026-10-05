# Liquid-only momentum mass and shear quadrature prerequisite

This companion derives a variational route for the existing bounded column geometry and implements one exact model restriction: **flat, fixed free-surface tangential shear**, constant density, periodic lateral directions, and zero shear traction at the bottom and free surface. `ColumnShearWorkspace` reads a `ColumnSurfaceView` from the existing geometry owner and caller-provided MAC fields. It stages a checked output without owning accepted velocity, pressure or phase. **The moving-surface carrier facade is not extended.** Its existing viscosity rejection for surface modes remains intact.

The earlier Newtonian filled-box milestone (`0cc70a698232be1ed3c7e115c028856cd2de063c`, evidence `d7e48c38e908ba1e5c618105936673b91c02a93e`) is frozen. Its retained 2053-update f32 accuracy failure remains unchanged. The current prerequisite does not apply that filled-box stencil to partial cells or infer adhesion from damping.

## General geometry, momentum and stress contract

For column footprints C_c with areas A_c and heights H_c, an explicit geometry interpretation is

    Ω_h = union_c [C_c × (0,H_c)].

Its volume is sum_c A_c H_c, matching the authoritative canonical column fractions. This is a union of vertical prisms with stepped lateral height transitions. The current pressure implementation interpolates height crossings between cell centers; it is not already a complete finite-element description of this union. Choosing a smooth/bilinear surface instead would require different geometric integration and explicit volume reconciliation.

Let N_i be nonnegative scalar velocity basis functions on this domain, with component vectors ψ_i and partition of unity on each unconstrained component. The consistent momentum mass and a row-sum lumping are

    M_ij = ρ ∫_Ω N_i N_j dV,
    m_i = sum_j M_ij = ρ ∫_Ω N_i dV.

Positive liquid-support measure gives positive m_i; zero-support nodes must be excluded rather than divided by zero. A node whose sample point is in air can still have basis support intersecting liquid. Center-wet classification alone is therefore insufficient to derive a general momentum space. Basis behavior at prescribed wall degrees of freedom also affects mass lumping and requires an explicit boundary construction.

For Newtonian stress τ = 2μD(u), D(u) = (grad u + grad u transpose)/2, define

    K_ij = ∫_Ω 2μ D(ψ_i):D(ψ_j) dV.

With fixed geometry and nonnegative quadrature weights, a sampled version has K = E transpose W E. It is symmetric, and u transpose K u is nonnegative dissipative power. Integrating the weak momentum equation produces volume stress work and the boundary term ∫_∂Ω w·τn dS. Zero traction makes the latter vanish. Prescribed traction or moving walls instead supply explicit work; they cannot be hidden in a damping coefficient. A general implementation must integrate the same liquid geometry for momentum and strain, include all coupled strain components, and account for every prescribed/free boundary.

The full free-surface balance involves (-p I + τ)n. [Batty and Bridson, Sections 4–5](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf) motivate natural traction and variational viscous discretization, while distinguishing a pressure/viscosity split. The [Basilisk viscosity source](https://basilisk.fr/src/viscosity.h) shows coupled stress terms in an independent implementation. These primary sources support the research direction; their validation is not a qualification of Rheon's geometry or stencil.

## Implemented restriction and its basis

Choose a column-normal coordinate y, a common surface height H, a periodic lateral patch of area A, and velocity

    u(y) = U_0(y) t_0 + U_1(y) t_1,  u_n = 0.

The two tangential axes are independent only because all lateral derivatives and normal velocity vanish. Divergence and self-advection vanish, normal viscous stress is zero, and the nonzero tensor entries are D_nt = D_tn = U_t'/2. Thus 2μD:D = μ[(U_0')²+(U_1')²]. The scalar-looking operator is an exact restriction of symmetric strain to this velocity class, not a general decoupled free-surface approximation. The free surface stays fixed. A constant pressure equal to atmospheric pressure is compatible with this restricted continuum model; no native pressure solution is supplied or published by this operator.

At spacing h, use only centers y_j = (j+1/2)h strictly below H. For a column height `full + fraction`, the number L of wet nodes is

    L = full + indicator(fraction > 1/2).

Interpolate each U linearly between consecutive wet centers, and extend it constantly from the first center to the bottom and from the last center to H. The basis is nonnegative and partitions unity. Both endpoint shear tractions vanish in this trial space. There is no exterior air edge, prescribed ghost velocity, or bottom wall-damping term. For L = 1, the basis is constant across the entire liquid interval, so the shear operator is zero.

Integrating the basis gives the exact liquid-only row-sum mass lengths

    ω_j = h                    for j < L-1,
    ω_(L-1) = H-(L-1)h,
    m_j = ρ A ω_j.

Their sum is H, hence total mass is ρAH. If the top fraction is <= 1/2, a partially filled air-centered cap belongs to the last wet node's support. For example, with `full=2, fraction=1/4`, lengths are (h,1.25h), not (h,h). With fraction=3/4, lengths are (h,h,0.75h). This is a change in momentum weights derived from the basis, not damping assigned to an arbitrary partial-cell fraction.

For each interior wet-center interval, the strain quadrature is exact because U' is constant:

    P(U) = (μA/h) sum_(j=0..L-2) (U_(j+1)-U_j)².

Each edge has stiffness k = μA/h. It transfers force k(U_(j+1)-U_j) to node j and its negative to node j+1. These are integrated nodal forces in N; total force cancels in exact arithmetic. The endpoint rows contain one edge, interior rows two. A sampled affine profile produces constant interior shear and opposite endpoint relaxation forces, because its constant endpoint extensions do not represent a maintained Couette boundary condition. Translation is a zero-strain mode and produces no force anywhere.

This basis selection changes discretely when a center becomes wet. The total liquid mass remains continuous, but the number of velocity degrees of freedom and the local mass distribution change. Dynamic geometry would require a conservative momentum remap and an account of energy/work caused by changing mass. Neither is implemented here.

## Explicit step, bounds, and diagnostics

Here M = diag(m_j) is the **lumped** mass, not the consistent mass matrix M_ij above. Its kinetic energy approximates the continuum integral for nonconstant profiles; exact liquid mass and the declared discrete energy are the checked quantities. The fixed-geometry update is

    M(v-u) = -dt K u.

For d_j = sum adjacent stiffnesses, the checked number is r = dt max_j(d_j/m_j), required <= 1. Positive edge weights then make every coordinate update a convex combination of itself and its neighbors. Also, x transpose K x <= 2 max_j(d_j/m_j) x transpose M x, giving the familiar explicit energy bound. The implementation rejects an excessive dt and performs no automatic substeps.

With δ = -dt M^-1 K u and kinetic energy T(u) = (1/2)u transpose M u,

    T(v)-T(u) = -dt P(u) + (1/2)δ transpose M δ.

The report includes actual stored-f32 rounding work, weighted momentum before/after, rounding momentum, internal force sums and the mass-volume partition. Compensated sums and explicit 64 binary64-epsilon budgets qualify these identities; no existing production gate or historical accuracy tolerance changes. Raw convex bounds are checked before and after f32 conversion. Nonzero subnormal intermediates, underflow-to-zero multiplication/division, nonfinite arithmetic, and nonzero subnormal stored candidates reject the operation. These are finite-scale restrictions, not an IEEE correctness proof.

The output copy begins only after every shape, field, geometry, coefficient, stability, force, momentum, energy and final-cancellation gate passes. Input normal velocities must be zero, tangential fields must be exactly uniform on each normal layer, and dry fields must be zero. Lateral periodic duplicate faces must agree. Varying heights, nonuniform fields and geometry mismatches reject. No tolerance is used to coerce an unsupported field into this model.

The workspace owns five normal-count f64 vectors: one mass vector, two force vectors and two staged candidate profiles. Nominal payload is 40N bytes; actual capacities are charged against its explicit cap. It allocates no arrays per update. Its optional read-only mass/force accessors expose **working scratch**, which can be incomplete after rejection; they are not accepted state or independent geometry authority. The report's geometry stamp is borrowed from the supplied column owner.

## Native qualification and independent numerical evidence

Six Rust contract tests cover the hand-derived partial-mass matrix on all three axes, affine internal stress and endpoint force, translation, both sides of the center classification and a single-node limit, all callback occurrences with retry equality, unsupported fields/geometry, scale failures and excessive explicit steps. The analytic refinement regression uses liquid-only weights.

The native example emits 24 cases and 1298 intervals, with all nodal masses, forces, before/after profiles and diagnostics recorded. The decay mode is U(y,t) = cos(πy/H) exp(-νπ²t/H²), with a second tangential component at half amplitude, H=1 m, ρ=3 kg/m³, μ=0.15 Pa s and T=0.1 s. Both endpoint derivatives vanish. Periodic lateral directions and zero normal motion make this an analytic fixed-flat-free-surface shear model. It is not a moving liquid property demonstration or material calibration.

For top fraction 0.25, native liquid-weighted RMS errors at full layer counts 8,16,32,64 are approximately 1.672e-4, 3.885e-5, 9.734e-6, 1.891e-6 m/s. Fractions 0.5 and 0.75 also refine. A separately assembled dense generalized operator provides both a double-precision explicit-step reference and an exact-discrete exponential reference. At fixed geometry, the latter isolates time error: 1.2684e-5, 6.3404e-6 and 3.1698e-6 m/s for 64,128,256 steps in double explicit reference. The actual native f32 errors against the same exact-discrete reference are 1.2653e-5, 6.4003e-6 and 3.1668e-6 m/s. Both sequences approximately halve. Continuum error is distinct and eventually limited by geometry and f32 rounding. The largest native/double-reference difference in this packet is about 1.661e-6 m/s. These observations are restricted benchmark evidence, not a general convergence theorem.

The independent Python gate checks finite values before any maxima, derives every liquid-only mass and internal force, verifies each actual f32 update, exact profile carry-forward, raw convex bounds, force/momentum/energy budgets, final face fields, and native grayscale diagnostic pixels. Twenty-eight tampered cases exercise nonfinite values, full-box mass substitution, balanced-but-wrong nodal forces, altered geometry/units/traction claims, ledgers, pixels and missing records under normal Python and `python -O`. The earlier filled-box f32 negative result remains separately frozen and bound in the historical preservation inventory.

## Conditional proofs and unresolved integration

`ColumnShear.lean` proves fifteen exact-real statements: dual-length partition, conditional cap-mass positivity, row-sum mass algebra, edge and three-node force cancellation, three-node dissipative work, nonnegative dissipation, translation force, weighted momentum from coordinate update equations, old-velocity work from those equations, weighted energy expansion, conditional energy nonincrease, rounding-momentum algebra, zero endpoint traction force, and two-neighbor convex bounds. All are compiled and audited against the original pins in an isolated project. Real injected `sorry` and extra-axiom variants are rejected. Historical proof files and inventories remain unchanged.

These statements do not prove basis integration, arbitrary-node or varying-height tensor assembly, mesh topology, spectral bounds for a general cut operator, dynamic mass remapping, pressure compatibility, Rust/IEEE refinement, convergence, or material calibration. The exact statements, actual kernel checks, and physical continuum model are separate qualifications.

The existing carrier still prescribes zero normal velocity on all outer box faces and uses full face momentum weights in pressure correction. This shear prototype has periodic lateral boundaries and liquid-only masses. Copying its output into that sealed carrier would change the boundary model and destroy the derived compatibility. Moreover, the current column pressure crossing geometry differs from the stepped-prism interpretation above. A coupled successor must define the same geometry/basis for pressure, derive a mass-compatible divergence/gradient pair, handle changing velocity support conservatively, and assemble all coupled strain components with normal/tangential surface traction. Until then, surface viscosity in `LiquidTransportSimulation::step_viscous` remains rejected.

This successor also repairs the precise CI path-filter omission reported by review: both pull-request and main-push filters now include the frozen viscosity verifier/tests/evidence and this new prerequisite's equivalents. The existing isolated Pillow install adds pinned NumPy 2.3.5 for the new dense numerical verifier. No unrelated workflow behavior, frozen evidence, PDF, research chapter or proof inventory is changed.
