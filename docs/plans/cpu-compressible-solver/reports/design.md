# Numerical design, authority and validation boundaries

## Inspected source and standards route

Published Rheon base is `8baf459b91bb53ebd4a7230c4d9f3399dd5cb0bc`.
The clean saved checkout was `6ba41c7`; fetch plus a new branch from main preserved
both prior refs and all workspace/tooling files. No AGENTS.md was found at the
workspace/ancestor locations or either Rheon tree; no repository-specific agent
skill exists. Applicable repository rules are [output policy](../../../REPOSITORY_OUTPUTS.md),
Cargo lints/locks, `rust-toolchain.toml`, workflow gates and the shipped API docs.

Source standards are [MrScripty/Coding-Standards 188beda1](https://github.com/MrScripty/Coding-Standards/tree/188beda1fa477d21d576c233dd8c7f4c4c267d23).
Read Core and Router, Planning (including required Implementation, Verification
and Development Proportionality), Commit, Documentation, Architecture, Code
Design, Contracts/evolution, Dependencies, Licensing, Performance and independent
Oracles. Persistence and Library profiles inform the proposed contracts. Current
edits are Markdown only: no Rust-language mechanism or build/release change.
Implementation admission must route Rust/API/tooling and the actually affected
UI/accessibility, concurrency, resilience, diagnostics, persistence and generated
contract details anew. No Standards Engine tool is exposed; source-guided review
is not an automated engine certificate. No standards-authoring skill applies.

Research authority: [Ch26](../../../research-book/chapters/26-compressible-state-and-waves.md),
[Ch27](../../../research-book/chapters/27-compressible-discretization.md),
[Ch28](../../../research-book/chapters/28-compressible-boundaries-and-validation.md),
[research roadmap](../../../research-book/implementation/compressible-flow-roadmap.md),
[reference scope](../../../research-book/compressible/README.md),
[inspected source ledger](../../../research-book/sources/compressible-sources.json),
and [Lean source](../../../../proofs/Rheon/Compressible.lean).
This plan uses their stated equations and qualification limits. Metadata-only
paper entries do not establish uninspected theorem assumptions. Read the exact
primary theorem before making any later invariant-domain claim.

`src/simulation.rs` evolves fixed-density staggered velocity and appearance;
`src/liquid_volume.rs` evolves represented volume, not total thermal energy.
`src/particle_fluid.rs` is a bounded material/SPH model. `src/solver_provider.rs`
owns sealed MAC pressure-stage requests; `src/session.rs` owns existing bounded
sessions. `src/static_obstacle.rs`, `src/obstacle_pressure.rs`,
`src/obstacle_shear.rs` and `experiments/plate_coupling.rs` retain distinct reduced
geometry/boundary scopes. `GridGeometry` validates Cartesian dimensions/world
coordinates and uses x-fast indexing; reuse that metadata only, not MAC layouts.
The Python reference instead stores `(nx,ny,nz,5)` with z-fast spatial flattening.
Tests must explicitly transpose, using asymmetric shapes to detect axis errors.

## Conservative state and EOS

Store cell averages `U=(rho,mx,my,mz,E)` in binary64, x-fast; SI units are
kg/m³, kg/(m² s), J/m³. `E` is total energy density; momentum is not velocity.
Initial data must identify whether it supplies exact conservative cell averages
or center samples. Smooth-reference acceptance uses quadrature with a smaller
error than the intended numerical budget. Appearance/tracers cannot replace a
component of U. State/scratch allocation uses checked counts and actual retained
Vec capacities, fallible reservations and a caller-owned payload cap; external
buffers, allocator overhead, recorder/UI/model and RSS are separately reported.

S1 closure: constant `gamma>1`, positive gas constant R for SI temperature,
`cv=R/(gamma-1)`, `p=(gamma-1)*(E-|m|²/(2rho))`,
`T=p/(rho R)`, `c²=gamma*p/rho`. Validate all parameters, finite intermediate
results, `rho>0`, internal energy density `>0`, `p>0`, `T>0` and `c²>0`.
Absolute EOS pressure has no arbitrary incompressible gauge. Floors are not
physical repairs. Reject overflow, underflow/lost internal energy and invalid
thermodynamics with cell/stage/cause; retain accepted state/time/report.
High-Mach subtraction and low-Mach cancellation require dedicated conditioning
evidence; f64 alone supplies no unrestricted regime guarantee. Dual-energy
policies require a separate authoritative-equation and conservation decision.

The numerical owner maintains this deliberately finite ideal-gas/periodic/LLF
scope and its independent native gates; owning a broader standardized physics
framework is not selected. Adoption-versus-ownership comparison:

| Candidate | Observed fit and mismatch against S1 | Decision and maintenance consequence |
| --- | --- | --- |
| Existing Rheon projection core | Native bounded transactional patterns and grid metadata; no evolving thermodynamic energy or acoustic flux | Reuse metadata/ownership patterns only; EOS/flux invariants cannot be inherited |
| Existing original Python Euler reference at 8baf459 | Same finite educational equations and useful translation evidence; Python/NumPy array lifecycle does not provide native Rust borrowed views, retained payload accounting or cooperative atomic publication | Keep as differential reference; an adapter would retain two runtime/state owners and still require acceptance/resource machinery |
| Clawpack 5.14.x documented Classic/PyClaw solvers | Official docs describe 1/2/3D Classic algorithms and Python/Fortran execution with supplied Riemann solvers; official software terms are BSD with notice/nonendorsement obligations. Native safe-Rust capped allocation/rollback/cancellation is not a documented contract of this candidate | Do not adopt in S1: it adds a foreign/runtime boundary and an independent solver lifecycle while the product-specific acceptance owner is still necessary; no claim that Clawpack lacks accuracy or cannot be adapted |
| Original finite Rust Euler owner | Can contain the five-field update and product-specific finite/admissible/capped transactional invariant in one owner, with analytic and separately maintained translation evidence | Select for S1 only. Numerical owner maintains equations/assumptions and negatives; independent validator owns accuracy/origin harness. This decision does not grant an unrestricted positivity, physical-validation or high-order physics claim |

Candidate facts were checked read-only on 2026-10-14 against official
[Clawpack license](https://www.clawpack.org/license.html),
[installation](https://www.clawpack.org/installing.html) and
[solver documentation](https://www.clawpack.org/pyclaw/solvers.html).
No Clawpack package, source or notice text is incorporated or installed; no
transitive-license or Rust-embedding qualification is claimed. The comparison
selects ownership of a bounded product contract, not a fallback from unavailable
candidate facts. Maintenance cost is kept local by excluding general EOS,
implicit, high-order, moving-mesh and closure features. Revisit this decision
when that semantic surface expands or a proven embedded candidate satisfies
the same native ownership/resource contract. Such expansion requires a real
dependency comparison and its own license/consumer evidence.
Later EOS support must define `p(rho,e)`, temperature/inversion and isentropic
derivatives/domain/ranges. Stiffened gas needs consistent offsets/caloric law
and `p+p_inf>0`; Tait is barotropic and does not supply shock heating. Neither is
a shortcut to calibrated submarine water, cavitation or phase transitions.

## Flux, reconstruction, time and admissibility

Normal physical flux is `(rho un, m un+p n, (E+p)un)`.
S1 uses one shared interior/periodic LLF face value
`0.5*(F_L+F_R)-0.5*alpha*(U_R-U_L)` with endpoint
`alpha=max(|un,L|+cL,|un,R|+cR)`, opposite updates weighted by area/volume.
Every momentum component evolves on every axis. A serial unique-face loop
avoids races; verify periodic wrap pairing, including one/two-cell dimensions.
Either a bounded face strip or direct paired accumulation may be selected after
measuring scratch cost; no full face tensor or per-step allocation is required.

Use an unsplit sum of three directional divergences, piecewise constants and
forward Euler. For this shared-face method define an engineering step bound
`dt <= C * min_i(V_i / sum_faces(A_f alpha_f))`, taking the actual incident
speeds (periodic faces included), initially `C=0.5` maximum. On uniform speeds
this equals the reference's `0.25/sum_axes((abs(u)+c)/dx)`.
This is a declared cautious baseline, not a LLF positivity theorem. If varying
face speeds make it stricter than the research bound, differential tests compare
one step at the same explicit dt and document the scheduling difference.

Actual dt is min(requested, CFL bound, remaining time). Reject nonfinite/nonpositive
requests and any increment that cannot advance the represented clock. Inspect
the full candidate before publication. Retry only a candidate admissibility or
arithmetic failure that step reduction can address: halve from unchanged
accepted state, at most 8 retries, recording attempts and cause; invalid input,
allocation, EOS parameters and cancellation never retry. No state or clock
change on exhausted rejection. Hit output/final times using actual accepted
increments; no successful zero step or rounded-clock overwrite hides failure.
All stages of later integrators must satisfy the same publication contract.

S2: compare conservative limited linear reconstruction with primitive or
characteristic alternatives; choose one declared variable set. Preserve the
cell average, then use admissible convex scaling at all required quadrature/face
points and SSPRK2 stage checks. Convexity alone cannot validate an already
inadmissible mean or the flux/CFL assumptions. HLLE with inspected wave bounds
is the next robust comparison; HLLC contact resolution requires denominator,
star-state, rarefaction and perturbed strong-shock evidence before default use.
Fallback/limiting selects one shared face value, never a different value per
cell. Record every limiter/fallback/retry, its affected content and timestep.
Roe, WENO and all-speed remedies remain optional later design decisions.

Conservation, positivity, entropy and convergence are independent. Entropy
diagnostics use physical `s=cv*log((p/p_ref)*(rho_ref/rho)^gamma)` and report
whether global/cellwise, fully/semi-discrete and which boundary entropy flux
is used. A measured global entropy increase does not prove entropy stability.
Low-Mach explicit acoustics retains its stiffness and LLF damping; no modified
sound speed is used to manufacture a speed improvement.

## Boundaries, geometry and vehicle paths

S1 is periodic. S3 first adds exact fixed endpoint states for the isolated Sod
tube, then explicit characteristic/freestream data and stationary fitted slip
walls. With outward normal and `a=(u-vb)·n`, incoming counts are 5 supersonic
inflow, 4 subsonic inflow, 1 subsonic outflow, 0 supersonic outflow. Sonic points,
backflow and switching regimes must have defined outcomes. Prescribed full
primitive state at every subsonic boundary is not generally admissible.
Outlet reflection claims need incidence/frequency/Mach/error measurements.
Slip wall mass/energy fluxes are zero, momentum is `pw n`; ghost reflection
recomputes total energy. Slip is different from viscous no-slip/thermal data.

General geometry follow-on requires one versioned owner for fluid volumes,
open areas, oriented wall patches, connectivity, quadrature and geometry closure
`sum A n=0`. Reuse imported CAD as source identity only after actual topology
assembly; collision hits do not supply cut volumes. Small cells require their
actual `V/sum A alpha`, minimum fraction/control cell diagnostics and separately
qualified conservative stabilization. No volume clamp hides stability costs.

Moving geometry uses conservative contents `V U` and ALE `F_n-vb,n U`, matched
swept face/volume quadrature and discrete GCL. Wall energy flux is `pw vw,n`
outward from fluid; inward piston motion adds fluid energy. Include equal/opposite
force, torque and work, or label prescribed motion as external actuator work.
Conservative covered/uncovered-cell transfers and topology events publish
together with geometry/state/time; failures preserve the old coherent state.
Moving-grid freestream and physical piston compression are different tests.

| Intended case | Physical route and elementary prerequisites | Promotion evidence and exclusions |
| --- | --- | --- |
| Passenger jet in flight | Compressible external aerodynamics with specified Mach, altitude atmosphere, angle of attack, Reynolds, thermal law and vehicle/reference frame. Uniform/acoustics/Sod/vortex → oblique wedge → laminar flat plate → airfoil → wing → complete aircraft. A jet aircraft is not the desktop emitter jet. | Independent force/pressure/heat benchmarks, grid/time/domain refinement, correct farfield data, viscosity and validated turbulence/wall regime. Inviscid Euler can support only its declared inviscid questions; no aircraft readiness from the lab. High temperature/engine chemistry/propulsion are separate exclusions. |
| Submerged submarine moving forward while increasing elevation | Choose coordinates: z upward; prescribe `(Ux,0,Uz>0)` and a fully submerged trajectory, or an equivalent body frame with correct relative flow and inertial terms. Begin with constant-density incompressible viscous regime if `U/c` is small. Hydrostatic `dp/dz=-rho*g`, Couette/Poiseuille, stationary body flow → prescribed translating body → combined forward/rising motion. | General conservative moving volume/no penetration/no slip, pressure+shear loads, displaced-fluid/reaction/work, hydrostatic balance, GCL and topology transfers. Validate stationary drag/wake then rising-body loads/trajectory against licensed experiments at matched Reynolds/domain/depth. Free surface, cavitation, compressible water shocks, propulsor and 6-DOF feedback require separately selected models. Current reduced incompressible components are not a validated submarine solver. |

## Viscosity and turbulence staging

S6 adds conservative Newtonian stress
`tau=2mu(D-div(u)I/3)+zeta div(u)I`, `q=-k grad(T)` with declared nonnegative
transport laws and EOS range. Momentum and energy share stress flux, energy
includes `tau u-q`; velocity diffusion alone loses thermodynamic conversion.
Boundary data select no-slip/slip and adiabatic/prescribed-T/heat law. Restrict
explicit dt with the actual multidimensional shear/bulk/conduction operator;
simple `h²/nu` and `h²/chi` estimates alone are not its complete bound. Any
implicit solve adds actual stored-field residual, nonlinear/domain acceptance,
failure rollback and dependency review. Verify conduction, kinetic-to-internal
conversion, Couette/Poiseuille and compatible manufactured solutions before
flat-plate comparisons. NASA's low-Mach Blasius comparison is approximate at
finite Mach; high-Mach wall heating needs a matched compressible thermal oracle.

S7 selects a regime and closure, not a decorative perturbation. DNS needs resolved
dissipative scales; LES needs filter/subgrid stress and heat-flux conventions;
RANS needs specific closure variant, extra equations, coefficients, turbulent
Prandtl, inflow/transition and wall treatment. Record full y+ distribution,
spacing/layers, sampling intervals and statistical/model-form uncertainty.
Keep calibration and validation data independent. Frozen closure validation
must precede vehicle claims; SGS pressure preconditioning and seeded emitter
noise are not validated turbulence physics.

## Storage, API and UI contracts

PR69 ([b1acf2e](https://github.com/MrScripty/Rheon/pull/69)) adds generic SQLite
schema-v1 accepted-publication recording and material adaptation. PR71
([ecfc741](https://github.com/MrScripty/Rheon/pull/71)) stacks on PR69, adding
carrier adaptation/cold inspection. Their bases are PR67 → PR69 → PR71; they
remain open and are not present on inspected main. Read the actual integrated
`docs/api/recording.md` and storage-decision report again before S4.

Retain indexed metadata + one binary BLOB per named field/timestep; CSV is export
only. Define a compressible adapter/decoder with required native rho/m/E,
derived p/u/T/c/Mach, cell association, scalar widths, x-fast layout, SI units
and evaluation times. Native U is authoritative; derived fields must identify
derivation/EOS/stage and never masquerade as independent native measurements.
Archive integrity/transaction limits belong to recording, EOS validation to
numerical decoding. Reject unsupported models/versions and inconsistent fields.
Snapshots borrow immutable accepted state; retention/copying is consumer-owned.
No incomplete scratch or rejected attempt becomes a step ordinal.

Append after physics acceptance, before corresponding preview. Recording failure
does not undo physics: stop capture, expose accepted vs durable tail and uncertain
outcome, reconcile with fresh reader before completion. Keep constructor distinct,
actual dt/report/controls bound, missing data absent, selection exact and no
latest-value fallback. Archives are inspection/interchange, not restart checkpoints.
Keep-all/stop limits include main DB, record and count; budget rollback journals
and concurrent decoded/raw/solver/UI buffers separately. Compressible five-f64
state alone is `40*N` bytes per step, before derived fields/metadata; test requested
retention against real byte costs. Compression/chunking requires its own measured
decision and persisted-consumer evolution; no silent lossy quantization.

New session has one numerical owner, validated controls, typed errors,
cooperative cancellation, actual accepted increments and explicit capability
metadata. No existing pressure-session alias pretends to solve Euler. Desktop
exposes model/EOS/method/domain limits, compute/playback distinction, stop/failure,
selected field/unit/time, range/probe/export and rejected/fallback counts. Test
the real user path and native display, not only widget or screenshot construction.
Generic Surrogate Lab generation/data controls include producer/model, case/seed,
grid/EOS/boundaries/time, output field/association, quota/capture cadence,
validation/split selection, and results/timing/fallback. Unsupported controls
are explicit. Persist project choices through the consumer's actual schema;
do not invent a second Tuldok project format inside Rheon.

## Reproducible datasets and model evaluation

The producer records actual accepted conservative fields, actual interval controls
and timestamps, geometry/EOS/transport/model identities, source tree and binary
hash, toolchain/lock/runtime, seeds, numerical method, boundary schedules,
resource limits, reference/error policy, trajectory and source-family identity.
Opaque Debug strings in pending archives are provenance, not replayable config.
Complete configuration codecs and consumer agreement are S5 prerequisites.
Generation/validation/completion are separate: partial trajectories remain
incomplete; durable completion requires verified field/control hashes, count,
ordering, semantics and output publication. Reuse existing publication mechanisms
only where their contract actually fits.

Existing [dense3D controls](../../../implementation/dense3d-producer-controls.md)
are fixed eight-interval transport capture and version-specific consumers;
arbitrary compressible data cannot be relabeled as v2. Consumer/schema ownership,
versions, unsupported rejection, units/layout, family graph and normalization
must be agreed with a verified Tuldok revision. Preserve existing v1/v2 readers;
no dual-write shim or guessed schema is admitted by this plan.

Split whole trajectories and correlated source families before fitting anything.
Group repeated seeds, shared initial/geometry/source variants, refinements and
resampled/restarted copies according to actual lineage; no frame leakage and no
deleted/unused bridge breaks a connected family. Fit normalization on training
only; freeze train/validation/untouched test manifests and checksums. Hold out
declared parameter/regime/geometry bands; a previously used seed is not untouched.
Record license/provenance/retention for source data and checkpoints.

Model evaluation covers one-step and whole-rollout rho/m/E/p/u errors, field
norms, conservation/positivity, shock location, long-horizon drift, unsupported
domain and real invalid-output recovery. A learned compressible model needs its
own same-domain acceptance gates and typed request, separately from pressure
providers. First prove the real native elementary producer→consumer path; no
training campaign or learned capability is part of S1 or this plan task.

End-to-end CPU timing includes cold model load, input conversion, inference,
candidate qualification, rejection, numerical fallback/retries, publication,
recording and selected export where claimed. Also report warm phase timings,
fallback frequency, complete process/run latency, actual simulated seconds,
payload/disk and measured RSS separately. Compare equal grids/physical error
and horizons with numerical references; rotate order/repeat, record distributions
and uncertainty, host/threads/storage/cache/build/runtime and failure cases.
Inference-only timing is not end-to-end speedup; fallback can erase it. Without
an agreed representative workload/budget, report observations rather than a
performance pass or aircraft real-time claim.

## Independent verification, exact source and proof limits

| Claim/domain | Deciding oracle independent of subject | Unsupported domain / negative discriminator |
| --- | --- | --- |
| Uniform/flux/orientation/shared-face algebra | independently derived physical flux and volume ledger; all axes/asymmetric counts | not continuum accuracy; wrong pressure energy term and unmatched face sign must fail |
| Planar Sod and pressure forces | ideal-gas exact fan with bracketed root; integrate conservative averages across waves | not turbulence/3D; periodic central t<=0.08 versus isolated t=0.2 must not share endpoint budgets |
| Small acoustic propagation | continuum linear solution with amplitude sweep and quadrature; separate nonlinear O(epsilon²) error | not finite-amplitude exact Euler; frozen fields must fail evolved phase check |
| Smooth vortex/order | Ch28 analytic exterior solution, independent cell-average quadrature, separate temporal refinement | extrusion not full 3D; wrapped x/y Gaussian seam is not exact periodicity |
| Boundary/geometry/work | characteristic count and long-domain reflection comparison; independent volume/area/GCL/force/piston identities | not arbitrary topology; moving-wall missing work and stale geometry must fail |
| Recording/consumer | separate-process real reader and independent native-field bits/units/times checks; real Tuldok parser | hashes not authenticity or accuracy; wrong stage/layout/controls/unsupported version must fail specifically |
| Physical validation | independently licensed experiments with matched regime and uncertainty | analytic references/Lean/replay not experiments; retuned calibration is not validation |

Accuracy and origin are distinct: analytic references can reject frozen wrong
phase/fields, but cannot detect arbitrary copying of correct same-case fields.
Fresh pinned native execution and independently bound run/artifact identities
decide source/control origin. Hashes or a self-consistent manifest alone do not.

Freeze thresholds before qualification: derive dimensional absolute/relative
allowances from arithmetic, reference quadrature/root/linearization uncertainty
and discretization error; retain grid/time curves, not a single favorable norm.
S1 uses the concrete initial gates in its handoff. Later method/domain upgrades
cannot silently lower gates or switch references to obtain a pass. Adversarial
fixtures change one condition from a valid run and assert the intended diagnostic;
generic rejection, parsing and copied current results are insufficient oracles.

Each implementation acceptance execution uses a clean committed candidate and
records commit/tree, exact relevant source/locks/pins, executable hash, build
command/toolchain/runtime/hardware, controls, generated artifact hashes and
commands/results. A clean source identity establishes origin, not numerical
correctness. Differential agreement with the Python LLF baseline supplies
translation evidence only; analytic/experimental authorities decide accuracy.
Rerun for material subject changes; retain failures and unsupported cases.

Keep `proofs/scripts/check_sources.py`, kernel build and allowed-axiom audit
separate. Changed Lean sources require inventory/audit review and fresh pinned
Lean 4.19.0/mathlib qualification. S1 changes no proof or pin. Existing book
native-source preflight intentionally rejects unadmitted native changes; only
the book integration owner reviews a new full inventory and obtains fresh
native/rational/browser/PDF/publication qualification. Do not refresh hashes,
weaken gates or carry old research receipts onto a new solver tree.

| Lean declaration | Exact assumptions / actual meaning | Excluded inference |
| --- | --- | --- |
| `shared_face_conservation` | finite exact-real amounts/flux, incidence columns sum to zero, one shared extensive flux, any dt | does not prove mesh incidence assembly, floating sums, EOS/flux accuracy or external-boundary budgets |
| `ideal_gas_pressure_positive`, `admissible_iff_pressure_positive` | gamma>1; rho>0 and exact positive internal density | not finite arithmetic, EOS calibration or energy conditioning |
| `admissible_blend` | both endpoints admissible; exact-real convex coefficient in [0,1]; finite signed momentum | not LLF/HLLC positivity, reconstruction/quadrature/CFL or admissible means |
| `cfl_density_positive` / `nonnegative` | donor-density definition; nonnegative donors/coefficients, positive rho and sum<1 (or nonnegative rho/sum<=1) | not a Rusanov Euler update; no total-energy/internal-energy/entropy proof |
| `stationary_slip_wall_flux` | exact zero normal momentum under the stated flux definitions | not numerical wall pressure, no-slip, moving work, watertight geometry or loads validation |

Formal assumptions remain visible in API/evidence descriptions. Passing Lean
and native tests supplies separate scoped evidence; neither stands in for
empirical solver validation.
