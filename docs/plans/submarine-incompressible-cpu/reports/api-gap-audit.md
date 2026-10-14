# Incompressible submarine API and capability audit

Status: planning-only source audit, 2026-10-14. Source: main
`06eb5daee9e77e2b0843a5ee0a4105f6950e9b1b`. This report does not authorize or
claim implementation, numerical acceptance, public API promotion or vehicle
accuracy. Protected PR76 head `4bcdb7f49225cd645a0934953ba7d339ff5e168d`
is outside the write set. No PR, solver, tag, merge or dependency was changed.

The missing first capability is bounded **external creeping flow around a fixed
synthetic body**, with prescribed straight/oblique flow, Newtonian no-slip and
pressure/shear loads. Existing elementary kernels are prerequisites and reusable
evidence; they cannot supply that capability through an adapter alone.

The [canonical plan](../plan.md#milestones-and-exact-prospective-write-sets) owns
I1–I7 sequencing, exact prospective writes and implementation admission. This
report owns the inspected API facts and technical reuse assessment.

## Actual capability inventory

Source references below identify symbols and line numbers at the audited revision.

| Required capability | Real implementation and boundary | Gap for submarine use |
| --- | --- | --- |
| Constant-density pressure projection and transport | `Simulation`, `SimulationConfig`, `StateView`, `src/simulation.rs:10,212,224,318`; `PressureOperator`, `src/operator.rs:43`; semi-Lagrangian MAC transport, `src/advection.rs:25` | Fixed filled rectangular carrier. Default normal boundary speeds are zero. It owns neither a hull fluid domain nor hull stresses. |
| External box speeds | `PrescribedBoxFlux`, `src/box_flux.rs:14-23`; `Simulation::step_with_box_flux`, `src/simulation.rs:383-409` | Uniform **normal** speeds per complete side, balanced prescribed flux and pressure correction; no pressure outlet, tangential farfield prescription, solids or moving geometry. Appearance extension is clamped interpolation, not a physical reservoir or conservative scalar flux (`src/box_flux_step.rs:7-19`). |
| Newtonian tensor viscosity | `ViscosityWorkspace`, `src/viscosity.rs:1-3,37-60`; `LiquidTransportSimulation::step_viscous`, `src/liquid_step.rs:502-531` | Fully filled sealed box; stationary outer walls are free-slip. Explicit stability number is limited to 1/4, without automatic substeps. Public viscous step cannot combine box flux, surface geometry or phase sources. |
| Embedded fluid geometry | `StaticObstacleGeometry::new`, `src/static_obstacle.rs:1-3,61-73`; volume/area/component getters, `:232-257` | Only admitted closed axis-aligned box mesh. Retains actual volumes and shared open areas; no arbitrary hull, wall patches with traction trace, or motion. `CollisionMesh` admission is a separate contact contract. |
| Pressure around a static obstacle | `StaticObstaclePressure`, `src/obstacle_pressure.rs:192-213`; `project`, `:548-576`; `pressure`, `:371` | Binary64 caller arrays, connected-component gauges, actual divergence and energy gates are real. Outer and fully blocked faces must be exactly stationary. No transport, prescribed external flow or load integration. |
| Stationary no-slip shear | `StaticObstacleShear`, `src/obstacle_shear.rs:49-60,78`; `ObstacleShearWall::NoSlip/Navier`, `:9-16`; `step`, `:253` | Reduced extruded channel with one speed per layer; tangents invariant/periodic. Has body/wall impulse and work, but no general 3D stress, hull force vector or moments. |
| Three-dimensional aligned no-slip strain | `AlignedStrain`, `src/aligned_strain.rs:1-7,231`; `apply/diagnose`, `:555-578` | Exactly aligned padded box, sealed free-slip outer walls, stationary no-slip obstacle. Public owner is diagnostic, with unenclosed step-size estimates; not a public integrator or boundary-load API. |
| Accepted viscous-plus-pressure step around box | Path-import `experiments/aligned_stokes.rs`, `Workspace`, `:222-247`; `step`, `:556-675` | Real unforced experimental transaction; retains accepted pressure/certificates. No advection, forcing, nonzero boundary, clock or generation. It is outside `src/lib.rs`. Encircling decay is not an external wake. |
| Prescribed or finite-mass moving wall | Path-import `experiments/plate_coupling.rs:1-14,50-66`; `Plate::{Prescribed,Dynamic}`, `Workspace::step` | Two tangential planar plates, fixed normal volumes, scalar speeds and impulses. No wall position, normal pressure, torque, rotation, finite edges, hull motion or advection. Public `PlateSession` remains proposed/deferred in `docs/plans/public-simulation-api/reports/moving-plate-integration.md:13-23`. |
| Mesh contacts and prescribed translation | `CollisionMesh::resolve_segment`, `src/mesh_collision.rs:1-4,432-445`; `TranslationInterval`, `src/motion.rs:1-2,44-63` | Point sweep/normal contact removal or relative geometric queries. Contact force outputs, fluid no-slip, swept liquid volumes and body solver are absent. Translation supports no rotation. |
| Emitter motion/propulsion | `RotatingCylinderEmitter`, `src/emitter.rs:1-4,9-24`; `center_at/peak_velocity_at`, `:65-94`; `Simulation::step_with_emitter`, `src/simulation.rs:341-344` | Moving source with inherited nozzle speed and opposed jets. It is not a solid boundary, propulsor performance model or coupled reaction force. Inlet perturbations are not validated turbulence. |
| Gravity/hydrostatic inputs | `BodyForce`, `ForceUnits`, `src/forces.rs:1-27`; `SmokeSource`, `src/simulation.rs:20-26` | Constant acceleration/force density and work are real; smoke Y acceleration is authored forcing. Neither field is a buoyancy/body-displacement model. No generic accepted pressure-plus-hull-hydrostatic load contract. |
| Pressure force/shear vector/moment | No such public hull API in `src/lib.rs:161-175`; pressure and strain reports above | Absent. Diagnostic internal viscous force is not integrated boundary traction. Do not encode unavailable loads as zero. |
| Advancing exterior wake | Filled-box advection is real; obstacle Stokes kernel excludes it | No integrated external obstacle convection/no-slip/load solver. A creeping-flow field can show a disturbance; it cannot support a finite-Reynolds separated/turbulent wake claim. |
| Public session/API | `SESSION_CAPABILITIES`, `src/session.rs:24-51`; `PressureSession`, `src/pressure_session.rs:104` | Transport-only and sealed-box provider scopes are explicit. Snapshot pressure is last interval projection pressure (`src/session.rs:60-63`), not automatically physical end-time hull pressure. |

The separate `TranslatedViscousFlow` is not an escape hatch: its selected family
is `u=(a,0,w)` with zero relative xy transport and atmospheric relative pressure
(`src/translated_viscous.rs:1-3`). Its fitted/column geometry does not represent
exterior submarine flow.

## Deciding load counterexample and existing plan constraints

`docs/broader-physics-milestone.md:39-44` records that adjacent-cell pressure on a
unit embedded cube gives **twice the affine physical force** at the selected grid
spacing although gauge cancellation and a discrete work identity pass. This is
source evidence of a known failure, not a fresh reproduction. Uniform-pressure
closure and energy balance alone cannot accept a hull pressure trace.

That milestone's next geometry/transport/motion/body gates (`:93-118`) explicitly
defer external boundaries, forcing, arbitrary meshes, loads and moving topology.
`docs/plans/moving-plate-coupling/plan.md:24-31,106-110` likewise excludes normal
motion and points to pressure-load bias. The compressible design already selects
z-up, prescribed `(Ux,0,Uz>0)`, constant-density viscous prerequisites and
stationary-body before translating-body validation
(`docs/plans/cpu-compressible-solver/reports/design.md:170-173`). Its main plan
requires preserving incompressible/provider contracts (`plan.md:65-73`); PR76 is
not the owner of this new incompressible route.

## Bounded first runnable slice: fixed synthetic cuboid external Stokes

Proposed numerical owner: one path-import module
`experiments/submarine_stokes.rs`, consumed by one native example and contracts.
Borrow an immutable `StaticObstacleGeometry`; admit exactly a grid-aligned
internal cuboid with enough full-cell padding for the selected wall pressure and
shear traces. Use an original synthetic fixture, not SUBOFF or a CAD surrogate.
Constructor fixes geometry, positive constant rho/mu, frame, memory cap and
numerical settings. A solve request supplies a finite uniform outer velocity
vector and cooperative cancellation. Accepted solution is velocity, gauge-fixed
pressure, separate pressure/shear force and moment, boundary work/residuals,
geometry/config identity and accepted request identity. A stationary solve has
no invented physical timestep or trajectory.

Select **steady Stokes**, `-div(2 mu D(u)) + grad(p) = rho g`, `div(u)=0`, with
convection excluded. Prescribe the full uniform vector on all six outer planes,
not just side-normal velocities; enforce zero velocity at the fixed body. This
is a finite Dirichlet domain whose inlet and outlet are portions of the outer
box, not an infinite-domain or pressure-outlet approximation. The uniform vector
has balanced outer flux. Straight and oblique requests must use the same
admitted discretization. Start with zero gravity/dynamic pressure and separately
qualify gravity/hydrostatic pressure and buoyancy signs before attaching depth.

The I1 backend proposal is a bounded small direct mixed velocity-pressure system
with pivoting and numerical acceptance, initially capped at **4096 combined
active velocity and pressure degrees of freedom**. Preflight actual matrix,
factorization and candidate capacities against the caller payload cap. This
bound does not establish a practical maximum-grid runtime or scalable sparse
solver guarantee. Matrix derivation, gauge/rank, pivoting/conditioning and
independent residual acceptance remain pre-execution derivation gates; no
dependency import is selected or authorized by this audit.

Implementation must derive affine symmetric-strain rows including known outer
trace terms, compatible divergence/gradient, and explicit wall patches with
positions/areas/normals. It must compute a wall pressure trace that passes the
known affine-load counterexample, and Newtonian wall traction using the same
geometry. For body-outward normal `n_b`, report pressure traction `-p n_b` and
viscous traction `2 mu D(u) n_b`, and `r cross traction` about a declared origin.
State whether pressure is dynamic or includes `p_h = p_ref-rho*g*(z-z_ref)`.
Keep discrete boundary reactions distinct from reconstructed physical traction
until their consistency is established; a passing algebraic reaction/work test
cannot replace affine trace accuracy. Load acceptance requires both, or a
derived compatible trace/reaction mapping that unifies them.

Retain capped candidate buffers and previous accepted fields/report. Publish
solution, loads and metadata only after stored divergence, Stokes momentum,
boundary closure, finite arithmetic, pressure/traction and cancellation gates.
Return typed unsupported geometry/domain/platform and numerical refusal. Do not
relax tolerances, invent zero outputs or publish a converged velocity while its
load gate failed. Native records carry exact source/binary identity and the
actual accepted arrays; generated results live outside Git.

Deciding host evidence: actual straight and oblique nonzero solutions/loads on
Linux x86_64/pinned Rust 1.92.0; independent dense/rational tiny-system check;
constant and affine pressure force/moment controls including the known biased
cube; hydrostatic rest and displaced-volume buoyancy; Couette/Poiseuille
traction/work controls; discrete total boundary momentum and energy/work;
axis/sign/scale/linearity controls for Stokes; rejection/cancellation preserving
the complete accepted tuple. Add grid and domain refinement for the synthetic
body before treating its drag as continuum evidence. Corners may reduce load
convergence order; prescribe evidence-based limits rather than assuming second
order. This slice provides no convective wake or submarine validation.

## Reuse and dependency comparison

| Candidate | Relevant value | Decision |
| --- | --- | --- |
| Wrap `experiments/aligned_stokes.rs::Workspace` | Existing interval acceptance, bounded buffers and rollback | Insufficient: `step` accepts only velocities/dt; `:618-624` calls sealed `StaticObstaclePressure::project`, which refuses nonzero outer faces. Its energy-decay gates exclude external work. Reuse patterns and regression evidence, not the incompatible owner. |
| Reuse `AlignedStrain` directly for external affine operator | Stable face/row/geometry getters; existing internal/no-slip row controls | Insufficient as complete operator: outer shear is analytically omitted (`src/aligned_strain.rs:1-7`), known boundary values have no affine columns. Preserve existing source; derive only the new external operator under its separate owner and cross-check shared interior/hull terms. |
| Compose `BoxFluxWorkspace` + `ViscosityWorkspace` | Filled-box pressure flux and free-slip strain kernels | Wrong domain and closures: neither represents the obstacle and viscosity excludes nonzero normal walls. No compatible composition is currently exposed. |
| Reuse `StaticObstacleGeometry` + `experiments/outward.rs` | Immutable admitted geometry; common volumes/open areas/components; existing qualified scalar interval arithmetic | Selected reusable inputs. Borrow geometry once; charge retained capacities once according to the new cap. Do not clone numerical authority or fork interval types. |
| Existing reduced Thomas/plate solver | Proven layered SPD chain | Does not solve a general velocity-pressure saddle-point system. Retain as regression oracle only. |
| Bounded direct mixed Stokes system versus established linear algebra dependency | Tiny-system saddle solve, pivoting, deterministic failure, bounded memory and independent equation checks | I1 proposes the original finite safe-Rust owner with an initial 4096 combined active velocity/pressure dof cap, as recorded in the canonical first-slice and adoption comparison. Method/backend derivation and numerical acceptance remain pre-execution gates. No dependency import or scalable sparse-solver guarantee; a local finite solver needs independent acceptance and no-dependency preference alone is insufficient. |
| Import donor rigid/contact stack or compressible solver | Separate isolated body/contact or Euler capabilities | Does not supply required external incompressible wall work/traction. No wholesale merge, solver change or PR76 dependency. |

## Canonical implementation sequence

Use the [main plan's exact prospective write sets](../plan.md#milestones-and-exact-prospective-write-sets)
and its [first-slice contract](first-slice.md). They own future path selection;
this audit does not maintain a competing implementation table.

I1 is the steady external Stokes lab above. I2 adds finite laminar inertia,
curved stationary geometry and fixed/body-frame flow with wake-compatible
boundaries. I3 adds accelerating smooth nonrotating B-frame forward/ascent;
it requires `-rho*dV/dt` and atomic accepted field/pose/velocity/clock rather
than merely relabeling oblique stationary flow. I4 adds W-frame moving geometry,
covered/uncovered conservative transfer and GCL. I5 admits independently
licensed matched experimental validation; I6 optionally adds attitude and its
rotating-frame terms. I7 owns library, recording and Tuldok promotion after
numerical qualification, following the [storage/API contracts](storage-contracts.md).

A finite constant-speed creeping-flow frame transform can be checked against
I1 without waiting for I2. That special case has no acceleration or rotation
terms and is not I3's accelerating trajectory capability. It also does not
establish that an actual submarine's Reynolds number lies in the creeping
regime. Finite-Re wake claims require I2's convection and boundary gates.

Turbulent wall closure, free 6-DOF feedback, propulsor, cavitation and free surface
remain separate model decisions. Future recording/export integration must
inspect the then-accepted recording owner's field/pressure/stress/frame
contracts; PR76 is protected and no archive schema is invented in this audit.

## Ownership and blockers

Composed-design review is applicable. The new numerical module owns external
assembly, solve, trace/reaction consistency and atomic accepted values. Geometry
remains with its immutable existing owner; native example owns original fixture
and output; checker owns independent equations/controls. Callers supply physical
inputs and caps, not hidden row layouts or staged mutation sequences. Geometry
changes touch the geometry/operator admission; boundary/load changes touch the
numerical owner and oracle; UI/archive changes touch later consumers. Deleting
the numerical owner would scatter necessary coupled acceptance across callers;
deleting the oracle would remove independent physical-equation evidence. No new
registry, provider adapter, persistence or generalized CAD layer is needed.

Blocking facts for implementation admission: compatible external affine
pressure/strain/load derivation and acceptance of the proposed bounded direct
mixed-system method/backend. Blocking facts
for a submarine claim: matched regime/hull/depth, convective or creeping scope,
licensed benchmark uncertainty and grid/time/domain refinement, then chosen
motion model. None prevents completing this planning audit. No action was denied
or left with uncertain tool outcome.

Reviewed Coding-Standards at `188beda1fa477d21d576c233dd8c7f4c4c267d23`:
`topics/architecture.md` (composed design/authority), `topics/contracts.md`,
`topics/dependencies.md` (complete requirement and reuse/adoption comparison),
`workflows/planning.md`, and supporting Rust/dependency references. Its
`.agents/skills/standards-engine/SKILL.md` explicitly limits that authoring skill
to standards maintenance; no application-purpose Engine approval is claimed.
No AGENTS instructions were found in this checkout or its parent directories.
Repository output policy is `docs/REPOSITORY_OUTPUTS.md:3-19`; pinned toolchain is
`rust-toolchain.toml`. This audit used source inspection only: historical test
names/evidence are inventory, not freshly passed native numerical checks.
