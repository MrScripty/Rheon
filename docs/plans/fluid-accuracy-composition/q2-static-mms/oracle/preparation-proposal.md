# CV3-A independent preparation and uncertainty method v1

Prepared 2026-10-20. This document selects prospective algorithms; it records no
selected assembly, solve, spectrum, benchmark, test or campaign result. The
parent selected this validated route before oracle implementation. There is no
estimated-bound fallback. Prospective source is now prepared; numerical launch
remains pending final source/protocol/resource review, hashes and custody.

## Authority and source custody

Read the exact published PR141 head
`38c40ca7b4b2401e0ac802f12c647f7229072f52`, rather than a mutable local report:

| File under docs/plans/fluid-accuracy-composition | SHA256 of exact blob |
| --- | --- |
| reports/cv3a-admission.md | 8d2a8393c8af2dd3f7489f2c29076da3050c4616543d549fd88b8d1fd4cb8208 |
| reports/cv3a-review/recommendation.md | d54d52efd222b9defc7a37ec2efb43daaef7f31c7ff97c607c6d267e9187f69a |
| reports/cv3a-review/review.md | 2161bb6e08fd24dcd6e328b78d1842aca07a64709c2537951522e22812d097ee |

Dependency PR139 is head `902f6092de6044419d028f92dabd8e59ae7a87ba`, numerical
source `b0a9d52cc266cb651c5e67c07ec5f4f82419db55`. Read and preserve independent
CV2 helpers (not candidate imports):

| Helper | Original SHA256 |
| --- | --- |
| exact_polynomial.py | 4fe9c518beaa691ee6a1c5c5ab12ecf6cdfba79c83f93c482e988b203f7cf464 |
| exact_operators.py | 09e7d8ac268852b553557b8fa8f1e7661a6976727a996ed764a8481e8f5cee49 |
| high_precision.py | d17b2a452f874141a6051cbac33ef61313114b9a10fe0722552f4145a6d51f7f |

Their copies, if extended, receive this new owner's identity. Old sources,
outputs and archives are read-only. This owner writes only
`/workspace/rheon-q2-static-mms-independent`.

## Dimension replan disposition

Read the pre-edit replan at `bc82e92c277015a4944ef0dbd31f3505ba165c31`,
q2-static-mms/replan.md, SHA256
`89639b0df7c2f297be8173086abcd3b097acb0a1ba4ae16a9665a7b82acb3f31`.
Accept the bounded prospective extension: Layout n<=16, assembly and selected
Cholesky/QR velocity dimension<=544, pressure dimension<=256, checked allocation
arithmetic, and refusal beyond these limits. The support, spaces and formulas
stay the same; the scientific source identity changes. Record exact shared writes
and lead coordination, preserve frozen CV1/CV2 and old-case byte controls.

The full-grid bounds are nv=2n(n+1), nc=n^2, N=nv+nc. They are not actual cut
support counts. Elementary strict support inequalities give rectangle bounds:

| n | active velocities | pressure cells | KKT size |
| --- | ---: | ---: | ---: |
| 4 | 40 | 16 | 56 |
| 8 | 98 | 36 | 134 |
| 16 | 275 | 120 | 395 |

For n16, X has 13x11=143 and Y has 12x11=132. The fixed rotated square's box
width is strictly below .375; each phase is bounded by respectively
(nv,nc,N)=(32,9,41),(50,16,66),(128,49,177) at n4/8/16. These are source
inequality bounds, not executed support counts. Actual support/rank must still
be independently reconstructed. The conservative universal n16 limit is
(544,256,800). No CV3 baseline KKT factor is selected: the shared velocity limit
544 must not be represented as admitting an arbitrary 800-row baseline solve.

## Exact continuum and independent target

Use exact Fractions of represented source coordinates, grid primitives and mu,
never rational 1/10 for represented .1. Independently differentiate the declared
streamfunction and pressure. At the fixed units:

```
u=(x^2+2xy,-2xy-y^2), p=1+x+2y+xy,
sigma=(-p+4mu(x+y), 2mu(x-y), -p-4mu(x+y)),
f=(x^2+2xy-2mu+1+y,-2xy-y^2+2mu+2+x).
```

The symmetric stress tuple is xx,xy,yy. Integrate body phi*f and physical
traction phi*sigma*n separately by exact simplex/segment monomial moments, with
all component knots and pressure-cell splits. Body, boundary power, resultant
and torque remain separate. No discrete RHS manufactured from desired U/P.

R1 is the full exported rounded operator and actual exported rounded RHS,
including any rounding asymmetry. R2 is the exact independently reconstructed
integral target and continuum load. C=[A,-B^T;-B,0], A=M/dt+K. No gauge,
support deletion, symmetrization of R1, pressure shift or stabilization.

## Validated original-coordinate solution uncertainty

Generate an approximate inverse R and candidate z using independently pivoted
mpmath LU at fixed 120 decimal digits. Convert each actual finite mpf binary
tuple to a Fraction; printed decimal text is not the mathematical value. The
generation is untrusted for enclosure purposes. For each exact target C,b,
verify by exact rational arithmetic:

```
E=I-R*C; delta=max_i sum_j |Eij| < 1;
r=b-C*z; eta=max_i |(R*r)i|/(1-delta).
```

Then C is invertible and ||C^-1*b-z||inf<=eta, by the Neumann series. Every term
of the certificate is exact; the guard cannot be bypassed by Python -O. R1 and
R2 have separate certificates and output radii. A failed defect, singular LU,
nonfinite generator, allocation or resource cap is a refusal. Fixed precision
is not increased after data. No condition-number estimate substitutes for eta.
The proof is this explicit derivation; [Rump's verified numerical linear algebra
survey](https://www.tuhh.de/ti3/paper/rump/Ru10.pdf) supplies primary context.

Stream rows of R*C and retain only exact row sums, leveraging the exact sparse
columns of C. Do not form a dense exact defect matrix. Factor/inverse storage
and generation remain potentially expensive. The prepared policy generates
R1 and R2 separately, plus the H1 inverse; it does not adaptively reuse a target
or select a different precision/backend. Actual finite inverse/center generators
are retained with exact rational encoding and SHA binding for replay.

For native represented coefficient vector zhat, exact |zhat-z| plus eta bounds
the error relative to each target. Keep R1 solve error, R2 solve+assembly error
and R1-R2 perturbation distinct. Propagate velocity/pressure coefficient radii
through actual positive Q1/P0 basis and derivative integrals to volume and
physical traction norms. Because the component hats form a partition of unity,
a uniform velocity radius eps gives pointwise Euclidean radius sqrt(2)*eps;
use derivative-weighted support bounds for stress, rather than treating pressure
or derivatives as nodal velocities. No nominal full-cell measures replace cuts.

## Bounded physical norms

Select Euclidean-vector L1 and L2 for velocity and traction, scalar absolute L1
and L2 for pressure. Relative velocity/pressure denominators are norms of the
exact continuum field on the same fixed polygon, not coefficient norms or a
mean-subtracted pressure. Parent subsequently froze traction as
(integral_Gamma ||sigma_h*n-g||^q / length_Gamma)^(1/q)/tau_ref, for q=1,2.
Report normal/tangential components separately. This resolves the initial
normalization question prospectively, before any selected evaluation.

L2 squared volume errors are exact degree<=4 polynomial integrals. Traction
pieces use the correct liquid-side gradient/P0 pressure on every split edge.
For oriented edge d=b-a, let q(t)=(sigma_h-sigma_exact)*(dy,-dx). Then Euclidean
traction L1 is integral_0^1 ||q(t)||dt: the edge-length denominator cancels ds.
Traction L2 squared is integral ||q||^2 dt/|d|; it is generally not rational on
an oblique represented edge. Use exact polynomial moments and outward rational
square-root enclosures, not a claim that its entire result is rational.

For Euclidean L1 over a triangle, represent the degree2 error vector in its six
degree2 Bernstein coefficients b_alpha. Positivity and partition of unity give

```
||integral e|| <= integral ||e|| <= area/6 * sum_alpha ||b_alpha||.
```

The lower bound is the norm of the exact integral; each norm/square root is
outward enclosed. The scalar absolute case uses the same inequality; a common
coefficient sign resolves it exactly. Split the largest uncertain piece,
deterministically using edge-length and index ties; use exact dyadic midpoint
subdivision. The analogous 1D quadratic Bernstein bound treats edge traces.
This encloses Euclidean L1 without asserting it is polynomial. Freeze depth24,
at most 2,000,000 created pieces per case, 256-bit outward square-root endpoints,
and requested normalized integral width<=1e-8. Bounds and resource checks remain
always on. A piece/resource limit stops without replacing the result by Gaussian
sampling or dropping a difficult cut.

Convert rational q>=0 to a sqrt interval with m=floor(sqrt(q*2^512)), using
integer square root of the exact quotient floor: [m/2^256,(m+1)/2^256], with an
exact endpoint if m^2=q*2^512. Interval operations remain rational and outward.

For every deciding error and refinement difference, require total propagated
algebraic/integration uncertainty<=1% of its positive lower bound. A zero or
sign-uncertain gap is unresolved, not floored. Decide finest5% and order>=.9
using conservative error intervals. For a pair, the lower order is
log2(E8_lower/E16_upper); implement the .9 comparison with outward-enclosed
constants/logarithms or an exact integer-powered inequality, not a rounded
printed logarithm. Preserve all nine outcomes and both phase comparisons.
L-infinity remains a clearly named search/Bernstein-range diagnostic unless an
actual global enclosure is implemented and validated.

## Verified H1/pressure spectral diagnostic

Independently integrate exact Gv=integral(grad phi_i:grad phi_j+phi_i.phi_j),
Qp=diag(actual positive volumes), and B. Exact rational elimination gives rank;
retain every pressure row and constant-pressure mode. No beta cutoff.

Generate an approximate inverse RG of Gv and verify deltaG=||I-RG*Gv||inf<1.
Let epsG=deltaG*||RG||inf/(1-deltaG). Exact Gv^-1 is symmetric. With
Rs=(RG+RG^T)/2, a safe spectral inverse error is epsG plus
.5||RG-RG^T||inf. Consequently
epsT=||B||inf*||B||1*(epsG+.5||RG-RG^T||inf) bounds
||B*Gv^-1*B^T-B*Rs*B^T||2.

Enclose every Qp^-1/2 diagonal by the integer-square-root method. Let D0 be
the rational midpoint, epsD the largest radius, D its exact value, and
T0=B*Rs*B^T. The symmetric rational center S0=D0*T0*D0 has target uncertainty

```
epsS <= ||D||^2*epsT + epsD*(||D||+||D0||)*||T0||inf.
```

Generate an ordinary symmetric eigendecomposition V,Lambda of S0. Treat each
finite returned value as an exact rational. Verify omega=||V^T V-I||inf<1 and
F=S0-V Lambda V^T exactly. Polar orthogonalization implies a conservative
distance dV<=(1+omega)*omega/(1-omega) from V to an orthogonal Q. Thus Weyl gives
sorted eigenvalue intervals centered on Lambda with common radius

```
||F||inf + dV*(2+omega)*max|Lambda| + epsS.
```

Report their square-root intervals, smallest mode, eigenvector/residual and
alternating-cell-pressure pattern/Rayleigh quotient. A nonpositive lower
endpoint gives unresolved positive beta uncertainty; do not replace it by a
positive floor. Exact rank/Gram reasoning may establish mathematical positivity
without establishing a narrow measured beta interval. A single eigenvector is
not unique in a cluster; report an eigenspace/diagnostic pattern unless a verified
gap supports a directional interpretation. This is finite-instance evidence,
never a uniform inf-sup theorem. [LAPACK's error-measure definitions](https://www.netlib.org/lapack/lug/node75.html)
separate vector, matrix and subspace error, which must not be conflated here.

## Resources and readiness

Read-only metadata: cgroup16GiB RAM, CPU quota4, about21.85GiB free workspace;
mpmath1.3.0, numpy2.3.5, scipy1.17.0, sympy1.14.0 available. python-flint and
gmpy2 are not installed. No numerical benchmark was run. Metadata inspection is
not a performance measurement or an installed validated-linear-algebra backend.

Accept hard native caps: one worker,4GiB RSS,2GiB scratch,120s/case,900s total;
independent caps: one worker,8GiB address/RSS,8GiB scratch,1800s/child,
10800s per interpreter mode and32400s total for three complete modes.
Controls have a separate10800s total/2GiB scratch budget,1800s/child.
Set BLAS/OpenMP workers1 and use an external supervisor for RSS/time/output
caps, not merely internal Python checks. Process one case, and one inverse or
certificate phase, at a time; retain every partial receipt and refusal. Parent
must serialize native/oracle workloads to fit the shared machine and disk.

At universal800 rows a single binary64 dense matrix is5.12MB; actual largest
reference is395 rows. Ordinary dense storage is not the main risk. Arbitrary
precision LU/inverse, Fraction numerator/denominator growth, exact rank,
certificate products and L1 subdivision dominate. Sparse target multiplication
may reduce inverse-defect verification work from dense cubic cost to
N*nnz(C), but no speedup has been measured. Do not extrapolate a package author's
benchmark or claim the1800s bound is demonstrated feasible.

The exact-rational method is mathematically sufficient if it executes and its
certificates pass. Unmeasured runtime remains risk under enforced hard caps,
not an external blocker or reason for an additional approval/benchmark ritual.
No launch follows from this document alone: final source/protocol/resource
review and freeze remain necessary. A genuine cap/certificate/integration stop
retains its exact case/stage and returns for a reviewed backend/resource decision.
A future pinned
compiled ball/exact backend is possible but is not selected or installed here:
[official python-flint arb_mat documentation](https://python-flint.readthedocs.io/en/latest/arb_mat.html)
distinguishes bounded solves from its explicitly unbounded approximate option.
No implicit dependency or estimated-bound substitution is authorized.

## Required before numerical execution

Candidate-free source now includes exact target/load/reconstruction/physical
ledgers, certificates, all-nine structural guards and targeted wrong-load/
pressure/phase/field controls. Review and freeze all sources/resource commands
before raw inspection. This remains a prospective method/cap review, not an
executed evaluator qualification or source/campaign admission.
Normal/-O/-OO behavior is tested only after the separate execution gate; all
acceptance guards use explicit checks, not assertions. The parent accepted L1
limits/normalization and complete three-mode budget before data. Fixed spectrum
enclosures, error/gap propagation, cross-phase and bounded serialization are
implemented prospectively and require actual future checks, not assumed passes.
Display-only log arguments are outward2^-256 before the64-term series and
endpoints outward2^-128; all other rational report integers
are bounded at3,000,000 bits/1,000,000 decimal digits with explicit resource refusal.
