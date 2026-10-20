# CV3-A all-case prospective resource gate

Status: source-level resource design reviewed and frozen for preparation.
This records prospective ceilings, not measured runtime or execution admission.
No selected assembly, test, rank, inverse, spectrum or quadrature benchmark ran.

## Source-derived sizes

Full-grid upper bounds at n4/8/16 are respectively40/144/544 velocities,
16/64/256 pressure rows, and56/208/800 KKT rows. These bounds apply to every
fixture at that level, without a support count probe. Exact source/support
reasoning sharpens the rectangle counts:

| rectangle n | velocity | pressure | KKT |
| --- | --- | --- | --- |
|4|40|16|56|
|8|98|36|134|
|16|275|120|395|

For n16 square at both phases, the bounding box contains at most128 velocity
supports and49 primal cells, hence177 KKT rows; this is an upper bound, not an
executed active-space count. Thus395 is the largest actual-corpus KKT bound.
The old CV2 baseline is not selected, and its square-matrix ceiling544 is not
presented as permission for an800-row full-grid KKT baseline solve.

Native binary64 dense storage at the universal bound is2.37MB per544-square
matrix,1.11MB per544x256 block and5.12MB per800-square KKT. The selected solver
does not form the KKT or a Gram solve. Its work is Cholesky O(nv^3), triangular
velocity-pressure work O(nv^2*np), QR/reconstruction O(nv*np^2), then exactly
three corrections. Native geometry/quadrature/CSV buffers and factor diagnostics
dominate these small matrix byte estimates. A conservative common-patch bound
is1024 patches at n16 with up to6 fan triangles and9 points/triangle:55,296
points and442,368 local basis records before source-specific clipping reduces
them. These are storage/cost bounds, not measured memory use.

Independent rational operator assembly/rank may incur coefficient growth.
395-square multiprecision inverse/LU generation and exact rational inverse-defect
checks dominate reference cost. Dense defect work is O(395^3) products per
target; using the exact target's known zeros reduces work without changing
the certificate. Support overlap bounds each velocity A row by25 entries and
its B-transpose row by6; each B row has at most12 entries. Thus
`nnz(C)<=31*nv+12*np`, at most9,965 for the largest rectangle, and the
streamed395-row defect needs at most3,936,175 exact products per target.
The same-component H1 stencil has at most9 entries per row, so its275-row
defect has at most680,625 exact products. Highprecision inverse generation
remains cubic; these structural counts imply neither runtime nor bit-growth guarantees. A bit-growth/RSS bound and demonstrated time feasibility do
not follow from binary64 storage or a small residual. Bounded L1 subdivision
and certified H1 spectrum add separately owned limits. Unmeasured runtime stays
a declared risk, not a measured feasibility claim or an extra approval ritual.
The admitted campaign may later stop at these frozen caps with outputs retained.

## Inspected environment and proposed hard caps

The current cgroup exposes16GiB RAM and4 CPU equivalents; host reports5 logical
CPUs, approximately22GiB available workspace disk. Only cgroup limits govern
admission. Installed Python3.12 has mpmath1.3.0, numpy2.3.5 and sympy1.14.0;
python-flint is not installed. Rust uses the existing1.92 toolchain/Cargo lock,
one compiler job. No dependency or Cargo change is admitted.

| Stage | concurrent workers | address-space ceiling | new-output ceiling | per-case CPU/wall | total wall |
| --- | --- | --- | --- | --- | --- |
| native nine-case campaign |1|4GiB|2GiB|120s/120s|900s|
| independent case reference, each interpreter mode |1|8GiB|8GiB shared across modes|1800s/1800s|10,800s/mode;32,400s for normal/-O/-OO|
| evaluator controls/aggregation |1|8GiB|2GiB|1800s/1800s per subprocess|10,800s|

Reserve remaining memory/disk for toolchain, lead-independent work and custody.
All three complete scientific interpreter reports use the same single native
corpus; full reference calculations are serialized in each mode, without
substituting aggregation of cached results for actual mode checks. The larger
prospective total follows this unchanged evidence requirement, before any data.
Raw native/reference/control caps sum to12GiB; reserve2GiB for the eventual
archive and2GiB for headroom (16GiB new allocation against about22GiB free).
A final archive exceeding2GiB is a custody refusal, not permission to delete
raw evidence or silently omit payloads. Native/reference/control workloads
never run concurrently. Negative controls copy one fixed n4 case, not nine
whole-corpus trees, and output links are refused by the resource supervisor.
Enforce CPU/address-space/per-file limits on children; parent owns wall timeout,
aggregate-output/RSS observation, process-group cancellation, wait/reap and
failure receipts. Stop at a cap, retain partial outputs and report the exact
case/stage. No automatic retry, precision/backend fallback or later-level skip.
Fresh output creation must fail on existing paths, never overwrite earlier data.
Binary/source/oracle/manifests and actual argv must be frozen before launch.

## Preparation disposition

The numerical-reference candidate and actual bounded L1/spectral/rank algorithms
are complete and source-reviewed. preparation-freeze.md and commands.json bind
all-case sizes/costs, supported algorithms, actual source hashes/commands and
enforceable CPU/RAM/output limits. scipy1.17.0 is also present for the untrusted
eigenvector generator; generator accuracy is never assumed.
Resource-design admission does not assert that every reference will finish in
its cap: the contract explicitly permits a retained resource refusal and then
replanning. No additional benchmark or user approval is required solely because
runtime is unmeasured. Known infeasibility or a mathematical inability to bound
the chosen uncertainty would block launch; neither has been established here.
The current handoff is preparation only, so no scientific corpus is run to
manufacture a feasibility result. If the later admitted campaign cannot establish
the guaranteed uncertainty within these caps, return to the lead for a separately
reviewed backend/protocol decision. The1% bound and5%/.9 physics criteria remain unchanged.
