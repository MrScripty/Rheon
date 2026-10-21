# Independent numerical verification and experimental validation design

Status: proposed acceptance design; no new solver, harness, dataset import, or
submarine simulation was executed for this report. Source review date:
2026-10-14. Baseline: `main` at
`06eb5daee9e77e2b0843a5ee0a4105f6950e9b1b`.
Acceptance-design author: `/root/validation_design`. Active I1 independent
acceptance executor: `01a139a7-fc1b-7384-9200-e10dd033f016`; implementation owner:
`01a139a7-da3b-71b3-b155-ddca2f780005`. Parent owns serial integration. These
assignments admit only the canonical existing disjoint I1 write sets and retain
a separate acceptance executor with authority to reject a milestone. A solver
author's self-report is input to review, not acceptance authority.

## Claim boundary and admission

The objective is trustworthy incompressible CPU external-flow results for
prescribed forward travel and ascent. Fully submerged single-phase flow,
prescribed motion, constant density and Newtonian viscosity are the initial
model. Low-Reynolds numerical qualification cannot establish full submarine
turbulent capability, near-surface ascent, cavitation, propulsion, compressible
flow, or free six-degree-of-freedom dynamics. None is accepted here.

Reviewed actual owners and evidence: `src/pressure.rs`,
`src/obstacle_pressure.rs`, `src/obstacle_shear.rs`, `src/forces.rs`,
`experiments/aligned_stokes.rs`, `experiments/plate_coupling.rs`,
`tests/forces_contract.rs`, `tests/obstacle_flow_contract.rs`,
`tests/aligned_stokes_contract.rs`, `tests/fixed_bottom_ale_contract.rs`,
`docs/plans/moving-plate-coupling/plan.md`, and research-book chapters 18 and the
constitutive validation research record. Existing tests address component
gauges, compatibility, physical cut-layer coefficients, bounded projection,
reduced shear and coupled plate work. The aligned Stokes experiment rejects
nonzero wall speeds; plate coupling preserves normal geometry. Neither proves
open external flow or normal moving embedded geometry. Existing executed
evidence remains scoped to its original reports; this source inspection is not
a fresh regression run.

The planned independent harness is justified by reachable defects that current
tests cannot decide: an affine pressure-load bias can produce a plausible drag
with converged pressure; open-boundary bookkeeping can hide momentum loss;
accelerating frames can report fictitious pressure as physical load; and moving
geometry can create mass. The harness will own exported-record acceptance,
analytic expectations, refinement calculations and experimental comparisons.
It will not duplicate the solver, geometry builder, source policy or API owner.
Review its retention when these claims are subsumed by another independent
oracle. This follows Coding-Standards `188beda1`, Core/Router,
Verification and its Independent Test Oracles detail, and Documentation.

| Milestone | Required independent evidence | Current disposition |
| --- | --- | --- |
| I1 fixed synthetic axis-aligned body, steady creeping Stokes, straight/oblique uniform external flow | `stokes`: open fluxes, no-slip, manufactured solution, pressure/shear/torque quadrature, force conservation and mesh/domain studies; no physical time gate | Pending; N1/N2 tiny discrete acceptance and N3 continuum load qualification are separate; example/harness absent at reviewed baseline |
| I2 finite laminar-Re stationary or constant-velocity body frame | `stationary`: inertial manufactured solution, convection conservation, laminar reference and wake quantities | Pending; no inherited turbulence claim |
| I3 smooth prescribed forward/ascent in accelerating body frame | `translation`: acceleration source, physical-pressure recovery, time histories and frame controls | Pending; constant-velocity equivalence alone is insufficient |
| I4 prescribed moving embedded body in world frame | `gcl`: swept volume, remapping/topology, moving wall impulse/work and frame comparison | Pending; reduced tangential plate tests cannot close this gate |
| I5 matched physical experiment | `experiment`: approved provenance, experimental/input/numerical uncertainty and predeclared accuracy targets | Pending future capable-model and data-rights gates; ingestion remains unavailable until authorized matched data |
| I6 optional prescribed attitude | `attitude`: rotating-frame sources or world-frame rotation, torque, power and rotation/refinement controls | Deferred; translation tests do not qualify rotation |

## Future executable procedure and evidence contract

The [first-slice handoff](first-slice.md) owns the proposed native/runner CLI.
These are proposed commands to implement, **not existing executable examples or
test results**. `examples/submarine_stokes.rs` supplies I1 real native records;
`validation/submarine/run.py`, `oracle.py`, `test_validation.py` are proposed
independent verification files. Keep one runner rather than several unrelated
tools. A missing example, suite, export, oracle or required observation must
exit nonzero and report `unavailable`; never turn it into a passing skip.

```bash
cargo run --locked --release --no-default-features --example submarine_stokes -- --case tiny-straight --output .generated/submarine-i1/straight
cargo run --locked --release --no-default-features --example submarine_stokes -- --case tiny-oblique --output .generated/submarine-i1/oblique
python3 validation/submarine/run.py --suite stokes --source <implementation-revision> --output .generated/submarine-i1/validation
python3 validation/submarine/run.py --suite stationary --source <implementation-revision> --output .generated/submarine-i2/validation
python3 validation/submarine/run.py --suite translation --source <implementation-revision> --output .generated/submarine-i3/validation
python3 validation/submarine/run.py --suite gcl --source <implementation-revision> --output .generated/submarine-i4/validation
python3 validation/submarine/run.py --suite experiment --source <implementation-revision> --output .generated/submarine-i5/validation
python3 -m unittest discover -s validation/submarine -p 'test_*.py'
python3 -O -m unittest discover -s validation/submarine -p 'test_*.py'
```

The runner builds/executes the pinned native source and captures origin in a
parent-controlled manifest separately from numeric acceptance. An optional
`--input` record-analysis mode requires that manifest for an execution-origin
claim; imported records alone can prove only their numeric properties.
The real implementation example can expose different options only through an
explicit coordinated update of producer and consumer contract. The runner must
accept actual native outputs; regenerated analytic fields cannot stand in for a
solver solve/advance. Exports must contain each accepted solve or timestep,
initial conditions,
boundaries and force definitions, not only a final drag or a solver's `passed`
flag. Independent checks recompute observations from these records.

Steady I1 exports an accepted solve identity and no invented physical timestep,
trajectory clock or accepted horizon. Time-dependent items below apply only
to later suites. Each evidence manifest must record:

- Exact source revision and dirty state, suite/case identifier, executor,
  actual command, compiler/toolchain, CPU/OS, binary64/scalar or parallel path,
  worker count, environment and terminal outcome. Initial qualification is
  Linux x86_64 Rust 1.92.0; additional supported platforms need their own evidence.
- Equations including convection/turbulence switches; SI units; density,
  dynamic viscosity, body volume/centroid, reference length/area, gravity,
  coordinate axes, pressure units and reference, body pose/velocity/acceleration,
  normal convention and frame. Explicitly label pressure potential versus Pa.
- Exact mesh/geometry identity, counts/spacings, solid surface, open areas,
  fluid/dual volumes, component IDs/gauges, face orientations, stress points,
  timestep history, accepted horizon, solver tolerances/iterations, all
  boundary operators and prescribed values. Include relevant old/new geometry,
  swept volumes and remap transfers at I4.
- Velocity, pressure, fluid momentum, boundary mass/momentum fluxes, pressure
  and viscous surface tractions, force/torque, actuator impulse/work and
  accepted/refused state. Label independently recomputed values versus native
  reports. Retain cancellations and refused steps separately from accepted data.
- Per-claim status `pending`, `blocked` or `satisfied`, and diagnostics
  `invalid`, `unsupported` or `unavailable` where applicable; numerical
  thresholds, measured values, raw refinement series and uncertainty method.
- For physical references, originating report and case/page/table/station,
  acquisition record, rights decision, source bytes hash, units/conversions,
  measurement uncertainty and normalization. Hashes establish source identity,
  not correct physics or permission to redistribute.

Use explicit `failure_kind`, `claim_id`, case/step and measured/allowed fields
for failures. Validator tests begin with one valid record and mutate one
condition: wrong force sign, pressure offset leakage, omitted viscous traction,
missing outer flux, omitted acceleration correction, omitted swept volume,
wrong units, missing uncertainty, altered geometry identity, and failed-step
publication. Each must reach and identify its intended failure. Test the
refinement estimator with independently derived midpoint integration of `x²`
(error `1/(12 n²)`), a constant-biased control, noisy/nonmonotone series,
near-zero quantities and incomplete data. Do not import published example code.

## Metrics, units and numerical acceptance targets

The following values are project-proposed engineering acceptance limits,
not ITTC/ASME prescribed tolerances or measured performance. Freeze them before
running a candidate. A revision requires owner-reviewed rationale and a fresh
comparison; failure is not justification for silently weakening a target.

Set a case reference `L`, positive `U_ref` and force scale in its input. For I1
use `F_ref = μ U_ref L`; for I2 onward also report conventional
`q = ρ U_ref²/2`, `C_F = F/(q A_ref)` and
`C_M = M/(q A_ref L)`. At rest retain the case's positive scale or use the
specific hydrostatic/acceleration scale; never normalize by the observed zero
load. Report dimensional values alongside normalized ones. Small quantities
use absolute normalized error, not percentages divided by cancellation.

| Claim | Observable metric | Proposed blocking limit |
| --- | --- | --- |
| V01 discrete pressure/viscous equations | Independent residual in every active row, including retained component gauge/compatibility rows | Tiny direct solve: `abs(defect) <= 4096 ε sum(abs(physical row terms))`; later iterative solve: normalized true row residual `<= 1e-10` plus QoI tolerance sensitivity check; zero scale requires zero defect; ill-conditioned enclosure remains uncertain, not passed |
| V02 incompressibility and wall condition | `L/U_ref` times maximum physical cell divergence; maximum no-slip error divided by `U_ref` | Divergence `<= 1e-8`; represented prescribed wall velocity error `<= 1e-10`; for reconstructed wall trace additionally `<= 1e-4` and converging under refinement |
| V03 mass balance | Steady I1 net mass/volume flux normalized by `U_ref A_ref`; transient inventory change plus outward flux minus sources normalized by `U_ref A_ref T` | `<= 1e-8`; per-component compatibility separately checked |
| V04 discrete fluid/body momentum or impulse | Steady force balance, or transient native momentum change against independently accumulated outer momentum flux, tractions, body-force and actuator impulse | Algebraic conservation `<= 4096 ε` times sum of term magnitudes with no unit-sized floor; iterative cases additionally carry their V01 solve-residual bound; deliberate splitting defects must be derived/exported and converge, not labelled exact conservation |
| V05 reconstructed surface versus control-volume force | Force component differences divided by `F_ref`; torque by `F_ref L` | `<= 0.005` on fine grid and decreasing under refinement; interval/quadrature uncertainty must be smaller than target |
| V06 uniform/affine traction and gauge closure | Exact geometric identities described below | `<= 1e-10` of declared pressure/stress geometric scale; constant-pressure force/torque leakage must satisfy its independent roundoff budget |
| V07 smooth manufactured continuum fields | Volume-weighted velocity L2/U_ref, pressure L2 after declared gauge alignment divided by its case pressure scale; reconstructed force/F_ref | Fine field errors `<= 0.01`, force/torque errors `<= 0.02`; observed orders at least declared formal order minus `0.25` on the asymptotic sequence |
| V08 I1 synthetic exterior result | Independent grid uncertainty in each nonzero force; domain sensitivity; near-zero symmetry component normalized by F_ref | Spatial `<= 0.02`, domain `<= 0.01`; symmetry force `<= 0.005`, symmetry torque `<= 0.005 F_ref L`; temporal criterion is N/A for steady I1 and `<= 0.005` for later transient suites |
| V09 I3/I4 frame histories | Transformed velocity L2, force/torque and impulse time histories at common physical locations/times | Velocity `<= 0.01 U_ref`; loads and integrated impulse `<= 0.02` of declared scale plus resolved independent numerical uncertainties; decreasing difference under separate mesh/time refinement |
| V10 I4 geometry/remap | Per-cell GCL volume defect and total mass/constant-state transport defect | Algebraic swept-volume defect `<= 4096 ε` times volume-term sum; constant state relative drift `<= 1e-10` over 1000 steps; no unreported cell creation/loss |

Here `ε` is binary64 machine epsilon; these algebraic budgets apply to supported
well-scaled fixtures, with independent compensated sums or outward enclosures.
They do not excuse an implementation claiming exact conservation when its
actual discretization has a systematic defect. Residual units depend on the
owning equation (flux volume/time, momentum or force); export that scale.
Production tolerances remain separately owned and cannot be inferred from
these fixture limits. Tighten the solver by factors 10 and 100: the resulting
QoI change must be `<= 0.001 F_ref` and `< 10%` of the finest adjacent
refinement difference. This separates iterative error from spatial/time error.

## I1: low-Reynolds analytic and synthetic exterior verification

The first executable demonstration is `submarine_stokes`, a fixed axis-aligned
synthetic body. Its creeping-flow model excludes convective inertia. A steady
Stokes solution remains a Stokes solution even when input units produce a
large nominal Reynolds number; the example must label this limitation and
reject any claim of finite-Re or turbulent prediction.

1. **Open-domain rest/uniform flow.** First run no body with uniform velocities
   on each positive/negative axis and oblique `(1, 0, 1)/sqrt(2)`. Match inflow
   and outflow fluxes, including oblique inflow on multiple faces. Require V01–V04.
   Add the body at rest with zero flow; demand zero velocity, hydrodynamic
   force and torque. Sealed-box forcing is a separate inherited control and
   cannot replace this open-domain case.
2. **No-slip and stress.** Run uniform exterior straight and oblique flow
   around a box with length/thickness ratio 4, using all axis permutations,
   reversed flow and origin offsets. Compare linear scaling in `U` and `μ`;
   in steady Stokes changing density must not change dimensional load. For
   flow along each reflection-symmetry axis, lateral force and centroid torque
   vanish within V08. Corners can lower convergence order; no nominal
   second-order claim follows from an interior stencil alone.
3. **Manufactured exterior boundary-value problem.** In dimensionless
   coordinates around a unit box, use
   `ψ = (x²-a²)² (y²-b²)² (z²-c²)²`,
   `u = (∂ψ/∂y, -∂ψ/∂x, 0)` and a nonconstant linear-plus-quadratic `p`.
   Normalize amplitudes before evaluating. This field is divergence-free
   and vanishes on every body face. Independently differentiate to prescribe
   forcing `f = ∇p - μ Δu` in steady Stokes; use exact outer velocity/traction
   consistent with each declared boundary type. For later transient suites,
   add time factor `exp(-t/T)` and analytic `ρ ∂u/∂t` source; steady I1 has
   no time test. Verify velocity,
   pressure, symmetric stress, force and torque, not just residual. Export
   actual advanced unknowns. This nonuniform manufactured forcing case is
   distinct from the uniform-flow demonstration.
4. **Small discrete independent solve.** For a tiny open geometry, assemble
   masses, incidence, boundary terms, symmetric strain and pressure constraint
   from published native geometric records; solve the finite saddle system
   independently with rational arithmetic where the fixture permits. Compare
   actual accepted native state and both load components. Independent assembly
   must not call production assembly/reconstruction helpers.
5. **Later curved analytic check.** When curved no-slip geometry is admitted,
   use a sphere in the Stokes limit. For radius `a`, stationary-body uniform
   stream `U`, total force magnitude is `6π μ a U`, pressure contribution
   `2π μ a U` and viscous contribution `4π μ a U`, with centroid torque zero.
   Prefer exact sphere-solution outer data for mathematical verification;
   uniform far-field truncation requires its own domain study. A faceted
   sphere is a different geometry until its surface error converges. Do not
   demand this unavailable curved case to certify an axis-aligned-only I1,
   and do not claim I1 establishes it. The classical result is attributed to
   Stokes (1851); see the source register for access status.

## Force, torque, gauge and stress controls

Fix `n_B` outward from the body into the fluid. Physical fluid force on body is
`F_B = ∫_S [-p I + μ(∇u + ∇uᵀ)] n_B dS`; moment about `r0` is
`M_B = ∫_S (r-r0) × traction dS`. Separate pressure and viscous records.
The fluid-domain normal on the body is `-n_B`; its traction impulse is the
opposite body impulse. Test both signs explicitly. For a translating body,
fluid and body interface powers sum to zero; a prescribed actuator owns the
remaining work. Steady exterior dissipation is supplied by the outer boundary
work, not an invented stationary-body energy source.

Before reporting solved-body drag, evaluate independently supplied fields on
a watertight box and a skew/transformed closed polyhedron where supported:

- `∫ n_B dS = 0`, `∫ (r-r0) × n_B dS = 0`, and
  `∫ r_i n_j dS = V_B δ_ij`. For `p = p0 + b·r`, expected pressure force is
  `-V_B b`; moment about centroid is zero, while moment about another origin
  is `(r_centroid-r0) × F`. Repeat with each axis, anisotropic cells and
  shifted body positions. This is a blocking discriminator for the known
  affine-pressure bias; pressure-solver convergence cannot close it.
- Add constant pressure offsets with `p0/P_ref` equal to `0, ±1, ±10³, ±10⁶`
  and require unchanged physical force/torque. Retain area-normal closure
  and a forward roundoff bound including `|p0|`; do not let cancellation
  inflate the allowed physical load. Repeat solve with alternative component
  pressure gauges. Pressure differences, velocities and closed-body loads
  must agree within arithmetic and solved-state tolerances. Open boundary
  references must shift consistently; a pressure Dirichlet condition can fix
  physical gauge and must not be independently changed.
- On an unclosed diagnostic patch, constant pressure produces its expected
  nonzero traction. Do not impose closed-surface gauge invariance on a patch.
  Omit one face and reverse one face deliberately; the validator must detect
  closure/sign faults rather than silently subtracting their load bias.
- Constitutive-only affine velocity tests include translation, rigid rotation
  (`ε(u)=0`), simple shear and extension; compute symmetric stress exactly.
  These supplied fields are not all compatible with a stationary no-slip
  body and are not called solved flows. Rigid rotation detects substitution
  of unsymmetrized `μ∇u` for Newtonian stress.
- Hydrostatic gravity control: `p_h=p0+ρ g·r`, `u=0`; buoyancy is
  `F_h=-ρ V_B g`, with centroid torque zero. Define whether reported load
  includes hydrostatic pressure. Do not include it twice by adding analytic
  buoyancy to a total-pressure surface integral.

Recompute control-volume force from fluid momentum change and all outer
advective momentum flux, pressure/viscous tractions and body forces. For an ALE
control surface use relative flux `(u-w)·n`, with mesh velocity `w`. Repeat
on two nested control volumes outside the body and compare with surface load.
Use exact discrete reaction/impulse records for V04, and reconstructed
continuum tractions for V05; agreement between two shared reconstruction paths
is only internal consistency. Include moment and boundary angular-momentum
flux; origin transformation must satisfy
`M(r1)=M(r0)-(r1-r0)×F`.

## Separate mesh, time, geometry and domain studies

Run the same physical case; transient suites use the same accepted horizon,
not the same step count. The first-slice native fixture uses `L=1 m`, body
thickness `D=L/4`, grid spacing `L/8` and half-widths `(2L,L,L)` with full
uniform velocity Dirichlet data on all outer planes. Its proposed meshes
`(32,16,16)`, `(64,32,32)`, `(128,64,64)` exceed the initial dense backend's
4096 combined active-DOF cap. N1/N2 must start with an explicitly defined tiny
fixture satisfying body alignment and padding; passing it proves discrete
external-flow/load controls. N3 mesh/domain load convergence stays pending
until an admitted sparse/iterative extension or a separately declared smaller
valid refinement family executes. Do not publish continuum load accuracy from
the tiny solve or silently shrink a failed qualification family.

For the later affordable smooth manufactured family use body thickness `D`,
local spacings `D/8, D/16, D/32, D/64` (ratio 2), retaining exact alignment
and fixed analytic geometry. This is a proposed qualification family, not
guaranteed executable with the tiny backend. Refine all relevant directions,
wall sampling and force
quadrature. Record actual counts and local wall/wake spacings; a cube-root
total-cell proxy alone cannot prove wall refinement on an anisotropic mesh.
The proposed smooth I1 scheme must state its formal spatial order before
observing results; transient schemes must also declare time order. Absent
orders leave V07 unavailable. Nonsmooth synthetic
corner flows require positive/stable QoI convergence and uncertainty, not an
unearned smooth-field order requirement.

For physical-force cases use the same mesh family with sufficiently fine
surface/wake resolution, refining geometry separately if it is faceted.
For transient suites hold temporal error small during mesh studies; I1 has
no temporal error contribution. On the fine mesh of later suites run
`dt/T_ref = 1/20, 1/40, 1/80, 1/160`, with
`T_ref = min(L/U_ref, L²/ν, ramp_duration)` among applicable finite scales.
Reduce further to satisfy stability and wall sweep constraints; retain a
constant ratio and actual accepted `dt` history. In I4 require initially
`max |V_B| dt/h <= 0.25` in addition to the fluid Courant constraint.
Time estimates must use the same physical sampling and account for interpolant
order. For later stationary flows computed by time evolution or nonlinear iteration, certify that longer evolution or additional nonlinear
iterations changes force `< 0.1% F_ref`; over the final five characteristic
times require drift `< 0.1% F_ref` and matching momentum balance. If the flow
is genuinely unsteady, retain time histories and estimate sampling uncertainty
instead of forcing a steady criterion.

For an equal-ratio monotone three-grid series, index fine/medium/coarse as
`Q1,Q2,Q3`, compute `p = log((Q3-Q2)/(Q2-Q1))/log(r)`,
`Q_ext = Q1 + (Q1-Q2)/(r^p-1)`, and absolute fine uncertainty
`GCI_abs = 1.25 |Q1-Q2|/(r^p-1)`. State the Celik/Roache GCI procedure,
not a hybrid presented as the current ITTC algorithm. Use the fourth grid to
assess stability of observed order (`<= 0.25` change between adjacent triples)
and error model. The usual same-triplet GCI-ratio calculation partly repeats
the fitted order and cannot independently establish asymptotic behavior.
Report every grid, sign and raw difference. For unequal ratios use the
published generalized equation. For noisy/oscillatory/nonpositive-order or
roundoff-limited series refuse the simple extrapolation; add grids or perform
an explicitly selected Eça–Hoekstra fit with at least four levels. Do not
discard a failed grid, replace observed order with desired order, or interpret
GCI as a universal physical or statistical confidence bound.

The tiny I1 finite domain is the prescribed all-plane Dirichlet box in the
first-slice handoff; its uniform data carry inlet/outlet flow through the
corresponding planes without qualifying other outlet stress operators.
Start later external finite-Re domains with inlet 3L upstream of the bow, outlet
8L downstream of the stern, lateral/top/bottom clearance 3L from the hull.
Compare clearance/outlet pairs `(5L,12L)` and `(8L,20L)` at fixed local
wall/wake resolution, same reference pressure and same physical boundaries.
For oblique/ascent flow, apply distances along the actual stream and check
all inflow/outflow faces. Require V08 domain sensitivity on two successive
enlargements; if it fails, enlarge again and retain all cases. Creeping Stokes
disturbances decay slowly, so these finite-Re initial extents are not assumed
sufficient for I1: use exact manufactured outer data for code verification,
then enlarge uniform-flow domains until the stated target is met. A dense CPU
allocation cap may make the needed study infeasible; that leaves the claim
unavailable rather than changing the domain criterion.

For steady I1, increased direct precision/conditioning controls and the true
residual replace a temporal/longer-evolution test. N1/N2 acceptance does not
close V07/V08 continuum convergence until N3 executes.

The study log separates `U_space`, `U_time`, `U_iter`, `U_domain`,
`U_geometry` and, when relevant, `U_sampling`. Combine systematic numerical
contributions conservatively unless justified statistical independence is
established. Refinement alone detects neither wrong physics nor incorrect
geometry. Geometry/units/boundary provenance and independent exact identities
remain blocking checks.

## I2/I3: finite laminar-Re and accelerating body-frame qualification

I2 includes convection and must reproduce a divergence-free manufactured
Navier–Stokes field with independently evaluated `ρ(u·∇)u`. Pin laminar cases
at `Re_L=1,10,100` only within the geometry's supported/stable regime;
these are proposed probes, not declarations that every case is steady.
Compare reversal, oblique incidence, wake profiles, force and torque. Any
published laminar benchmark must reproduce its complete geometry, inlet,
pressure gauge, outlet stress operator and force normalization; a reference
drag number from a different boundary-value problem is inadmissible.

For I3 prescribe `X(t)` and `V(t)=dX/dt`; fixed attitude is the scope. In
translated coordinates `r=x-X(t)`, relative velocity is `v=u-V(t)` and

```text
rho [d_t v + (v dot grad) v] = -grad p + div(tau) + rho g - rho a(t)
a(t) = d_t V(t)
v_wall = 0;  v_far = u_far - V(t)
```

Physical pressure remains `p` when acceleration is explicitly retained. If
the solver instead absorbs the uniform fictitious source into
`pi = p + rho a(t) dot r`, recover `p = pi - rho a(t) dot r` before physical
loads. The pressure-force correction is `+rho V_B a`, and moment correction
about a displaced origin is
`(r_centroid-r0) × (rho V_B a)`. This is a field correction with a physical
load consequence, not a pressure gauge shift. A control omitting it must fail.
Apply the corresponding uniform source and pressure convention at outer
boundaries. Gravity and analytic buoyancy have independent accounting.

Prescribe smooth forward and ascent ramps using
`s=t/T_ramp`, `V=V_final(10s³-15s⁴+6s⁵)` for `0<=s<=1`, then constant speed;
test vectors `(U,0,0)`, `(0,0,U)` and `(U,0,U)/sqrt(2)`, plus reversals and a
return-to-rest ramp. Record actual derivatives and reference axes; do not
imply vertical ascent is a free-surface simulation. Require:

- Constant-velocity Galilean control: transformed initial/far/wall data must
  describe the same problem, including matching relative transport velocity.
- Accelerating-frame manufactured control: a no-body fluid at inertial rest
  has `v=-V(t)`, constant physical pressure and source `-rho a(t)`; an
  absorbed-source implementation has affine `pi` and must recover zero
  physical pressure gradient. Check each axis and oblique acceleration.
- I3 time refinement and force/impulse histories. Once I4 exists, compare the
  same finite-domain physical solution in world/body frames after transforming
  the entire outer boundary and initial state. Fixed world boundaries transform
  into moving body-frame boundaries; two convenient differently positioned
  boxes are not mathematically identical. Before that I3 is independently
  verified by analytic/MMS controls; world/body solver agreement is pending.

## I4: moving embedded geometry and geometric conservation

The Thomas–Lombard geometric conservation principle is a necessary moving-grid
consistency condition, not sufficient evidence of correct force, no-slip,
momentum or topology handling. For each fluid control volume require

```text
V_fluid^(n+1) - V_fluid^n = integral_step integral_boundary w dot n_fluid dS dt
```

with the sign tied to the exported fluid normal. On a fixed Cartesian cut cell
only its moving solid boundary contributes mesh motion; fluxes/remapping must
use compatible swept volumes. Record transfers for cells becoming covered or
uncovered and component merges/splits. A rebuilt geometry with new volumes and
reused old state is not a conservative remap.

Planned GCL cases include a box translating ±0.1 and ±0.9 cell widths across
each axis, oblique translation, exact crossings of a grid plane, repeated
cover/uncover events and forward/back cycles. Include very small cuts permitted
by the owning geometry contract and a case beyond its cap: it must reject with
the promised diagnostic and preserve the last accepted state.

Separate valid controls:

1. Uniform free stream on a moving mesh with no body. Exact velocity/pressure
   and a constant passive field remain unchanged under arbitrary allowed grid
   motion. Then a body co-translates with a matching uniform fluid and outer
   data, so no-slip is satisfied and relative flow is zero. Uniform fluid past
   a differently moving no-slip body is not a free-stream preservation test.
2. Constant scalar on a changing fluid domain: stored mass equals scalar times
   actual fluid volume, with outer transport flux and conservative transfers.
   Total fluid volume is constant when a wholly contained rigid body translates
   inside a fixed outer box. An individual cut-cell volume need not be constant.
3. Moving body into inertial-rest fluid: independent swept-volume balance,
   physical wall velocity, interface equal/opposite impulse and work, V01–V05,
   mesh/time refinement, and the I3 counterpart. A stationary-wall replay and
   a forward/back geometry cycle distinguish remap drift from physical wakes.

Run 1000-step reversible geometry/constant-state controls and a shorter
resolved transient force case at four mesh/time levels. Cancellation at
geometry build, remap, solve, reconstruction and precommit must preserve
geometry, velocities, pressure/load witnesses, time and generation together.
Allocation/iteration exhaustion and invalid/nonfinite motion must do the same.
Repeat deterministically; for parallel CPU require the declared reduction
contract and unchanged physics/acceptance limits, not an assumed bitwise result.

## I5: experimental validation and rights boundary

Official DTRC/NSWCCD reports establish a candidate reference family: Groves,
Huang and Chang (1989), **Geometric Characteristics of DARPA SUBOFF Models**,
ADA210642; and Liu and Huang (1998), **Summary of DARPA SUBOFF Experimental
Program Data**, ADA359226. Official NTIS records confirm the geometry authors,
year, model identities and analytic definition scope, and the experimental
summary identity and wind-tunnel/towing-basin scope. These are report records,
not evidence that a currently downloadable raw database has a reuse license.
[Geometry record](https://ntrl.ntis.gov/NTRL/dashboard/searchResults/titleDetail/ADA210642.xhtml),
[experimental summary record](https://ntrl.ntis.gov/NTRL/dashboard/searchResults/titleDetail/ADA359226.xhtml).

Do not mix bare-hull and appended configurations or air wind-tunnel and water
tow-tank conditions. Select the report's exact configuration identifier and
test number, length/diameter, appendages, pressure taps, velocity stations,
Reynolds number, density/viscosity, transition trips/roughness, support/sting,
facility wall/blockage corrections, angle and uncertainty definition. Rebuild
geometry independently from verified mathematical definitions; external CAD,
meshes and report code listings have separate provenance and rights. Publish
geometry residual checks against report dimensions and station locations.

SUBOFF high-Re turbulent resistance/wake data are a later model-validation
target. An I1 Stokes or I2 under-resolved laminar run against those data has a
known model mismatch; matching one drag value cannot confer turbulence
capability. I5 admission requires a separately specified/verified turbulence
closure or adequately resolved turbulence method, wall treatment/y-plus
evidence, transition assumptions and CPU resource evidence. Laminar numerical
qualification may finish while this requirement remains blocked. No full
turbulent submarine claim is permitted in this plan.

Before importing any experimental values, the acceptance owner must approve a
record identifying source/issuer, exact tables or files, allowed use and
redistribution/derivative rights, third-party exceptions, attribution and
license compatibility with this Apache-2.0 repository. Public accessibility,
unclassified status and public-release distribution are separate facts from
ownership/license of every underlying raw dataset. Government report facts
and an independently derived formula do not authorize copying unlicensed
dataset archives, external code or meshes. Do not copy GPL code or artifacts
whose rights are absent. If file reuse is unresolved, link the report and
retain the experiment gate as blocked; do not silently vendor a mirror.
No experimental data, report PDF, external solver code or CAD was imported
during this planning work.

For approved data use the following proposed accuracy contract, fixed before
candidate comparison:

- Total resistance: discrepancy `|S-D|/|D| <= 5%`; independently established
  spatial/time/iteration/domain/sampling numerical budget `<= 2%` of D.
- Selected hull-pressure distribution: weighted RMS `Cp` discrepancy
  `<= 0.05` and maximum `<= 0.10` on predeclared supported taps.
- Wake velocity: normalized weighted RMS error `<= 0.05 U_ref`, including
  predeclared near-wake stations; require wake deficit and transverse velocity
  observables, not just total drag.
- When skin friction is an intended claim: normalized RMS discrepancy
  `<= 10%` of declared nonzero reference friction scale, separately from pressure
  drag. Near-zero values use an absolute physical scale.

These engineering limits are proposed targets, not published SUBOFF accuracy
or uncertainty values. Establish experiment-specific measurement and input
uncertainty before making a quantitative validation claim. Missing uncertainty
leaves the comparison descriptive and acceptance blocked. For each QoI retain
signed comparison error `E=S-D` and an explicitly justified validation
uncertainty `U_val` covering experiment, propagated inputs and numerical error.
Combine stochastic standard uncertainties with covariance and a stated
coverage factor; retain conservative systematic numerical estimates separately
if they cannot legitimately enter that statistical combination. Report the
inferred discrepancy range, not just overlapping bands. For acceptance require
the conservative bound `|E|+U_val` within the declared engineering target;
`|E|<=U_val` alone cannot pass by enlarging uncertainty. Do not infer agreement
outside tested Re/configuration/angle or claim ASME certification.

## Primary-source register and access limitations

All links were checked or searched on 2026-10-14. The following attribution is
methodological; this report does not claim certification or compliance with
unread licensed standards. Recheck current editions before implementation.

| Source | Exact link and verified status | Use and limitation |
| --- | --- | --- |
| ITTC 7.5-03-01-01, *Uncertainty Analysis in CFD Verification and Validation Methodology and Procedures*, 2024 revision 05 | [Official PDF](https://www.ittc.info/media/11950/75-03-01-01.pdf); [official catalogue](https://ittc.info/downloads/quality-systems-manual/recommended-procedures-and-guidelines/). Both identify 2024 rev05, latest listed in the checked catalogue | Verification/validation distinction; current procedure discusses Stern and Eça–Hoekstra approaches. This plan's explicit GCI choice must not be mislabelled as that complete procedure. No external text/code imported |
| ASME V&V 20, *Standard for Verification and Validation in Computational Fluid Dynamics and Heat Transfer*; Eça, Dowding and Roache, interpretation/scope paper (ASME journal 2022) | [ASME author paper](https://asmedigitalcollection.asme.org/verification/article/7/2/021005/1136718/On-the-Interpretation-and-Scope-of-the-V-amp-V-20); [author manuscript at OSTI](https://www.osti.gov/servlets/purl/1774746); [Dowding 2016 overview](https://www.osti.gov/servlets/purl/1368927). Indexed primary records located; OSTI opens returned 502 and legacy standard product URL returned 404 | Attribution to variable/setpoint-specific accuracy and uncertainty methodology; licensed standard full text and current reaffirmation were not obtained. No standards-compliance/certification assertion |
| Celik, Ghia, Roache, Freitas, Coleman and Raad (2008), *Procedure for Estimation and Reporting of Uncertainty Due to Discretization in CFD Applications*, JFE 130, 078001 | [ASME publisher record](https://asmedigitalcollection.asme.org/fluidsengineering/article/130/7/078001/444689/Procedure-for-Estimation-and-Reporting-of), DOI [10.1115/1.2960953](https://doi.org/10.1115/1.2960953). Indexed publisher record found; full opening returned 403 | GCI attribution; no publisher code or article text copied |
| NASA NPARC Alliance, *Examining Spatial (Grid) Convergence*, updated 2021-02-10 | [Official NASA tutorial](https://www.grc.nasa.gov/www/wind/valid/tutorial/spatconv.html), full text accessible | Supports Richardson/GCI formulas, three-grid recommendation, safety factor 1.25 and asymptotic cautions. The site's downloadable Fortran was not imported |
| Thomas and Lombard (1979), *Geometric Conservation Law and Its Application to Flow Computations on Moving Grids*, AIAA Journal 17, 1030–1037 | [AIAA DOI](https://arc.aiaa.org/doi/10.2514/3.61273). Indexed publisher attribution verified; full text returned 403 | Original GCL attribution. The displayed cell balance/tests are this plan's proposed design, not a reproduced implementation or claim to have read the inaccessible article |
| Stokes (1851), *On the Effect of the Internal Friction of Fluids on the Motion of Pendulums* | [Original volume scan](https://babel.hathitrust.org/cgi/pt?id=mdp.39015012112531&seq=251); [Royal Society published research on Stokes' law](https://royalsocietypublishing.org/rsta/article/378/2179/20200214/111689/Stokes-law-viscometry-and-the-Stokes-falling). Original formula bibliographic pointer located; original scan not inspected | Classical creeping sphere check only when curved geometry is supported; not an experimental submarine benchmark |
| DTRC Groves/Huang/Chang geometry report ADA210642 (1989) | [Official NTIS record](https://ntrl.ntis.gov/NTRL/dashboard/searchResults/titleDetail/ADA210642.xhtml), accessible; [DTIC record](https://apps.dtic.mil/sti/citations/ADA210642), unavailable through current browser | Candidate geometry reference; authoritative metadata verified, complete equations/rights of bundled material not yet reviewed |
| NSWCCD Liu/Huang experimental summary ADA359226 (1998) | [Official NTIS record](https://ntrl.ntis.gov/NTRL/dashboard/searchResults/titleDetail/ADA359226.xhtml), accessible; [DTIC record](https://apps.dtic.mil/sti/citations/ADA359226), unavailable through current browser; DOI [10.21236/ADA359226](https://doi.org/10.21236/ADA359226) | Candidate experimental provenance; raw-file licenses, full uncertainty tables and matched test inputs remain unresolved. Mirrors are discovery aids, not license authority |

## Acceptance disposition

This document makes the future tests concrete and reviewable. It does not
satisfy I1–I5. The parent may accept the planning deliverable after scope,
source attribution, thresholds and producer/consumer contract review. Solver
milestones require the actual named native evidence, independent checks and
review of the identified implementation and material acceptance contract; adding unchanged ledger/review evidence does not invalidate that review. I1 can pass as creeping synthetic external flow while
I2–I5 remain pending/blocked, provided every user-facing claim preserves that
limit. All source-code implementations, experimental ingestion and full
turbulent capability remain unaccepted by this report.
