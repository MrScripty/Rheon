# Independent submarine I1 validation

This Python stdlib validator is independent of Rheon's native assembly and load
helpers. It implements the bounded steady, aligned, padded cuboid creeping-Stokes
contract in the [canonical plan](../../docs/plans/submarine-incompressible-cpu/plan.md).
Parent admitted operator profile **`rheon-i1-sector-half-cell-v1`** after the
independent corner comparison. This is not a report that native I1 has run or
passed. The revised producer schema/source handoff and actual independent
execution remain pending. N1/N2 remain pending actual captures and complete
controls. N3 continuum mesh/domain convergence and N7 experiments remain pending.

Only the six assigned validator/report files change. Native owner
`01a139a7-da3b-71b3-b155-ddca2f780005` owns source/export implementation. Parent
pins source and validator revisions and admits the export interface before capture.
There is no recording, Pages, physics-library, GPU, remote, transient, arbitrary
hull, sparse-backend or experiment expansion here.

## Runnable boundaries

Harness tests work now, including under optimized Python:

```sh
python3 -m unittest discover -s validation/submarine -p 'test_*.py'
python3 -O -m unittest discover -s validation/submarine -p 'test_*.py'
```

The source-controlled `fixtures.json` freezes 18 native cases, 34 constitutive
controls and the quadrature/wall/pivot/MMS semantic profile in `discretization`. They are prospective inputs, never stored native output. Native capture
requires an admitted native commit containing `examples/submarine_stokes.rs`,
explicit agreement on the exact fixture digest, and this producer CLI:

```sh
source /workspace/rheon-setup/env.sh
python3 validation/submarine/run.py --suite stokes --source FULL_40_DIGIT_COMMIT \
  --native-contract-sha256 AGREED_FIXTURES_SHA256 --output /tmp/FRESH_SUBMARINE_RUN
```

The runner archives the exact tracked source, builds with Rust 1.92.0,
`cargo build --verbose --locked --release --no-default-features --example submarine_stokes`,
then invokes the actual built executable for every selected fixture:

```text
EXECUTABLE --fixtures EXTERNAL_FIXTURES_JSON --case FIXTURE_ID --output FRESH_CASE_DIR
```

Each native invocation must write `FRESH_CASE_DIR/record.json`. `--case` can be
repeated to request a partial suite; a partial suite never closes N1/N2. The
runner refuses missing native capability, unsupported suites, absent contract
admission, reused output and output inside the checkout. Missing native capability
exits 2 with `unavailable/native-capability`, not a passing skip. It never generates
an analytic field in place of native output.

Imported data can be numerically checked without claiming origin:

```sh
python3 validation/submarine/run.py --suite stokes --source FULL_40_DIGIT_COMMIT \
  --input /tmp/ACTUAL_RUN/cases/tiny-straight/record.json \
  --origin /tmp/ACTUAL_RUN/origin.json --origin-sha256 PARENT_PINNED_SHA256 \
  --output /tmp/FRESH_RECHECK
```

Omitting `--origin` leaves provenance explicitly unavailable. Source does not
authenticate an imported record by itself. A parent-supplied origin hash is an
external trust input; a self-hash, copied correct field, or matching executable
hash alone cannot prove that the claimed execution occurred. Recheck checks the
actual source tree/archive, executable, fixture bytes, current validator files,
exact case argument and captured command/output hashes. Its assurance is
externally pinned manifest integrity, not an independently signed build attestation.

## Derivation and frozen finite operator

Use right-handed SI Cartesian coordinates and x-fast arrays. Pressure is Pa;
velocity is m/s; viscosity is Pa s; volume force is N/m³. A face is active iff its
two adjacent cells are fluid, prescribed iff exactly one is fluid, and masked iff
neither is fluid. Masks are 1/2/0 respectively. Prescribed outer **full-vector**
velocity comes from the fixture; stationary body velocity is zero. Solid-array
slots retain canonical positive-zero bits, not an invented fluid value.

At fluid cell c of volume V, normal strain is
`E_a=(u_a(high)-u_a(low))/h_a`. For each unordered pair `a<b`, shear
`S_ab=du_a/dx_b+du_b/dx_a` lives on its MAC edge. Each incident fluid sector
has quadrature volume `V/4`. Four-fluid interior sectors share the centered row;
stationary two-fluid flat-wall sectors share the signed half-cell row and may be
aggregated. A three-fluid body corner retains **three distinct sector rows**.
No extra free-slip omission or duplicate ordered shear pair is admitted.

For a three-fluid corner and sector q, use its incident face samples and the
full-vector prescribed edge trace:
`S_ab,q=2 s_b (u_a,q-g_a(edge))/h_b + 2 s_a (u_b,q-g_b(edge))/h_a`,
where each s is +1 in the high quadrant and -1 in the low quadrant.
Both prescribed normal-face samples and edge-trace columns remain in these
rows even when their stationary value is zero. Each row contributes
`(mu/2)(V/4) S_ab,q²` to Phi. Collapsing these rows into one centered clipped
row changes the functional and is refused by this profile.

Two available derivative samples use their ordinary centered difference. One
missing sample uses the full vector boundary value at the intervening edge:
`+2*(u_inside-g)/h` on a lower Cartesian boundary and
`-2*(u_inside-g)/h` on an upper boundary. Outside the three-fluid corner rule,
available prescribed normal-face samples remain samples; only a missing sample
gets reflected.
At outer MMS edges g varies analytically. Zero tangential wall derivatives apply
only to the stationary body's constant no-slip trace, not the nonuniform outer
boundary. Rigid rotation must produce zero symmetric strain at upper boundaries.

Define the half-dissipation functional
`Phi = mu sum_cells V sum_a E_a² + (mu/2) sum_edge_sectors (V/4) S_ab,q²`.
The viscous matrix is its Hessian. Physical dissipation is **2 Phi**, not Phi.
For volume-integrated divergence `B u`, with prescribed velocities separated as
an affine constant d, the unknown equations are:

```text
K x + c - B_active^T p = F
B_active x + d = 0
```

Here c includes every prescribed normal face and tangential ghost-Dirichlet
contribution. `F_face=dual_volume*point_force`. A face's dual volume is V times
its adjacent fluid-cell count divided by two. Distinct ghost and corner boundary
variables remain in the functional, so algebraic reaction/work includes them too.
Each reaction carries its actual component, body/outer owner and SI location;
corner torque uses the edge point, not an adjacent face point.
Ignoring those contributions is not a valid momentum/work control.

Pressure components follow fluid-cell adjacency; the padded cuboid/no-body
profile is connected. The first x-fast fluid cell fixes each component's pressure.
For uniform cases it is the declared fixture datum; for MMS it is the analytic
pressure at that cell. Eliminate that pressure unknown and its redundant divergence
row, retain its known momentum contribution, and independently check **all**
divergence rows afterward. Check prescribed flux compatibility in every component
before solve/gauge elimination. Do not pin all components to one unavailable row.

The fixture `tiny-straight` has 474 active velocities and 195 nongauge pressures:
**669 unknowns**. Count before allocating operator rows; the admitted mixed cap
is 4096 and a smaller caller cap refuses earlier. File size is capped at 16 MiB.
The oracle uses exact rational coefficients/decoded f64 values for sparse physical
row residuals, with `4096 epsilon sum(abs(expanded physical row terms))` and no
unit-sized floor. Pressure/body-force/affine terms contribute to that bound.

A separate `exact-no-body` 2³ case has **19 unknowns** and is rational-solved.
The exact dense oracle is capped at 128 unknowns. Columns are nondimensionalized
with U_ref and `mu U_ref/L_ref`; rows are scaled by their maximum coefficient.
Partial pivoting refuses a scaled pivot below 1e-12. This is a frozen refusal,
not a general conditioning proof. The 669-unknown body case is independently
assembled and row-checked, **not rational-solved**. The native solver's backend,
minimum scaled pivot, actual DOFs and retained-payload/caller cap are explicit.
Its resource report does not establish an RSS or hardware-speed bound.

## Wall trace and load meaning

Each flat body patch is the exposed fluid-cell wall face. Record its exact
center, outward **body** normal, area, adjacent fluid-cell ID and zero wall
velocity. The fixture guarantees two fluid layers. Pressure extrapolation is
`p_wall=(3 p_near-p_next)/2`, from distances h/2 and 3h/2. This is affine-exact;
using the adjacent-cell pressure directly fails the affine-load discriminator.

For the normal velocity component, derivative into fluid is
`(-3 g+4 u(h)-u(2h))/(2h)`. For a tangential component, first average its two
MAC samples to the patch center at each depth, then use
`(9 u(h/2)-u(3h/2)-8 g)/(3h)`. Both are quadratic-exact one-sided formulas.
Multiply by the body normal sign to recover the Cartesian derivative.
Stationary body g=0; tangential derivatives along that flat wall are zero.
Edge/corner patch averaging is part of the fixed wall reconstruction contract;
no continuum corner order is assumed.

Compute pressure and Newtonian shear separately:
`traction = -p n_body + mu (grad u + grad u^T) n_body`.
Integrate pressure/shear forces and moments about the fixture reference point.
Keep physical reconstructed traction distinct from algebraic boundary reactions.
Check algebraic force, moment, external boundary work and physical dissipation;
do not infer physical trace accuracy merely from algebraic closure. Net physical
loads use independently accumulated patch/operation magnitudes for roundoff,
including large pressure offsets and cancellation. Zero underlying terms still
require zero defect. Fine-grid physical/algebraic load agreement and convergence
remain N3 evidence, unavailable from one coarse body solve.

The 34 frozen constitutive controls include affine pressure with seven offsets,
hydrostatic pressure, translation, rigid rotation, shear, extension, Couette and
Poiseuille. They are supplied analytic **constitutive-only** controls, not claimed
stationary-body solutions. Six closed cuboid midpoint patches use the fixed
1 x 1/4 x 1/4 body; channel controls use two declared unit patches. Every patch
must match its frozen geometry and carry local pressure, gradient, separate
force/moment components, analytic wall velocity and mechanical work. Empty
surfaces, self-selected gradients, omitted patches and wrong signs are refused.
Patch shear is checked before net loads: a constant symmetric shear can have zero
closed-body net force while omitted shear is still wrong. Couette checks wall-work
sign; Poiseuille checks both wall shear signs. An optional native
`constitutive_controls` collection must contain all 34 unique frozen IDs;
absence is disclosed and keeps N1/N2 pending.

## MMS sampling is not a discrete exact solution

The fixed body-centered polynomial is
`psi=A (x²-a²)² (y²-b²)² (z²-c²)²`,
`u=(dpsi/dy,-dpsi/dx,0)`, with declared linear/quadratic pressure.
Independent polynomial differentiation gives `f=grad(p)-mu Laplacian(u)`.
It vanishes on body faces and is continuously divergence-free. Export actual
native **point values** of f on MAC nodes; the oracle independently checks them
and multiplies by clipped face dual volumes. The exact integrated-volume forcing
and integrated-face-flux versions are different methods. Neither can be relabeled
as this `point-value-times-dual-volume` profile. Sampling a continuum divergence-
free field does not automatically make its finite MAC divergence zero, so those
analytic samples never replace the actual solved unknowns. Single coarse MMS L2
errors are reported only; no fine-grid error/order or N3 pass is invented.

## Record JSON producer handoff

All f64 values are strings of exactly 16 lowercase hex digits representing the
numeric IEEE-754 binary64 `to_bits` integer. They are not little-endian byte
strings. Arrays have exact shapes; NaN, infinity, duplicate JSON keys, malformed
values and oversized files fail. Integers remain integers, not booleans.

Required top-level members:

| Member | Meaning |
| --- | --- |
| `schema` | `rheon-submarine-i1-mac-v2` |
| `operator_profile` | `rheon-i1-sector-half-cell-v1` |
| `fixture_id`, `fixtures_sha256`, `controls` | Exact prospective case, actual fixture bytes and complete case object |
| `discretization` | Exact approved fixture semantic profile; its bytes participate in the parent/native agreement digest |
| `accepted`, `solve_id` | True and 1 for one fresh exporter solve; no `time`, `dt` or timestep |
| `equations`, `frame` | `steady-incompressible-newtonian-stokes`, `fixed-right-handed-SI` |
| `units` | velocity m/s, pressure Pa, gradient 1/s, force N, moment N m, volume_forcing N/m^3 |
| `layout` | x-fast; MAC velocity offsets `(0,1/2,1/2)`, `(1/2,0,1/2)`, `(1/2,1/2,0)`; pressure `(1/2,1/2,1/2)`, all offsets encoded as f64 bits |
| `geometry`, `geometry_sha256` | Counts, bit origins/spacings/body bounds, cell/face masks, face dual volumes, pressure components and component gauges; SHA256 of sorted compact JSON |
| `fields` | Three full MAC velocity arrays, three full point-volume-forcing arrays, full cell pressure array |
| `forcing_semantics` | `zero` or `point-value-times-dual-volume`, exactly the fixture mode |
| `outer_traces` | Complete unique normal-face and edge-ghost boundary samples, with axis, side, bit point and full bit velocity; per-plane associations retained |
| `body_traces` | Complete unique prescribed body face/edge points and actual full-vector no-slip values; corner/ghost traces are retained explicitly |
| `wall_patches` | Deterministic x-fast fluid-cell then axis then sign order; cell, axis, side, point, normal, area, velocity, reconstructed pressure and 3x3 gradient |
| `reaction_sites` | Exact map from every prescribed face/ghost/corner key to integer component, `body`/`outer` owner and three SI-point f64 bits |
| `algebraic_reactions` | One force-unit scalar per prescribed face, ghost and corner variable |
| `algebraic_loads` | body_force, outer_force, body_moment, outer_moment; body sign is opposite the functional boundary derivative |
| `physical_loads` | pressure_force, shear_force, pressure_moment, shear_moment |
| `dissipation` | Actual 2 Phi in W |
| `solve_report` | backend `dense-scaled-partial-pivot`, actual_dofs, min_scaled_pivot bits, retained_payload_bytes and payload_limit_bytes integers |
| `constitutive_controls` | Optional complete frozen control collection; absence is unavailable, not a passing skip |

Array shapes are `(Nx+1,Ny,Nz)`, `(Nx,Ny+1,Nz)`, `(Nx,Ny,Nz+1)` and
`(Nx,Ny,Nz)`. `face:A:I:J:K` identifies a MAC boundary variable.
`ghost:A:B:I:J:K:COMPONENT` identifies a reflected full-vector edge value with
`A<B`. `corner:A:B:I:J:K:COMPONENT` identifies the full-vector body edge
trace column for three-fluid corners. Its point is
`origin + (I,J,K)*spacing`, with a half-cell offset only on the third axis.
The tiny body fixture has 1464 strain rows, 780 prescribed scalar variables
and 54 unique full-vector body trace points. Reaction sites cover all 780
variables. There is no inference from latest fields or omission of prescribed trace.
`oracle.Operator` exposes the independent geometry associations/row coefficients
for diagnostics; producer code must not import it or copy its assembly as an
"independent" implementation.

## Provenance and partial-run handling

Outputs go to a fresh directory outside the checkout. `origin.json` starts active
and records exact source commit/tree/archive, verbose compiler command streams/flags, effective
build environment, Cargo-home configuration hashes, executable bytes, immutable
fixture copy, validator identities, exact case argv and actual output hashes.
The origin envelope remains `rheon-submarine-i1-origin-v1` and now explicitly
binds `record_schema` and `operator_profile`; an older envelope lacking these
members cannot be relabeled as a capture of this revised contract.
Ambient compiler flags/wrappers and credentials are removed; Rust 1.92.0 and one
build job are selected explicitly. Native output is parsed only after successful
execution; accepted data are never generated from expectations.

Origin/assessment JSON uses exclusive temporary writes, file fsync and atomic
replace. Failure/cancellation marks the origin terminal failed/cancelled and keeps
actual artifacts. Abrupt termination may leave an active origin or `.partial`
file: neither establishes completion; reusing the directory is refused. Successful
case prefixes remain evidence of those cases only. No power-loss durability,
whole-suite success from a prefix, deterministic replay, or restartable checkpoint
is claimed. Unbounded external Cargo/native log size and build duration are not
process-wide resource guarantees; record/unknown/exact-matrix caps are explicit.

A passing parser, test-only synthetic record, row check or constitutive control
is not a native execution or complete N1/N2 acceptance. Parent integration must
freeze the real source/binary/fixture/command/output subjects, obtain independent
review and separately disposition each named numerical gate.

## Preserved profile decision and qualification boundary

The earlier clipped-edge proposal is commit
`52322ca18dd4e9a32c83e834b5f05a558163216a`, fixtures SHA256
`6b6f0d55afaf3a92b6e88c116a0b858107e3e24e21a426f080d19345a78dab9b`.
The mismatch decision remains in commit
`597c3ca02a375cf47e4be083c658857fa2e66ff6` and the
[I1 validation report](../../docs/plans/submarine-incompressible-cpu/reports/i1-validation.md).
Their prior test receipts remain bound to those subjects.

Pinned `src/aligned_strain.rs` at `9efef18798d48268d9c5f28ea49e968cde55a39f`
supports the stationary corner coefficients, centered interior and flat-wall
equivalence. Its outer free-slip implementation does not qualify the full I1
outer Dirichlet or retained reaction columns. Exact affine, rotation, corner
and virtual-work harness controls now test the independent oracle's extension;
native implementation, exported sites, reactions and work require separate
actual execution and review. No acceptance thresholds were changed.
