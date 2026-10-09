# Supplying wall observations from actual solver state

Preserve4a268efd32d6429000ef73d1dd9ccd606ecebe1c and its original failing fixtures. This branch is a research implementation plan with read-only actual-API and exact topology/layout diagnostics. It creates no velocity/pressure solve, step, physical coupling, production change or publication. The16,000,000-byte managed cap is unchanged. No refused native h=1/8 run occurs. No analytic velocity, stress or previous oracle output is an input.

## Outcome and scope

A persistent, genuinely resolved velocity hierarchy could supply the needed observations. The current simulator does not own such a hierarchy, or even accepted velocity state for the retained no-slip obstacle model. Enlarging the cap alone does not fix that. The first implementation phase should establish an owned f64 obstacle-state/provenance contract on admitted uniform grids. Adaptive refinement follows only after its pressure/strain interface contracts and error estimator are derived and independently qualified. Neither interpolation nor an energy-minimizing patch driven solely by coarse data recovers the missing instantaneous information.

The proposed first2:1 composite collar can be budgeted within the existing cap: conservative totals10,388,460 bytes with eight-donor transfer layouts, or14,631,276 with27-donor layouts and explicit reserves. These are resource proposals, not qualified transfer/stencil implementations or physical error bounds. A larger cap is not justified now. A hypothetical existing-row uniform N24 benchmark has a separately specified48,289,256-byte payload and50,000,000-byte proposed envelope; it is not run, requested or authorized, and remains meaningless for qualification until actual physical state/provenance and an error contract exist.

## What the actual code supplies

Simulation owns accepted/candidate f32 full-box MAC velocity and tracer arrays, a uniform GridGeometry and pressure workspace. StateView exposes x,y,z,tracer,time,generation. It carries no obstacle stamp, material/force history, accepted pressure identity or discretization-error enclosure. Config/forces must be logged separately by an authoritative provider; a StateView is not a replay checkpoint by itself. BodyForce is borrowed per call and sampled on component faces. PressureWorkspace pressure is scratch; failed preparation can overwrite it while accepted state remains unchanged. Candidate pressure is not accepted-pressure provenance.

The optional ViscosityWorkspace is a filled-box free-slip operator. TriangleSurface barriers alter tracer transport, not velocity or pressure. StaticObstacleGeometry/StaticObstaclePressure and AlignedStrain consume caller-owned f64 arrays; they are not part of Simulation's accepted-state transaction. The column MAC bridge is a distinct flat-liquid representation, not an AMR obstacle flow owner. GridGeometry has one spacing per axis; no hierarchy, hanging faces, mortar interfaces or flux synchronization exists.

VelocitySampler is bounded convex interpolation of stored f32 component values. Upcasting to f64 exactly preserves those stored values; neither conversion nor additional queries produces independent physical observations. The read-only constructor audit builds actual Simulation objects at N3/6/12, observes initial rest state at time/generation0, and queries16,128 hypothetical first/second-layer positions per state. Every sampled velocity remains0. Simulation plus free-slip viscosity payloads are3,240;24,192;186,624 bytes. This is negative availability evidence for fresh rest states, not an evolved-flow or wall-load accuracy benchmark. No solve or timestep is executed.

## Options compared

|Approach|New information source|Required structures/consistency|Qualification limit|
|:--|:--|:--|:--|
|Upcast/current interpolation|None|Read-only accepted-state adapter|Cannot distinguish the preserved coarse fields or bound physical sample error|
|Offline refined patch, frozen coarse boundary|Specified PDE, forcing/time terms and boundary data, if independently available|Explicit artificial boundary lifts, coupled corners, local momentum residual and sensitivity to exterior data|Coarse interface/data error remains a floor; small local residual is not physical truth|
|Owned uniform f64 obstacle flow at N3/6/12|Authoritative initial/boundary/material/force problem, solved independently at each resolution|New accepted/candidate owner around retained geometry/operators; transaction and persistent data provenance|Most direct first step; no AMR interface, but current physical composition is absent and error bound still required|
|Persistent2:1 composite collar|Physical problem evaluated/evolved with actual fine unknowns|Leaf cells/subfaces, conservative transfers, composite pressure/strain, shared corner connectivity, synchronized accepted state|Production-feasible structure within16MB proposal; new numerical derivation/qualification required|
|Matrix-free full uniform refinement|Same physical problem solved on independently resolved full uniform state|New streamed K action matching the existing retained-row action and its adjoint/mass semantics|Could reduce row storage but is a new implementation, not permission to rerun the refused native fixture|
|Existing retained-row uniform N24|Same physical problem, if an authoritative owner exists|Existing row/lift payload plus actual-state/solve workspaces|Over current cap; larger resources alone do not make the physical experiment meaningful|

A current coarse snapshot alone cannot specify a unique finer physical instantaneous field. Complete initial/boundary/forcing history for a well-posed chosen model can restrict that ambiguity and enable a new resolved solve; this is an added physical problem specification, not a claim that the old snapshot has become informative. Start a hierarchy from genuinely specified initial data (e.g. the actual rest initialization), or import a independently resolved state with error/provenance. Refining an already coarse frame initializes unknowns but does not recover discarded history. Replay from the authoritative initial problem or carry its initialization uncertainty.

## Proposed bounded implementation phases

1. **Read-only state/provider contract.** Add a separate future research adapter carrying grid/obstacle/material stamps, physical time and generation, units, accepted-state checksum, accepted pressure/interval identity, specified initial/boundary data, and immutable force/source/transport history references. Record known uncertainties separately from missing ones. Never relabel scratch pressure as accepted pressure. Replay history is an external durable journal consumed with a64KiB bounded reader; the plan does not claim an unbounded journal fits in resident memory. Current read-only diagnostics implement only availability/type/count checks.
2. **Uniform owned obstacle state before AMR.** Design an isolated f64 accepted/candidate owner on retained static grid-aligned box geometry, maintaining original face-space and rho*A*d contracts. Specify the physical evolution equation, forcing and temporal scheme rather than silently composing pressure/viscosity into the production carrier. Proposed initial qualification uses N3/6/12 only, actual rest initial data and a specified nonanalytic physical force provider; choose a support with a buffer from obstacle/outer walls or explicitly handle nonsmooth data. BodyForce/SmokeSource models provide physical acceleration definitions, not velocities/loads. No old tilted-curl oracle supplies state or derivatives. All physical solves/time advancement need separate authorization because this task only permits analytical/read-only planning. Keep the original instantaneous ambiguous-field tests unchanged and separate from any new forced-flow problem.
3. **Error-control feasibility on uniform grids.** First show that the chosen evolution/space discretization supplies sufficient observation accuracy. Independently repeat from the same physical data on admitted grids, track temporal/data/rounding errors, and obtain a reliable bound for the load or the required observations. Cross-grid differences and Richardson rates are indicators unless their saturation/regularity premises are justified. The existing PCG residual/divergence and energy budgets bound their declared discrete diagnostics, not continuum velocity/traction error. Do not add AMR until this error-provider contract is reviewable.
4. **Composite collar topology and numerical operators.** Refine the whole collar described below, including edge/corner fluid connectivity, rather than six isolated wall strips. Use one authoritative open subface velocity, geometric per-cell flux balance and shared coarse/fine flux registers. Fill ghost/transfer values as initialization or boundary approximations, with declared error; fine PDE unknowns subsequently supply independent resolved state through the physical problem. Derive a composite momentum/pressure/strain discretization and appropriate mass/Hodge operator. Reject interfaces that fail consistency/adjoint/trace premises; do not reuse the sealed/free-slip AlignedStrain outer boundary as an artificial collar boundary. A corrected pressure gradient's transpose is not automatically the existing geometric divergence.
5. **Composite history and acceptance.** Initially use one global time interval across levels; avoid claiming subcycling conservation without derived reflux/projection synchronization. Hold accepted hierarchy immutable until all level/interface checks pass, and atomically publish state, geometry, pressure interval and provenance. Existing transport can cross the solid because its barrier is tracer-only; obstacle-aware momentum transport and its boundary/data error require qualification, not a zero-filled ghost shortcut. No present phase implements these changes.
6. **Wall observation extraction and honest stopping.** On a qualified hierarchy expose the actual fine tangential MAC samples at h/2 and3h/2 with represented distances and face/side identities. A cell-center value reconstructed from fine MAC values is a reconstruction of those fine unknowns, not an extra independently solved degree of freedom; charge interpolation error explicitly. Existing MAC wall cubature can avoid such extra reconstruction. Require a componentwise error/load budget and refine where the goal estimator identifies error, including corners/interfaces. Stop only against an unchanged physical load requirement plus demonstrated uncertainty; current ideal-sample percentages are not a new tolerance.

## Error contract: what can and cannot be controlled

With exact stationary trace, P1 value error amplification is2 epsilon/h and P2 is10 epsilon/(3h). Thus epsilon_h=o(h) is sufficient for value-error contributions to disappear, under the appropriate geometry/face regularity and quadrature premises. A genuinely second-order pointwise velocity bound O(h^2) gives at best an O(h) load guarantee from these differentiation bounds; it does not preserve generic P2 order2, which would require O(h^3) value accuracy. At fixed coarse spacing H, transfer or boundary error O(H^q) does not disappear as only local h shrinks. Higher donor count alone does not bound physical coarse error. Current semi-Lagrangian/pressure accuracy must be analyzed as a complete evolution; reducing dt alone is not a proved remedy.

For a future coercive local viscous resolvent A=M/dt+mu*K, A>=M/dt, residual r=b-A*x, and exact solution for the specified discrete data x*, the algebraic goal bound is

`|l^T(x*-x)| <= dt sqrt(sum(l_i^2/m_i) sum(r_i^2/m_i))`.

A sharper dual z satisfying A*z=l gives the exact conditional error identity `l^T(x*-x)=z^T r`. If the true chosen problem has additional data defect delta_b, the identity becomes `z^T r+z^T delta_b`; approximate dual residual adds another term. Artificial-interface uncertainty is part of delta_b through its actual boundary lift. Missing pressure/time/acceleration/advection/forcing data cannot be silently assigned0. Certified finite-arithmetic bounds would require outward-rounded residual/dual bounds; existing nearest-rounded budgets cannot certify them or authorize stepping.

This controls only algebraic error for a specified discrete model. A physical bound also needs spatial/model/geometry/initial/time/force/transfer/rounding budgets. A momentum residual without reliable bounds on those defects is insufficient. Pressure-free minimum energy and harmonic interpolation are model reconstructions, not physical measurements. Full pressure coupling/global incompressibility may make exterior influence nonlocal; no exponential scalar-diffusion attenuation bound is assumed for vector Stokes or pressure. Increasing collar depth is not itself an error certificate.

Actual cube flow has reentrant fluid edges/corners. Do not inherit the manufactured control's uniform C3 closed-face regularity or its torque cancellation. If such regularity is unavailable, use a suitable integrated load/weak-traction goal estimator with explicit momentum/interface terms and reliable stress/dual error bounds, or report uncertainty as unavailable. An H1 energy norm alone does not certify pointwise wall derivatives. Equilibrated/goal bounds would need their own derivation for the selected PDE and corners; this plan does not assert one already exists.

## Exact2:1 collar and consistency diagnostic

Base grid H=1/4,N12 over[0,3]^3 with solid[1,2]^3. Refine all448 fluid parents inside[1/2,5/2]^3 by2; retain1216 exterior coarse fluid cells. The composite has3584 fine fluid cells and4800 fluid leaves. The0.5 physical collar margin contains both P2 layers at h=1/8; it does not shrink the artificial boundary onto an unresolved normal sample. Fine wall/corner regions are uniform; exterior coarse interfaces are a separate operator derivation.

The collar interface has384 coarse faces and1536 fine subfaces. Outer boundary has864 coarse faces, solid384 fine faces, open face count14352, total faces15600, cell-face incidences29952. Count identity: `2*F+B=6*C+384*(4-1)`. Exact integer occupancy enumeration independently streams these counts with a55,296-byte occupancy buffer; it neither assembles nor solves a native refined grid.

A coarse/fine normal center distance is3/16 while a child's tangential center is offset1/16. For affine pressure p=y, a naive two-point normal gradient gives±1/3 although the true normal gradient is0. A centered tangential coarse trace correction fixes that affine test but introduces a remote coarse pressure coefficient±2/3 into the acceleration-gradient operator. With Aface=1/64,rho=1,m=rho*A*d=3/1024, its M-adjoint produces a remote integrated-divergence/work coefficient±1/512. That is not the physical incidence of this face into its two neighboring cells. Taking the corrected transpose redistributes flux to other coarse cells. Keeping old geometric incidence and diagonal rho*A*d fixes its matched gradient and retains the affine defect.

Therefore consistency, geometric incidence and old diagonal face mass cannot simply all be claimed for this naive hanging-face construction. A new derived mortar/mixed/nonlocal-Hodge treatment, additional pressure/velocity moments or a different explicitly justified representation is required. Original uniform PR28 pressure/shear/data remain unchanged; no such new pressure operator or solve is implemented here.

## Managed resource proposal

Rust1.92/64-bit observed proposed record sizes: LeafCell24B, Face64B, eight-donor Transfer96B,27-donor Transfer328B, TileRow1040B. Transfers use u32 donor IDs (prove IDs fit; reserved sentinel never indexes arrays), f64 weights and implicit target order. Padding is included by actual sizeof. Transfer target/type/count mapping must be explicit in topology/index order; these are layouts, not qualified interpolation rules. All admitted arrays must check actual Vec capacities and total live payload before publication.

Outer two-cell ghost reserve:3904 cell targets and12144 face targets (padded fine patch20^3 minus16^3). Conservative inner reserve:512 solid cell targets+1728 solid face targets. All18288 targets get a transfer row and f64 value in the conservative budget, although stationary traces may later be eliminated only with represented-distance and corner proofs. No unaccounted solid-side ghost shortcut is assumed.

|Common live allocation|Bytes|
|:--|--:|
|4800 LeafCell records|115200|
|15600 Face records|998400|
|8 f64 vectors on14352 open faces|918528|
|7 f64 pressure vectors on4800 cells|268800|
|2 f32 tracer vectors|38400|
|u32 cell-face CSR offsets/incidences|139012|
|Coexisting N12 reference operators/geometry/pressure and filled-box owner+viscosity|5454440|
|384 interface parent/4-child maps|7680|
|384 four-component flux registers|12288|
|448 parent/8-child tables|14336|
|2 f64 goal-dual face vectors|229632|
|Bounded128-row tile,64 terms each|133120|
|864 wall observation rows,96B each|82944|
|Provenance reader and stream buffer|73728|
|Ghost rows+values,8 donors|1901952|
|**8-donor total**|**10388460**|
|Ghost rows+values,27 donors (replaces8-donor line)|6144768|
|**27-donor total**|**14631276**|

The optional N12 reference owners are counted even though they represent different physical models and are not coupled. Ghost donor counts define hard proposal capacities, not accuracy claims. The bounded tile admits at most128 transient rows with at most64 terms each; the derivation must prove every actual sector/interface stencil fits, otherwise admission fails without dropping rows. No global retained strain matrix is included: a new matrix-free gather/transpose action would have to regenerate the same qualified local forms. Solver/dual arrays can be reused by declared lifetimes but the table does not rely on an unimplemented lifetime optimization. Any additional mortar/Hodge/regularity-estimator storage must fit the remaining1,368,724B for the27-donor variant or trigger a new reviewed proposal. Process RSS, allocator overhead, fixed stack and external durable journal size are outside this managed-payload contract, as in the original cap. This is not an assertion that all unknown future AMR algorithms fit16MB.

For comparison only, keeping the existing retained rows/lifts at uniform N24 requires43,321,344B by itself. The following hypothetical future experiment budget is not a current admission or run:

|Uniform N24 item|Bytes|
|:--|--:|
|Existing aligned operator+lifts|43321344|
|Geometry retained payload|567264|
|Existing static pressure workspace|1119752|
|Proposed2 full-face f64 state arrays|691200|
|Proposed8 active-face f64 viscous CG arrays|2433024|
|864 observation rows|82944|
|Provenance/stream buffers|73728|
|**Declared total**|**48289256**|
|Hypothetical managed cap|50000000|
|Remaining envelope|1710744|

This total includes explicitly proposed solver/state arrays not currently implemented. Geometry constructor queue is freed before coexistence (peak677856B); actual allocation capacities and any later goal-estimator storage require preflight. A cap request would be meaningful only after a bounded physical case, real state provider, numerical model/error goal and reviewed live-array design exist. Since the composite proposal can fit the current cap and those prerequisites are missing, **no larger cap is requested or justified now**. No uniform N24 native allocation is performed, nor is matrix-free representation used to bypass that refusal.

## Research basis and evidence

[Almgren, Bell, Colella, Howell and Welcome, conservative adaptive projection](https://escholarship.org/content/qt0xg2k57t/qt0xg2k57t_noSplash_204a60cebf6e13eacfaf9626f5beee68.pdf), sections3.2–3.6, treats coarse/fine flux and projection mismatch with synchronization. It does not justify treating coarse interpolation as new observations or certify this solver's wall load. [Losasso, Gibou and Fedkiw, octree water/smoke](https://physbam.stanford.edu/papers/stanford2004-02.pdf), sections3–4, derives an adaptive pressure representation; its stated low-order advection/pressure accuracy is not the required near-wall error bound. These primary sources motivate interface derivation rather than a verbatim AMR stencil claim. The finite counts, affine obstruction, diagonal-mass adjoint calculation and plan here are newly derived.

Source examples/research_solver_wall_state_audit.rs calls constructors and immutable state/sampling only. tools/research_solver_wall_state_plan.py compares actual Rust record layouts/state bytes, enumerates topology, calculates all budget lines and checks the exact affine/adjoint obstruction. Outputs go to a new external directory. Independent review separately inspects source, enumerates topology and checks pressure work/availability. Fresh source/binary/record hashes are captured in diagnostic receipts. Existing4a268ef/10fcda3 and their evidence/capsules remain preserved; no previous oracle output is loaded by these new diagnostics.

**Precise remaining blocker:** an authoritative accepted f64 no-slip obstacle state for a specified physical problem, its initialization/forcing/pressure/time provenance, and a demonstrated error provider sufficient for wall loads. Following that, composite interface consistency/mass/flux/strain and reliable corner/load estimation need qualification. Memory is currently a plan constraint, not the reason the simulator lacks the information. This task provides a reviewed implementation sequence and exact resource proposal; it does not claim a production-capable state/error provider has been implemented.
