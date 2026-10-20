# Independent audit of Rheon's general fluid methods and mesh obstacles

Audit date: 2026-10-19 UTC. Audited remote `main`:
**`bc2f472787060d9a69cb619ed2c2fea10fd754b6`**. This is an independent source
inspection and fresh CPU execution, not a review of a proposed solver design.
No production solver was repaired. Findings are relative to using water as a
physically accurate reference dataset; several are explicitly documented model
limitations, not violations of Rheon's stated preview contract.

## Conclusion

The fixed-grid pressure and symmetric-strain components pass useful analytical
water verification. The restricted channel is second-order in space and
first-order in time in this audit, although its coarsest resolution misses the
fixed accuracy target. The conservative volume transport conserves its represented
volume but visibly diffuses interfaces in an independently checked native export.
These successes do **not** establish a composed general 3D liquid solver.

The production particle preview genuinely collides with internal triangle meshes,
and all tested centers remain outside solids. It does not supply physically
calibrated water/solid boundaries. Parcel mass stays constant while estimated
liquid volume, pressure and initially resting fluid depart substantially from the
water screens. Meshes do not occlude pressure/viscosity neighbors. Do not promote
these particle outputs to physically accurate reference water, impact loads or
wall shear data on the evidence available here.

There is no experimental validation result in this audit. The primary MARIN
benchmark was inspected; the completed release/obstacle specimen is explicitly a
CPU feasibility and conservation test, not a reproduction of that experiment.

## Provenance, authority and retained work

- The supplied checkout was `work` at `6ba41c7`. It was clean. Only `origin/main`
  was fetched; a new worktree `/workspace/Rheon-fluid-audit` and branch
  `audit/general-fluid-physics-20261019` were created. Original branches and
  checkout were preserved.
- `git ls-remote` independently confirmed the main OID above and PR59's
  `perf/accepted-pressure-warm-start` at
  `7f1733f0dc956ef8f25d5b110d990117fb765903`. PR59 was read, not changed,
  rebased, merged or used as the audited implementation. Current main starts
  pressure cold (`pressure.rs:565`); warm-start performance is outside scope.
- No workspace ancestor, checkout or main-tree `AGENTS.md` was present. No
  repository coding-standard file was found. Existing project standards records
  identify [MrScripty/Coding-Standards](https://github.com/MrScripty/Coding-Standards).
  Its freshly cloned main was `0fc92a24279f3a4574c80af0a2f31ea347a21597`.
  Core, Router, Implementation, Verification, Independent Oracles, Documentation,
  Commit and the Rust profile were read. No Standards MCP action is exposed;
  this is source-guided application, not an engine conformance certificate.
- Applicable repository rules: [output policy](../../REPOSITORY_OUTPUTS.md),
  pinned Rust 1.92.0, safe Rust, strict Clippy/formatting, independent oracles and
  preserved scope/refusal contracts. Only authored audit documentation, a probe
  example and its summarizer are added. Generated fields/logs remain outside Git.
- The [protocol](protocol.md) was committed as
  `d3370ef40b156df9c8151a276853c5d10323e076` before probe evaluation.
  Pressure MMS was added before that new case was evaluated; the original
  criteria were unchanged. All failed screens are retained.
- This task's branch/worktree is retained-protected for review and reproduction;
  the branch ref protects its commits. Owner: audit task/repository maintainer;
  target: main; next disposition: review the focused audit PR, then maintainers
  may retire the task worktree after preserving the evidence. No cleanup of other
  worktrees or branches is authorized by this record.

## Water, units and physical scope

Reference temperature is **293.15 K (20 °C)**, near **0.1 MPa**. All probes use
SI: metres, seconds, kilograms, pascals and joules. Rounded inputs intentionally
match the water preset in [material.rs](../../../src/material.rs#L108):

| Property | Audit value | Interpretation |
| --- | ---: | --- |
| Density rho | 998.2 kg/m³ | constant reference mass density |
| Dynamic viscosity mu | 0.001002 Pa s | stress per velocity gradient |
| Kinematic viscosity nu=mu/rho | 1.0038068523342015e-6 m²/s | momentum diffusivity |
| Surface tension sigma | 0.0728 N/m | input to preview cohesion; not a validated effective tension |
| Gravity magnitude | 9.80665 m/s² | prescribed conventional acceleration |
| SPH liquid sound parameter | 20 m/s | deliberately artificial stiffness |

Primary property authority is [IAPWS SR6-08(2011)](https://iapws.org/technical-guidance/release/LiquidWater.download),
sections 2–3. Its density correlation uncertainty in stable water is 0.0001%;
its viscosity section gives a general 1% uncertainty and discusses a 0.17%
reference uncertainty at 293.15 K. Evaluating equation 7 gives approximately
0.0010016 Pa s; the preset is rounded. Those property uncertainties are separate
from truncation error. The [IAPWS surface-tension release](https://iapws.org/technical-guidance/release/Surf-H2O.download)
gives about 0.07274 N/m at 20 °C, with tabulated uncertainty 0.00036 N/m. It
concerns equilibrium water/vapor, not a calibration of the preview force.

The single-density MAC model assumes incompressibility. The particle model is
weakly compressible with fixed material phase, reference viscosity and density;
source temperature does not provide a temperature-dependent water-property model.
Neither boiling, cavitation, dissolved gases nor a resolved air/water interface
is established. No turbulence closure or near-wall turbulence model was found;
emitter perturbations are prescribed forcing, not such a closure. The channel
benchmark has Umax approximately 0.001245 m/s and gap Reynolds number about 1.24,
an appropriate laminar regime. Metre-scale jets at 1 m/s have Re approximately
one million, so physical turbulence would require substantially more spatial
qualification than these low-resolution previews. This does not make turbulence
mandatory for every admitted low-Re component test.

For the SPH EOS, a hydrostatic depth H gives the scaling
`delta_rho/rho ~ g H / c²`: at c=20 m/s, 1 m implies about 2.45% compression.
A 1% density target would restrict this estimate to H<0.408 m, even with an ideal
boundary discretization. Thus one cannot interpret this input sound parameter as
physical water compressibility or the adaptive timestep as a water-accuracy bound.
The capillary length from the rounded inputs is about 2.73 mm; uncalibrated
cohesion/contact angles are especially consequential near that scale.

## Solver inventory and capability assessment

The inventory follows exported owners in [lib.rs](../../../src/lib.rs) and their
actual constructors/steppers, rather than counting pressure preconditioners as
separate physical solvers. The browser adapters call the same Rust grid owners;
viewer interpolation is not another physical solver.

| Implemented path | Equations/discretization and actual boundary scope | Audit assessment and test trace |
| --- | --- | --- |
| `Simulation`; Jacobi PCG / SGS PCG | constant-rho MAC projection, semi-Lagrangian velocity/tracer advection; fixed impermeable box; prescribed body force optional | Both pressure methods pass water hydrostatics and pressure MMS here. Different preconditioners solve the same equations. Advection is not conservative momentum transport; tracer integral is appearance, not liquid mass. `operator_contract`, `pressure_contract`, `advection_contract`, `forces_contract`, `implementation_contract`. |
| Box-flux workspace and `step_with_box_flux` | affine normal inlet/outlet speeds; net-flux compatibility, pressure boundary work, explicit tracer policy | Useful prescribed carrier flow. No general mesh obstacle or evolving liquid stress boundary. `box_flux_contract`, `box_flux_step_contract`. |
| `LiquidVolumeState` | shared signed face transfers of cell fractions; donor-cell unsplit FV and optional reconstructed-column normal sweep | Conservative represented volume, boundedness/Courant checks; no standalone momentum/pressure feedback. Fresh binomial-oracle replay below quantifies interface diffusion. `liquid_volume_contract`, `liquid_scale_division_contract`. |
| `LiquidTransportSimulation::new/step` | publishes projected carrier and transported fraction together | All-fluid constant-rho carrier remains occupied even where represented fraction is zero. Represented mass does not set local inertia; this is not a general free-surface solver. `liquid_step_contract`. |
| Slab and reconstructed-column surface variants | atmospheric p=0 at wet/air face; half-cell or ghost-fluid distance; lower-wall-connected column reconstruction | Slab hydrostatics works. Mixed/unresolved slab updates, thin/disconnected/steep unsupported columns and provider/boundary incompatibilities refuse. No arbitrary topology, air inertia, viscous normal stress or capillarity. `free_surface_contract`, `column_surface_contract`. |
| `ViscosityWorkspace`; filled-box `step_viscous` | explicit symmetric Newtonian strain on MAC faces; stationary free-slip outer walls; fully filled matched-density liquid only | Water vortex refinement and dissipation pass. Partial/free-surface domains explicitly refuse; no imported solid viscosity. `viscosity_contract`. |
| Column shear / momentum / flat MAC bridge | periodic tangential profiles, flat height; explicit shear with traction/Navier/no-slip variants; conservative overlap remap and cap exchanges; fixed-flat parcel/MAC transfer | Separate prerequisites, not a composed general evolving surface momentum step. Flat MAC publication makes later transport refuse. `column_shear_contract`, `column_momentum_contract`, `column_mac_contract`. |
| Fitted-height workspace | periodic fitted 2.5D strip; discrete divergence, pressure basis and symmetric strain | Instantaneous operators, no clock or independent general integrator. `fitted_height_contract`. |
| `TranslatedViscousFlow` | restricted u=(a,0,w), translating fitted frame; zero relative xy flux, atmospheric pressure; weak Neumann viscosity | Explicit special solution family, not arbitrary pressure dynamics. `translated_viscous_contract`. |
| `FixedBottomAleFlow` | same restricted velocity family; fixed bottom coordinates, moving cap and conservative relative transport | More general mesh-relative transport than pure translation, still not general liquid flow. `fixed_bottom_ale_contract`. |
| `CoupledDiscreteFlow`, extruded third component | fixed small two-column periodic topology, integrated moving mass and first-order endpoint donor momentum; bounded coupled pressure/strain update | Constructor explicitly requires rho=3, mu=0.05 and a specific slice (`coupled_discrete.rs:301`); cannot instantiate the audit water constants. Three velocity components do not make it general 3D. `coupled_discrete_contract`, `extruded_third_contract`. |
| Static-obstacle pressure and reduced shear | exact clipped measures of one axis-aligned box; area-weighted projection, or distinct periodic fully developed channel | New water channel verification passes refinement; coarse accuracy target fails. Pressure/shear owners are not one general 3D mesh/liquid integrator. `static_obstacle_contract`, `obstacle_flow_contract`. |
| Opt-in aligned Stokes experiment | padded grid-aligned internal box; explicit full strain, pressure proposal, interval energy/divergence/momentum-defect gates | Useful finite discrete unforced viscous benchmark; no advection, gravity, interfaces or arbitrary meshes. `aligned_stokes_contract`; no Lean kernel run in this audit. |
| Opt-in external/submarine Stokes experiment | steady full-vector creeping Stokes on a padded aligned cuboid, stationary no-slip body, all six outer velocity planes prescribed; mixed velocity/pressure dense solve with row scaling, bounded refinement and 4096-DOF cap | A real field/load experiment, not just planning. Reconstructs pressure/shear patches and separately reports functional reactions; no convection, clock, free surface, arbitrary imported hull or turbulence. `submarine_stokes_contract`; historical native/MMS reports are not new continuum/experimental validation here. |
| Opt-in moving-plate experiment | backward-Euler reduced shear coupled to prescribed or finite-mass sliding plates; shared conductances, fluid/plate impulse and interval work/energy checks | Actual two-way **tangential** plate coupling in a periodic/invariant channel. No normal geometry change, mesh rotation, pressure, torque or general rigid-body collision. `plate_coupling_contract`. |
| `ParticleFluidSimulation` / `MaterialSession` | reference-volume SPH density, positive liquid EOS, pair pressure/viscosity/cohesion, symplectic Euler; domain box contact and optional static triangle contact | Only production path combining freely distributed liquid parcels and imported static mesh collision. Physical-rest screens fail; no quantified continuum or experimental water accuracy. Pair/discrete contracts and mesh tests are materially narrower. |
| `CompressibleSimulation` | periodic cell-centered ideal-gas Euler, local Lax–Friedrichs FV flux and positivity retries | Actual separate gas solver, not a liquid-water EOS or mesh obstacle solver. `compressible_contract`. Learned providers/diagnostic surrogate routes are not independent liquid reference solvers. |

## Equations traced to code and dimensional checks

### MAC carrier, projection and advection

The target continuum form is `div u=0` and
`du/dt+(u·grad)u = -grad p/rho + div[mu(grad u+grad u^T)]/rho + g`.
Only paths explicitly enabling viscosity include that term. In
[operator.rs](../../../src/operator.rs#L68), for a face opening A and center
separation d, `w=A/(rho*d)` and `Lp=sum w*(p_i-p_j)`. With positive outward
predicted flux Q, [build/solve](../../../src/pressure.rs#L336) uses `b=-Q/dt`;
[correction](../../../src/operator.rs) is
`u_new=u_star-dt*(p_plus-p_minus)/(rho*d)`.
`Lp` and b have units m³/s²; p is Pa and u is m/s. Signs agree with eliminating
divergence, not amplifying it. Pressure and Krylov work are f64, stored grid
velocity is f32. That storage roundoff must be tested after correction.

Closed-box gauge and component compatibility are physical Neumann constraints,
not optional mean removal. The regular box pins one pressure gauge, checks the
full RHS sum and recomputes the full residual including its gauge row
([pressure.rs:367,515](../../../src/pressure.rs#L367)). Surface modes use p=0
atmospheric Dirichlet faces and no sealed gauge. Actual stored divergence is
rechecked, not inferred exclusively from a small linear residual. Current main
initializes pressure to zero each solve. The SGS preconditioner is symmetric;
its selection changes convergence work, not the continuum equations.

[advection.rs:28,72](../../../src/advection.rs#L28) traces trajectories with
midpoint/RK2 and trilinearly samples staggered fields. Normal outer faces are
zeroed. This gives bounded interpolation, not conservative momentum or tracer
mass, nor a proof that the complete split solver is second-order in time.
Forces distinguish acceleration from N/m³ and divide force density by rho.
The timestep uses velocity/acceleration Courant bounds; accepted divergence and
actual Courant are separately gated (`simulation.rs:740–991`). Semi-Lagrangian
stability and an accepted transaction do not bound physical advection error.

### Viscosity and represented liquid

[viscosity.rs:194,249,394](../../../src/viscosity.rs#L194) constructs normal
strain rows weighted by 2V and engineering shear rows weighted by V. Its
quadratic dissipation is `mu*V*(2 sum normal² + sum engineering_shear²)` W,
with cross-component derivatives. The explicit update is the mass-weighted
negative strain transpose; `nu*dt*sum(1/h_d²)<=1/4` is required. It uses **dynamic**
mu and computes nu=mu/rho; no factor-1000 viscosity substitution was found.
Damping/energy identities are finite discrete checks. They do not supply missing
interface stress or mesh no-slip conditions.

[liquid_volume.rs:490](../../../src/liquid_volume.rs#L490) uses, away from the
special column sweep, `T_f=dt*A_f*u_f*C_donor` m³ and
`V*C_new=V*C_old-sum_outward(T)+dt*source`. A shared face transfer is added with
opposite signs to adjacent cells. Outward Courant sum must be <=1 and fractions
are checked without a final artificial clamp. Source/inlet/outlet ledger terms
are explicit. Algebraic conservation does not prevent donor-cell interface
smearing. Pressure classification is only activated in the opted-in surface
modes; it is not a variable-density mixture model.

### Separate steady Stokes and reduced plate coupling

[experiments/submarine_stokes.rs](../../../experiments/submarine_stokes.rs#L614)
assembles the steady mixed Stokes problem
`-div(mu*(grad u+grad u^T))+grad p=f`, `div u=0`, with explicit outer traces,
stationary-body elimination and a pressure gauge. Strain quadrature includes
sector half-cell corner rows; forcing samples are multiplied by represented
velocity dual volumes. Dense factorization uses a scaled-pivot refusal and up to
four residual correction solves. There is no timestep stability claim. The
constitutive patch load is `(-p*I+mu*(grad u+grad u^T))*normal*area`, with a
consistently chosen body/fluid normal and moment reference. Functional boundary
reactions and reconstructed physical traction are distinct reported quantities;
agreement of one discrete residual does not guarantee agreement of those loads.
The existing implementation report explicitly retains a coarse load discrepancy
and pending continuum/domain refinement. This audit does not relabel historical
receipts as fresh measured water drag. For water, a creeping-flow interpretation
requires choosing U and L so rho*U*L/mu is small; a submarine-shaped name does not
justify applying Stokes flow to ordinary high-Re submarine operation.

[experiments/plate_coupling.rs](../../../experiments/plate_coupling.rs#L257)
adds boundary plate velocities to the reduced shear unknowns. Dynamic plate mass
is in kg for the represented periodic cross-section; conductance is kg/s. The
combined backward-Euler chain solves fluid and plate momentum with equal/opposite
interface impulse. Prescribed velocities instead contribute external work and
impulse. Interval gates check actual stored equation, momentum and energy defects.
This is genuine restricted two-way coupling, but its tangential invariance removes
all normal swept-volume/pressure/torque questions. It is not an exception to the
missing general moving-mesh boundary identified below.

### Particle preview

The rotating emitter is an external reservoir, not a solved pressure inlet.
`jet_speed` derives speed from the entered head by sqrt(2*head/rho) (or an explicit
velocity override). At `particle_fluid.rs:490`, volume credit is
`pi*radius²*speed*0.43*dt` m³; whole reference-volume parcels are born and fractional
credit is retained. Birth positions sample disk area and velocities use the
nonuniform square-root profile. The .43 factor is a documented approximation,
not calibrated nozzle discharge. Correct total emitted mass bookkeeping does
not by itself verify the radial mass-flux profile or downstream water density;
those require a flux-weighted source/inlet benchmark. Gravity is acceleration
in m/s², not a density-scaled weight input.

The implemented poly6 density is
`rho_i=rho0_i*max(0.05, sum_j (m_j/rho0_j)*W_poly6(r_ij,h))`, h=2*initial spacing.
Its liquid pressure is `max(c²*(rho_i-rho0_i),0)` Pa
([material.rs:65](../../../src/material.rs#L65),
[particle_fluid.rs:591](../../../src/particle_fluid.rs#L591)). Missing liquid
neighbors at free surfaces and solids lower density; no wall ghost contribution
restores support. The 0.05 floor is regularization, not a physical vacuum model.

At [particle_fluid.rs:656](../../../src/particle_fluid.rs#L656), pair pressure
acceleration uses `-m_j*(p_i*rho0_i/(rho0_j*rho_i²)+p_j*rho0_j/(rho0_i*rho_j²))*grad W_poly6`.
This has units m/s² and gives equal/opposite pair forces. Using the same density
kernel makes the unclamped compression-energy derivative consistent; fresh
`material_reference_contract` checks it with independent finite differences.
That is code correctness, not consistency of the discrete boundary with water.

Viscous acceleration is `sum_j m_j*mean(mu_i,mu_j)/(rho_i*rho_j) *
(45/(pi*h^6))*(h-r)*(v_j-v_i)`, also m/s². The positive coefficients dissipate
relative velocity and conserve pair momentum. Cohesion uses a symmetric
attractive spiky-kernel force with coefficient sigma*Vi*Vj/(mi*h), where
Vi=mi/rho_i. Units are consistent, but no calibration to Laplace pressure or
curvature traction follows from those units or an attractive pair fixture.

Integration is `v+=dt*a; x+=dt*v` followed by box and mesh response
([particle_fluid.rs:743](../../../src/particle_fluid.rs#L743)). Acoustic/advection,
nu, sigma/rho and gravity limits are applied; subsequent force and damping checks
can restart preparation, up to 16 retries. No silent speed/force clipping was
found. These are sufficient local refusal controls for the tested preview,
not a general energy-stability or accuracy theorem. Particle diagnostics describe
the **preintegration evaluation** of the last accepted step, not its final field.

Independent infinite cubic-lattice summation of the actual stencil at h/spacing=2
finds density/reference=1.009775166894685 and leading smooth-shear viscosity
nu_effective/nu=0.9690771396841362. This is a mathematical stencil diagnostic,
not a native shear-flow experiment. The initial bulk density bias would produce
about 3903 Pa at c=20. Reducing spacing while keeping the same dimensionless
neighbor stencil does not remove these particular moment biases. Boundary
support and particle disorder cause additional errors.

## Mesh-obstacle path

### Import, placement and admission

[mesh_import.rs:105](../../../src/mesh_import.rs#L105) imports native OBJ,
glTF 2.0 and GLB; it does not just send triangles to a renderer. OBJ positions
have no automatic unit inference, centering or welding; explicit scale maps
source numbers into metres. glTF node matrix/TRS hierarchy is applied before
positive uniform placement scale and XYZ translation. Reflections reverse
winding; nonfinite/singular/ambiguous transforms refuse. Source axes are
preserved. UI placement has no per-axis rotation; encoded glTF rotations and
already rotated vertices are supported. The fresh importer contracts include
analytic rotated hierarchy/reflection cases, polygons, negative OBJ indices,
unsupported features and malformed resource refusal.

A `CollisionMesh` builds a balanced AABB BVH and numerical tolerance
`epsilon=max(span*1e-10, magnitude*64*eps64, 1e-15)`
([mesh_collision.rs:132](../../../src/mesh_collision.rs#L132)). Convex-solid
policy requires connected indexed closure, opposite paired edge winding, no
unused/duplicate vertices and convex halfspaces; global winding reversal is
accepted. Open, concave and inconsistent solids refuse. Two-sided surface policy
allows open/disconnected/concave surfaces but does not infer a solid or promise
self-intersection reliability. Bounds must fit the entire physical domain.
Geometry/resource/precision limits reject rather than silently substituting a box.

### Particle collision and its limits

[particle_fluid.rs:776](../../../src/particle_fluid.rs#L776) sweeps old to
proposed center against actual BVH triangles. At earliest contact,
[resolve_segment](../../../src/mesh_collision.rs#L441) retreats by a numerical
normal clearance, removes normal velocity, and stops all velocity for tied
nonparallel contacts. It discards unused motion after contact; tangential velocity
can resume next step. Long unresolved sweeps refuse. This is swept geometric
collision of **points**, not voxelization or an SDF, and not just an amber
wireframe effect. Rendered/kernel support radii can overlap the surface.

There is no calibrated mesh pressure, no mesh ghost particles, no no-slip/Navier
law, wall heat/friction response or body load/impulse ledger. Domain walls instead
use radius=0.2*spacing, position clamp, zero inward normal velocity and exponential
tangential damping from `wall_drag` s^-1; that is different from a resolved
no-slip stress condition. Neighbors/forces remain unoccluded across triangles.
The emitter cylinder is a source illustration, not a collision solid.

Static mesh attachment at generation zero excludes inside/on centers and removes
their mass; after stepping, overlap replacement refuses. Removal does not restore
excluded mass. The audit deliberately uses this documented generation-zero API to
construct its half-filled/released initial conditions and reports exclusion
separately. It is not a leaking simulation or an emitter mass discard.

### Grid obstacles and motion are distinct capabilities

[StaticObstacleGeometry](../../../src/static_obstacle.rs#L414) accepts exactly
one eight-vertex/twelve-triangle **axis-aligned box**, computes clipped fluid cell
volumes/open face areas and rejects unresolved cell topology. It is not a general
mesh rasterizer. Rotated solids admitted by particle collision are not thereby
admitted to this grid owner. Pressure uses the same openings and one gauge per
wet component, but nominal center distances; its mass `rho*A*d` is an area-lumped
choice, not the exact clipped staggered volume. A pressure center may lie in
removed solid. Therefore symmetry, flux cancellation and discrete energy balance
do not establish continuum accuracy of arbitrary cut-cell pressures or loads.

`StaticObstacleShear` is a different, periodic/invariant fully developed channel
with centroid distances and stationary no-slip/Navier walls. It is valid for that
reduction; it cannot be composed indiscriminately with sealed 3D obstacle pressure.
The aligned Stokes experiment supplies one bounded aligned viscous/projection
composition, with additional interval checks, but no general transport/interface.

`TriangleSurface` and `TranslationInterval` provide geometric static/translating
queries; `step_with_tracer_barrier` affects passive tracer visibility only. None
supplies a moving-wall fluid volume, swept-volume conservation, wall pressure
work or equal/opposite fluid/body wrench. No moving/rotating mesh or two-way
rigid-body solver is enabled by the tested collision path. The separate reduced
plate experiment does couple tangential fluid/plate inertia, as inventoried
above; it is not a moving triangle-mesh fluid solver.

## Fresh numerical evidence

The probe source is [fluid_physics_audit.rs](../../../examples/fluid_physics_audit.rs);
its [summarizer](../../../tools/summarize_fluid_physics_audit.py) reports failed
screens instead of treating successful execution as physical acceptance. Actual
binary64 outputs, particle coordinates and channel profiles are in external
`probes-final/`. All thresholds are in the committed protocol; they were not
relaxed after observing results.

### Analytical pressure and viscosity

| Probe | Resolution / timestep | Actual error | Result |
| --- | --- | --- | --- |
| Water slab hydrostatics, both PCGs | 8/16/32 wet layers over 0.5 m; dt=0.001 s; sealed sides/bottom, atmospheric top | max pressure error <=1.182015e-4 Pa; max corrected speed <=6.86e-15 m/s; max wet divergence <=6.75e-13 s^-1 | All fixed thresholds pass. Pressure error is consistent with f32 storage of -g*dt; it is not spatial convergence evidence by itself. |
| Neumann pressure MMS p=1000*cos(pi*y/0.5) plus gauge | 8/16/32/64 cells; independently assembled continuum RHS; both PCGs | RMS Pa: 15.65884, 3.929765, 0.9833875, 0.2459061 | Error ratios about 0.25 pass; second-order spatial behavior for this smooth interval. |
| Water no-slip channel, gap 1 mm, acceleration .01 m/s² | 8/16/32 fluid layers; exact initial steady profile; BE dt=1 s, t=20 s | relative RMS: **2.139313%, 0.534882%, 0.133721%** | n=8 fails the fixed <2% target; both refinement ratios pass. This is ordinary coarse truncation, not evidence of an incorrect viscosity unit. |
| Unforced channel sine decay | fixed 32 layers; t=.05 s; 8/16/32 steps | RMS vs semidiscrete exact: .006386725, .003245675, .001636295 m/s | Ratios .5082/.5041 pass first-order time criterion; energy decreases every step. |
| MAC viscous divergence-free sin/cos vortex, free-slip 1 mm square | 8/16/32; 13/52/206 explicit steps to .01 s | RMS continuum speed error: 2.946396e-4, 7.418191e-5, 1.839277e-5 m/s | Ratios .2518/.2479 pass; energy nonincreasing. This isolates viscosity, not a full advective Navier–Stokes benchmark. |

Channel force/work identity errors are <=4.85e-26 J in these tiny-volume fixtures.
Small ledger errors do not replace the independently computed profile errors.
The three spatial levels and three time levels support the reported observed
orders only; they do not bound all geometries or all f32 long-time histories.

### Volume transport and interface accuracy

Fresh `liquid_step` execution/export and the existing independent successor
verifier completed all eight cases (both PCGs). It uses prescribed X-directed
slab translation on a 3D grid, rho=800 kg/m³, t=.5 s; this separate legacy control
is explicitly **not** called a water-property test. Volume starts at .125 m³.
At Courant .25, n=16/32/64 final volumes are .125 m³ within rounding, while L1
shape errors are .0583744/.0422247/.0301952 m³. Halving dt at n=64 increases that
error to .0326367 m³. The maximum recorded divergence is below 1.80e-12 s^-1.
The small recorded boundary outflow tail is independently included in the ledger.
Thus conservation passes; a claim that conserving volume proves an accurate
interface, or that smaller dt always improves donor-cell shape, fails.

### Particle water-rest screen

A 1 m sealed box is seeded at rest, without a source, at 8/12/16 samples per side
(512/1728/4096 particles). Gravity, viscosity, cohesion and wall drag are the water
defaults. Requested dt=.001 s; 100 accepted steps reach .1 s. A uniform fill has
no initialized hydrostatic pressure, so this screens initialization/equilibrium
suitability; it is not a proof that a correctly initialized hydrostatic state is
unstable. Lowering the threshold after seeing the transient would be inappropriate.

| Samples/side | Max speed over interval (m/s) | Final density RMS deviation | Gauge-aligned hydrostatic pressure RMS / (rho*g*1m) | Drift of sum(m/rho_estimated) |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 1.109019 | 18.7968% | 34.2590% | +2.27345% |
| 12 | 1.572570 | 14.7157% | 41.5834% | +2.79111% |
| 16 | 1.425279 | 12.5091% | 50.3559% | +2.86515% |

All nine water-rest physical criteria fail. Retained mass drift is zero at the
recorded arithmetic precision. `sum(m/rho_estimated)` is a density-based volume
diagnostic, not an exact geometric free-surface volume; it cannot be substituted
for parcel mass or treated as an independent particle leak. Diagnostics are at
preintegration time .099 s and final positions at .1 s, so the pressure screen
has at most one accepted-step position staggering. This .001 s ambiguity does
not explain errors of this magnitude and no asymptotic convergence is asserted.
Initial/final particle files allow reconstruction of kinetic and gravitational
potential energy. Final kinetic energies for rest are about 396.56/356.07/319.15 J
(check raw output for exact values). Kinetic plus gravity is not a complete SPH
energy invariant because compression, cohesion and contact work also matter;
no uncomputed total-energy conservation claim is made.

### Internal meshes, thin geometry and release

All geometry here lies inside the domain, rather than replacing a box wall:

- Submerged convex box: local `[.35,.65]^3` m, rotated .37 rad in XY about its
  center; cube volume .027 m³. At 8/12/16 samples/side, initially excluded masses
  are 15.596875/27.727778/23.395313 kg versus continuum rho*volume=26.9514 kg.
  The errors are approximately -42.1%/+2.88%/-13.2%, not monotone convergence.
- Surface-intersecting convex box: local x,z=[.35,.65], y=[.25,.75] m, same
  rotation; initially retain water below y=.5 m. Excluded upper-half water and
  obstacle centers are explicit, not accumulated leakage.
- Thin oblique **zero-thickness** barrier: corners (.05,.4,.05), (.95,.6,.05),
  (.95,.6,.95), (.05,.4,.95) m. Full initially resting water on both sides.
- Release specimen: 1 m cube, initial block x<.5 and y<.5 m, retained mass
  249.55 kg; static obstacle x=[.60,.75], y=[0,.35], z=[.35,.65] m. Run .4 s,
  400 steps, at all three samplings. This is an authored dam-release specimen,
  not the MARIN dimensions. Maximum speeds are 3.2275/5.5694/9.9263 m/s; no speed
  accuracy criterion or convergence claim is attached to those values.

Across all these accepted steps there are **zero detected center penetrations**,
zero crossings of the finite barrier, and zero retained-mass drift. Runtime
inside checks reuse the production containment owner; independent inverse-rotation
and strict local-box checks of all exported final states also find zero inside
centers (1e-9 m interior tolerance). The temporal barrier-crossing check uses an
independent analytic finite-plane intersection. Rotation
and increasing sampling do not turn those geometric checks into pressure/slip
validation. Nonvacuity is checked from exported coordinates: final submerged-box
contact-band centers are 6/20/50, barrier contact-band centers 20/60/160, and
release-obstacle contact-band centers 8/8/20 (band 1e-7 m). Fluid has reached the
obstacles. The one zero final contact count in the n=12 surface-box specimen is
not claimed as an independent contact event certificate.

After each .1 s barrier capture, one additional 1e-10 s accepted step evaluates
density at the known .1 s positions for the following independent comparison;
it is separate from the reported 100-step horizon. The oblique barrier exposes
the important remaining interaction: independent
all-pairs kernel sums reproduce native density to <=9.10e-13 kg/m³. Omitting
pairs whose center segment analytically crosses the finite barrier removes a
mean density contribution of **33.3038/17.7712/10.2860 kg/m³**, from
972/1960/3344 directed cross-barrier neighbor pairs. These are actual interactions
through the finite surface, not around its edges. A center barrier therefore
does not hydraulically partition its pressure/viscous neighborhoods.

A separate 300-sweep test uses the rotated cube at scales .001/1/1000 m:
zero penetrations and maximum remaining normal velocity 2.64e-16 m/s. The open
triangle-as-solid fixture refuses; the existing exact-category constructor tests
cover winding/concavity/unused vertices and unresolvable query scales. These
finite tests do not certify arbitrary self-intersecting surfaces, nanometre
geometry, all transform chains or moving solids.

### Numerical uncertainty and evidence limits

Analytical errors above are deterministic differences from closed-form solutions,
not statistical confidence intervals. The Poiseuille finest-grid RMS error is
0.133721% for this exact profile; pressure MMS finest-grid RMS error is 0.2459061 Pa.
The observed refinement ratios support their orders only in these smooth families.
Time error was isolated against a semidiscrete eigenmode; the vortex study instead
refines time with space and reports their combined error. Pressure solve tolerances
were relative/absolute residual 1e-12 and predicted-divergence 1e-8 s^-1; actual
fields were checked independently. Hydrostatic pressure error includes a bounded
f32 predictor representation error, rather than a tunable physics tolerance.

The particle studies have only three modest samplings and one physical-rest
horizon; they do not reach an asymptotic accuracy regime. No Richardson error bar
or experimental confidence interval can responsibly be assigned. Mesh collision
clearance is tied to geometric epsilon (typically 1e-10 times span), not particle
radius or an uncertainty in hydrodynamic loads. The explicit initial exclusion
error and preintegration diagnostic staggering are separate uncertainty sources.
Water-property rounding/reference uncertainty, numerical truncation and model
error have not been conflated into one acceptance statistic.

### Experimental validation boundary

The primary [Kleefsman et al., JCP 206 (2005), section 7](https://pure.rug.nl/ws/portalfiles/portal/2941055/2005JCompPhysKleefsman.pdf)
describes a 3.22×1×1 m open-top tank, .55 m initial depth, an internal box, four
height probes and eight pressure transducers. Its plots are credible physical
references, not scalar acceptance data supplied by Rheon. This audit did not
obtain an independent numerical sensor time series with measurement uncertainty
or digitization uncertainty. Rheon's mesh preview also lacks a matched wall
pressure/traction observation and no-slip boundary model. Those specific missing
inputs/capabilities block a quantitative impact-pressure validation. The grid
components cannot represent that complete interface/obstacle experiment.

No published pressure curve was fabricated, copied from a surrogate, or replaced
by visually similar splats. A future height-only benchmark could begin once
initial geometry, probe definitions and independent numerical observations are
retained; wall-load validation additionally requires a physically meaningful
boundary stress/force observation.

## Severity-ranked findings and minimal repair priorities

Severity is against the intended accurate-water-reference use. **No critical
memory-safety or demonstrated algebraic pressure-sign defect was found.**

| ID / severity | Finding, evidence and consequence | Minimal next repair / qualification |
| --- | --- | --- |
| F1 / High — model suitability | Mesh center contact does not provide a liquid/solid pressure or no-slip boundary. Cross-barrier kernel contributions measured above; source `particle_fluid.rs:591–685,776`. Force transmission through an impermeable separator and uncalibrated wall behavior invalidate impact/shear labels. This is documented scope, not a hidden promise. | Keep preview labels explicit. Qualify one boundary-support formulation on planar hydrostatics first; use a common visibility/support rule for density and forces, with its conservation/energy consequences derived together. Do not only occlude density or add an unexplained pressure correction. |
| F2 / High — reference accuracy | Resting water has 12.5–18.8% density RMS deviation and non-negligible motion; pressure screen worsens under this modest refinement. Artificial EOS, missing support and uninitialized hydrostatics all contribute; this experiment does not isolate their shares. | Add an explicit equilibrium/initial-state facility and a boundary-aware water-rest acceptance fixture. Choose stiffness from allowed compression/Mach/depth and requalify timestep cost. Avoid claiming that increasing preview voxels improves particle physics. |
| F3 / High — missing composition | General 3D viscous free-surface mesh flow is not assembled by any grid owner. Slab/column/full-box/reduced-shear restrictions and hardcoded coupled slice are reachable refusals, not completed capability. | Preserve refusals. Implement and qualify one common fixed-domain obstacle/transport/stress boundary slice before expanding geometry or calling it a general water solver. This audit does not request a broad repair branch. |
| F4 / Medium — spatial/initial-volume accuracy | Generation-zero solid exclusion samples centers, not exact fluid volume. Rotated cube excluded-mass errors -42.1/+2.88/-13.2%; constant retained mass hides initial discretization bias. `particle_fluid.rs:308`. | Expose before/after exclusion mass and estimated solid-displacement error in reference exports; use volume-aware initialization or explicitly qualify sampling across translations/rotations. Do not silently renormalize mass. |
| F5 / Medium — fixed-stencil consistency | Uniform-lattice density and shear moments differ from unity at fixed h/spacing. Refinement alone preserves those dimensionless biases. This is an analytically evaluated implemented-stencil diagnostic, not observed native shear convergence. | Add a native manufactured smooth-shear study and independently vary support/spacing. If correction is needed, preserve pair symmetry and pressure energy consistency; calibrating a preset number alone is insufficient. |
| F6 / Medium — accuracy limits | Channel n=8 misses 2% while converging correctly; donor-cell volume transport diffuses a translated interface, with smaller dt increasing fixed-grid shape error. | Retain convergence numbers and qualify resolution/Courant recommendations. Add no production equation change solely to make the coarse threshold green. |
| F7 / Medium — cut-cell interpretation | Grid obstacle pressure uses nominal centers and rho*A*d inertia rather than general cut dual masses; no continuum cut-cell/load order is established. Rotated shapes are refused. `static_obstacle.rs:414`, `obstacle_pressure.rs:286`. | Run a common-geometry pressure MMS/refinement and traction-quadrature study before enabling arbitrary meshes or physical load output; retain independent cell-flux acceptance. |
| F8 / Medium — capillarity/energy evidence | Cohesion has correct dimensions and pair attraction but no measured effective surface tension, Laplace-drop or capillary-wave qualification. Point contact discards leftover displacement and lacks a wall work ledger. | Start with static drop pressure vs radius and energy/work diagnostics for one planar contact; keep contact angles/adhesion unsupported until independently established. |

Proposed repairs are **separate follow-up work**, not changes included in the audit.
The minimal order is: preserve model labels and export initial mass accounting;
qualify planar wall support/equilibrium; establish a native viscosity consistency
study; then qualify one mesh/interface composition and physical sensor benchmark.
Moving/two-way bodies need swept-volume conservation and matched force/work maps
as a later independent gate.

## Reproduction, actual execution and remaining limits

Executed on Linux x86_64, Rust/cargo 1.92.0, Python 3.12.14, serial CPU kernels.
The environment's supplied toolchain was used via:

```sh
source /workspace/rheon-setup/env.sh
cd /workspace/Rheon-fluid-audit
cargo test --locked --no-default-features
cargo run --locked --release --no-default-features --example fluid_physics_audit -- /workspace/rheon-fluid-audit-evidence/probes-final
python3 tools/summarize_fluid_physics_audit.py /workspace/rheon-fluid-audit-evidence/probes-final > /workspace/rheon-fluid-audit-evidence/summary.json
cargo run --locked --release --example liquid_step -- /workspace/rheon-fluid-audit-evidence/liquid-step
python3 tools/verify_liquid_transport.py /workspace/rheon-fluid-audit-evidence/liquid-step
cargo clippy --locked --no-default-features --example fluid_physics_audit -- -D warnings
cargo fmt --all --check
git diff --check
```

Use **fresh** destination directories on reproduction. `commands.txt` in the
retained evidence also records the focused native test command. The first
`probes/` run is retained; `probes-final/` adds the pressure MMS before its first
evaluation and is the report's authoritative campaign. Original criteria and
original campaign failures were unchanged. All 283 common scalar measurements
are identical between campaigns; eight pressure-MMS measurements were added. Only
the final campaign source/executable was separately retained for reproduction. The summarizer's JSON contains 98
checks: 88 pass, one coarse channel target fails, and nine particle physical-rest
screens fail. Completion of the script is not all-physics acceptance.

Fresh full core suite: **582 passed, 0 failed, 3 ignored**, across 75 result
groups, command exit 0. The three ignored tests require `RHEON_AI_EXPORT` from a hash-verified real
checkpoint exporter: `frozen_model_selection_and_unsupported_domain_fallback_are_visible`,
`real_model_advances_production_session_for_twenty_four_intervals`, and
`real_checkpoint_support_replay_and_corruption`. They are surrogate integration
tests, not missing general-fluid water or collision tests, and were not claimed. Focused solver,
water-material and mesh suite: **125 passed, 0 failed**, command exit 0. The full
suite additionally exercised the external Stokes (13 tests) and moving-plate
(11 tests) experiments. Strict affected-example Clippy, formatting and diff checks
passed. The summarizer agrees byte-for-byte under normal and optimized Python;
missing, duplicate and nonfinite measurements are rejected at their intended
checks. These are CPU regression/record-integrity results, not physical validation.

The [manifest](manifest.json) binds baseline source files, audit inputs, executable
and all retained output hashes. Raw fields/logs and the compressed evidence package
are retained in `/workspace/rheon-fluid-audit-evidence` and separately archived
for durable handoff. Authored source/report/protocol are on the audit branch and
focused PR; generated simulation fields are not committed under repository policy.

No CUDA/GPU execution, native window/UI run, cross-platform runtime qualification,
Lean kernel recheck, experimental water calibration, general turbulence accuracy
or two-way rigid-body result is claimed. Unavailable surrogate A1 data did not
block or substitute for any test. The only scientific-validation blockers are
the specific physical observation/boundary deficiencies described above.
