# Periodic fitted seam correction and scope erratum

This successor corrects the declared periodic Powell–Sabin geometry. It preserves
all bytes and commit identities of research evidence `2db90e8c` and native
assembly evidence `c995671c`. Those checkpoints are genuine algebra/numerical
evidence for the actual mesh they constructed, but their declaration of periodic
Powell–Sabin geometry and its rank-33 pressure image was incorrect.

## Actual mismatch and corrected construction

The frozen reference used the physical-boundary rule at both periodic seam
edges: split at the midpoint, then identify the nodes. On the reference strip,
its left seam point was (0,1/2) with mesh derivative (1/8,0). The neighboring
macrotriangle centers on the periodic covering plane are
P=(1/12,3/4) and Q=(-1/12,1/3). Their line intersects the seam at (0,13/24), not
(0,1/2). At cap speed 1/4 the intersection derivative is (13/96,5/192).

The intended [Powell–Sabin construction, section 1](https://arxiv.org/html/1904.05466v1#S1)
joins neighboring macrotriangle interior points through the shared macro edge.
A periodic seam has that neighbor across the period; it is not a physical
boundary edge. The native repair records the actual neighboring macrotriangle
on both seam edges, translates its centroid by the fixed period when computing
the left intersection, and represents the right intersection as its exact
periodic translate. Dual-number differentiation follows this construction,
including motion of both neighboring centroids. The fixed-period translation has
zero derivative. Physical cap and bottom edges retain midpoint splits and the
existing affine velocity trace constraints. The other geometry and operator
contracts remain unchanged.

The repaired rational reference independently constructs both intersections and
checks their exact translated equality. Native regression checks the seam line,
(0,13/24), its (13/96,5/192) derivative and matching right-seam motion. The previous
midpoint construction fails this direct declared-model contract. Merely changing
a rank constant would fail the geometry test.

## Changed rank and conditioning limits

With periodic neighbor intersections, the exact constrained divergence image and
integrated pressure matrix have rank 32 for the 4-column fixture, compared with
33 for the frozen midpoint seam. The alternating divergence relation at the
singular seam vertex is now exact. Native tests check the corresponding 8C mode
count on C=2,3,4,6,8,12 snapshots, positive pressure Gram pivots, the selected
velocity-column/basis correspondence and constant pressure representation.
The previous 8C+1 claim is superseded for the intended periodic construction.

This correction does not establish uniform pressure stability. Exact rational
perturbations of the two seam heights by 1/64, 1/4096 and 1/2^30 each restore
rank 33. These are pressure-algebra perturbation experiments only; no trajectory
or GCL claim is made for those deliberately altered snapshots. The additional
mode approaches dependence as the geometry approaches the singular seam. This
sensitivity is material to conditioning and to geometric roundoff, rather than
a reason to retain the incorrect midpoint rule.

Native geometry/gradients are floating point. The constructor checks the
numerical rank using its explicit tolerance and expected topology count; a
roundoff-sized extra mode is not an exact-real rank proof. Pivot ratios and
positive finite Gram tests are not inf-sup bounds. Actual element divergence
remains a separately measured diagnostic; a future pressure solver cannot infer
adequate physical continuity from an arbitrary pressure residual alone. No
published stability theorem is inherited for the added boundary trace restriction
or nodal mass lumping.

## Identities retained, notation clarified

The corrected full reference witness still has exact zero moving-dual GCL,
third-component momentum rate, finite transport energy residual and affine
traction residual. Liquid area remains 19/16, mass per velocity component 57/16,
and unrestricted affine strain power 475/512. There are still 22 nonzero nodal
mass rates and 76 nonzero shared flux pairs, with corrected individual values.
For the same nonconstant third-component fixture,

    D_adv = 23503996984881317 / 52895810764800000,
    Uᵀ K U = 2595531373 / 1259712000,
    T' = −132491397549700517 / 52895810764800000.

These are the constrained semidiscrete third-component witness, not a genuinely
coupled moving free-surface numerical solution. The affine stress patch uses
prescribed traction and is not a globally periodic homogeneous free-cap flow.
The smooth flat-cap predictor is an instantaneous analytic benchmark. Those
scope distinctions continue to apply.

Equation (3) in the frozen formulation should explicitly use unordered face
pairs. The correct finite donor identity is

    T(U_adv,m_new)−T(U_old,m_old)
      +½ Σ_i m_old,i |U_adv,i−U_old,i|²
      +½ Σ_{unordered {i,j}} |F_ij| |U_adv,i−U_adv,j|² = 0.

Each shared physical face is counted once, as in the implementation and rational
witness. An ordered sum over both (i,j) and (j,i) would double this dissipation.
No implementation energy factor is changed by this notation clarification.

## Qualification and continuation

The repair adds a direct seam regression to the native suite, a corrected
bounded rational reference, exact singular-vertex and rank-sensitivity tests,
and actual rejection of the frozen geometry by the intended seam contract.
The complete Rust feature matrix, focused release tests, independent normal and
optimized reference/native validations, actual binary hashes and corrected
native figure are frozen in `evidence/fitted-height-periodic-repair/` separately
from the source commit. All old evidence and claimed limitations are retained;
the predecessor is also verified in a detached checkout.

No new Lean theorem is claimed. The preserved 15 conditional finite exact-real
identities are still applicable to their explicit algebraic hypotheses; they
never proved the geometric seam or a physical solver. Native managed payload is
unchanged: 55,408 bytes for the 4-column workspace, including rank and operation
scratch. The pressure coefficient array now has 32 modes for that fixture.

The next work remains physical space-time geometry/full-momentum transport and
compatible pressure/strain solve qualification, including the differentiated
constraint B'RZ, actual integrated face provenance/GCL, numerical residual/work
gates, and a single bounded accepted/candidate facade. Uniform pressure
stability, pointwise traction, temporal accuracy and native advancing/rendered
liquid dynamics remain unqualified. Existing advancing-viscosity refusal,
accepted flow owners, phase geometry authority and all simulation clocks are
unchanged by this repair.
