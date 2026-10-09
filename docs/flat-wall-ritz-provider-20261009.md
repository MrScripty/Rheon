# Bounded body-force-driven flat-wall Ritz source candidate

Base: `5c3005d10e94de9e1757d8c9417935ea814a4d7e`. This is new implementation,
not recovered source. Physical qualification is pending. The held PR25 case41,
geometry-band and native N24 campaigns remain untouched.

The implementation supplies a small reduced steady provider and a separate
physical viscous wall observable. It changes no retained pressure face, mass,
old shear/action/wrench fixture, scientific tolerance, pressure or timestep API.
The symmetric-stress energy motivation is Batty and Bridson's original
[viscosity paper](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf).
This trial space, source and wall reconstruction are newly derived choices.

## Exact finite model and represented implementation

Only counts N6/N9/N12, spacing stored `3/N`, origin0, domain `[0,3]^3`, retained
solid `[1,2]^3`, SI rho=mu=1 are admitted. Put m=N/3. A z-oriented potential q,
units m³/s, lives at x planes m+1..2m-1, y planes 1..m-1, z layers m..2m-1;
all other potential values are pinned 0. Each basis has four oriented face flux
incidences. Its velocity coefficient is the incidence divided by that face's
**stored open area**. The continuous consistency target for q is the z-layer
integral of the streamfunction, not the streamfunction itself.

For exact positive stored areas and exact reciprocals, integrated D*C=0 follows
from opposite incidence cancellation at each cell. Native reciprocals, products
and summed velocities are rounded; `flat_wall_divergence` reports both actual
integrated divergence and an outward arithmetic enclosure for the rounded
stored velocity. It is neither an exact-native identity nor spatial convergence.

The full selected roster consists of all three normals in the patch and both
directed gradients of every included XY/XZ/YZ fluid quadrant, including flat
stationary trace and reverse Uz rows. It has q/row/block counts2/176/100,
12/681/381,36/1680/936. Coverage inspection visits every global supported normal
and internal cross sector for **every trial coefficient**. Any nonzero omitted
row refuses. At any unsupported three-fluid-quadrant corner, all four surrounding
component-face samples must be structurally absent from every C column. This
admits zero corner energy only on this restricted space, not arbitrary corners.
Outer engineering shear is eliminated by sealed normal traces and even free-slip
continuation; outer tangential values need not be zero. This explicit model does
not extend the parent gradient's outer-edge API.

Stationary wall traces are exactly 0; flat rays retain actual center-to-wall
distance and sector volume. Normal flux pinning is not a continuum tangential
no-slip density/coercivity proof. Complete positive normal-Y energy rows give
finite-model uniqueness: zero energy forces Uy constant along each vertical
chain, zero endpoint Uy implies Uy0, then x incidence and pinned x endpoints
force q0. Physical consistency and boundary derivative convergence remain open.

The matrix is assembled from immutable parent row coefficients and matched
blocks: `Aq=CᵀKC`; normal contribution `2µV g_i g_j`, shear contribution
`µw(g_ab_i+g_ba_i)(g_ab_j+g_ba_j)`. No raw-field gradient substitution or mutable
snapshot action is added. A single bounded LDLᵀ attempt refuses nonpositive
pivots/nonfinite arithmetic/cancellation; it never symmetrizes, regularizes,
drops a degree of freedom or retries. Construction does not solve.

## Physical source and owned acquisition

The provider receives only component-tagged polynomial body-force density terms
and required physical/provenance inputs. There is no analytic velocity, stress,
reference load, interpolation or reference-solution argument. Source support is
the lower strip `[1,2]x[0,1]x[1,2]`. Declared force is analytically integrated
over represented MAC duals intersected with that support. The clipped geometric
volume is reported separately from stored pressure-face `A*d`; neither rho*A*d
nor geometry is replaced. Source intervals concern exact stored coefficients
and represented coordinates only, with subnormal endpoints permitted solely as
interval bookkeeping. Outward IEEE arithmetic is tested against exact rational
integrals; it is not a kernel proof of IEEE correctness or a PDE-error bound.

`tools/flat_wall_force_source.py` independently emits the benchmark **forcing
coefficients only**. With X=s⁶(1-s)⁶, Z=X(t)(t+1/2), Y=y⁴(1-y)², A=12012²/2,
its source is fx=-A(X''ZY'+XZ''Y'+XZY'''), fy=A(X'''ZY+X'Z''Y+X'ZY''), fz0.
No generated data is tracked. The standalone native example only parses these
coefficients. Exact independent reference comparison happens after its process
exits. The continuum viscous reference about `(1.5)^3` is F=(-1,0,0)N and
T=(0,-1/60,-1/2)Nm; it is not an argument to any native action.

After an explicitly requested solve the provider owns actual q/Cq values. A
source/head/binary/problem/boundary/geometry/C/matrix/RHS/factor/field journal is
hashed before `acquire_initial_state`. That method copies these numerical values
into immutable `ObstacleFlowState` as supplied initial data acquired from this
reduced steady solve, time/generation 0, Unknown errors and no pressure. The
TransientStokes tag is a storage envelope, not a claim of evolution from rest
or a full Stokes pressure solution. References are caller declarations, not
authentication. A supplied synthetic q unit control is distinctly labeled and
never reported as a numerical physical solution.

## Physical viscous traction and failures retained

`flat_wall_owned_traction` reads the actual owner, validates its entire trial
footprint, and integrates lower-Y fluid-on-solid traction with normal `-e_y`.
Tangential x P1 uses stationary0 and the actual wall distance; x hats use actual
mass and first moment, including their nonuniform centroid. Normal stress uses
every x/z cell area and geometric first moment. Normal traction is never erased
using reference pressure 0. Uz and endpoint/other-face zeros require structural
support checks; unsupported nonzero samples refuse rather than snap/fall back.
It reports all six components, arithmetic-only intervals, full wall area, active
P1 basis area and sector-derived effective area separately.

The unchanged first-normal-row P1 baseline gives `Tz=(1/2-h)Fx` on ideal/dyadic
levels. N6 torque is identically 0. Optional normal-only P2 uses the wall 0 plus
actual values at first two normal planes and actual Lagrange weights. With
tangential P1 retained, N6 gives `Tz=-Fx/2`, the wrong sign for negative force.
Both laws are regression tests, not acceptance targets or promised repairs.
N9 actual geometry uses actual distances/areas, not those ideal identities.

## Allocation, verification and execution boundary

No physical N6/N9/N12 solve is authorized in this implementation turn. Unit
checks use arbitrary q, source moments, unrelated small synthetic SPD systems
and rest construction. Generated outputs stay outside Git. The native
`--algebra` mode cannot call the provider solve. `--physical` performs exactly
one opt-in steady solve at its specified admitted level and therefore requires
separate explicit campaign approval. There is no default solve mode.

The envelope remains 10,000,000 B, inside unchanged 16,000,000 B. Constructors gate
planned and actual Vec capacities. Provider accounting includes geometry once,
source terms, requests, C columns, three complete face packs, row scratch, dense
matrix/factor, RHS/q, fixed metadata, copied rest/acquired owner, gradient rows,
stress blocks and temporary sorting indices. Coefficient owners end before
candidate owners begin. LDL, curl, source integration and traction allocate no
dynamic work buffers. Runner adds actual argument/q/control capacities, a 64 KiB
bounded working allowance and two 8 KiB I/O buffers, subtracting that allowance
from the provider limit before construction. Stack, allocator metadata and
process RSS are explicitly excluded; this is managed payload accounting.

The separately approved campaign launcher adds an aggregate 1080 s deadline,
bounded pre-write producer slots totaling 4 MiB and a 256 MiB RSS threshold sampled
at 50 ms. The latter is not a continuous hard memory bound. Native and comparison
levels retain 180 s deadlines, and Git checks get 10 s deadlines, all inside the
aggregate budget. Native artifact sets retain their shared 1 MiB writer cap;
comparison retains 16 MB managed-object gates. Full enforcement, cleanup caveats
and exact output reservations are documented in
[the execution-guard derivation](flat-wall-campaign-guards-20261009.md).

Physical success requires signed force/torque approaching the fixed continuum
reference on three actual solved fields, decreasing dimensional vector errors
and arithmetic/source/algebraic uncertainty too small to explain improvement.
Projected and full active-face discrete residuals, action/matrix mismatch,
velocity/divergence and surface errors stay separate. A continuous strong PDE
residual is unavailable without separate reconstruction qualification. H1 energy
or a small projected residual alone cannot control wall derivatives. Three
coarse cases can fail or be inconclusive; application absolute load budgets
are unspecified. No physical capability/qualification flag changes here.

The original supplied-field force/torque failures remain failures. PR25's
baseline/six-FD/E2/memory-reader integrations, geometry-band/refinement/retry
campaigns and N24/43,321,344 B refusal remain held; this source cannot close or
repackage them. No pressure, timestep, main merge, deployment or publication is
part of this implementation checkpoint.
