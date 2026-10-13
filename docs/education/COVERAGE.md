# Research-book interactive coverage

This ledger covers all 25 chapters and six appendices. The sampling/transport
follow-on expands the [540bdef teaching checkpoint](https://github.com/MrScripty/Rheon/commit/540bdef8fcbc0d0a55567325eba570589666bf80)
from 11 to 13 chapters, then the bounded browser timestep adds chapter 17 and the computed volume adds chapter 14.
There are now **18 chapters with partial scroll-linked examples**: 3–10, 12, 14, 17 and 19–25.
No appendix has a scroll-linked example. These counts describe placement,
not complete coverage of each chapter's topics or qualification of a solver.
[Heading bindings](teaching.py) fail the build when an expected section moves.

The [native timestep/computed-flow screenshots](screenshots/README.md#native-timestep-and-computed-flow)
use clean commit `db9781a1a26e43bd73fb08995fdc855d42978c12`. Their
[provenance](screenshots/research-book-native-flow/provenance.json) identifies
the actual computed state and complete native/WASM comparison. The compact
desktop angles show the same fields and time. Screenshots document the UI;
they do not replace numerical qualification.

The [current sampling/transport screenshots](screenshots/README.md#current-sampling-and-transport-lessons)
show the two recomputed lessons from clean local capture commit
`ae06472f96d568d2f7077561e7b0080763510366`, source tree
`4f96d068eb780fa22b6eca2481504596d8db75c0`. Their
[provenance](screenshots/research-book-transport/provenance.json) records the
capture source separately from the
[published implementation](https://github.com/MrScripty/Rheon/commit/1c14e5787420b4db305abed96863e15535ca8e21),
whose source tree is identical and commit metadata differs.
The [historical six-reference screenshots](screenshots/README.md#historical-six-reference-checkpoint)
retain source `540bdef8fcbc0d0a55567325eba570589666bf80` and do not demonstrate
the subsequent computations. PR42's native-app captures remain separate and
unchanged. Screenshots document the UI; numerical comparisons supply the
computation evidence.

## What executes

| Source kind | Meaning and source |
| --- | --- |
| Stored reference | The original six lessons select analytical or dense-solve records from [reference.py](../research-book/expansion/reference.py). [labs.js](labs.js) recomputes the labelled projection blend and readouts; it does not rerun the pressure solve. |
| Browser mechanism | [transport_reference.js](transport_reference.js) recomputes bounded affine sampling and periodic 1D scalar transport. Its references are the existing [sampling fixture](../research-book/companion/depth_experiments.py) and [transport formulas](../research-book/companion/experiments.py). It is an independent teaching implementation. |
| Browser native timestep | A separately compiled [Rust adapter](../../browser/solver/src/lib.rs) calls the existing `Simulation::step_with_forces` in a Web Worker. The single accepted interval is the fixed four-edge force/projection fixture. A current compiled packet is required; absent packets disable the controls. |
| Browser 3D flow | [flow.rs](../../browser/solver/src/flow.rs) advances a bounded 24³ constant-density appearance-tracer fixture using the existing native timestep. [flow_volume.js](flow_volume.js) renders the complete accepted tensor with perspective ray integration and fixed opacity. Source pulse: 30 steps; run limit: 80. |
| Browser operator | [aligned_strain.js](aligned_strain.js) applies immutable native strain rows and their transpose to chosen face samples. It recomputes force and loss, without a timestep or pressure solve. A qualified packet is required. |
| Native recording | Wall/force, static geometry, obstacle pressure and reduced-shear pages display admitted native packets. Browser controls choose saved cases, slices or times. Missing packets remain unavailable. |
| Text/fixture | Derivation, source links, proof disclosures or offline numerical fixtures; no corresponding browser computation is supplied. |

The [shared viewer](../../browser/viewer/README.md) supplies rendering, camera
controls and theme. The [timestep adapter audit](BROWSER_TIMESTEP.md) separates
build plumbing from missing physics. Native [Simulation::step](../../src/simulation.rs)
and [LiquidTransportSimulation::step](../../src/liquid_step.rs) already existed.
The WASM binding executes the fixed force cycle and a separate bounded 3D source-pulse sequence through `Simulation::step_with_forces`. It does not expose general grid configuration or liquid stepping.
Browser scalar transport and finite-row evaluation remain separate teaching
mechanisms and do not invoke those native APIs.

## Current native material and scheduler teaching

Chapters 11, 13, 16 and 17 now include source-linked material-pair/nozzle,
accepted-state, preview/history and CPU scheduling exercises. These additions
use the actual native APIs, independent numerical tests and bounded Lean
statements. They do not add a browser SPH runtime or additional scroll-linked
chapters: coverage remains 15/25 partial chapters and 0/6 appendices.
[Material contracts](../research-book/implementation/material-preview-contracts.md)
and [CPU scheduling contracts](../research-book/implementation/cpu-scheduling-contracts.md)
separate exact arithmetic from implementation, full-step stability, continuum,
wall-clock fairness and performance claims. Exact current kernel qualification
status is retained separately from historical receipts in `native-sequence.json`.

## Chapter and appendix matrix

“Separate” means a linked laboratory outside the chapter's sticky workspace.
Every native recording/operator entry is conditional on packet admission.

| Chapter | Source kind and scoped support | Remaining topics |
| --- | --- | --- |
| 01 Numerical contract | Text/fixture; evidence and scope distinctions | Accepted-state lifecycle, end-to-end guidance error and performance |
| 02 Continuum model | Text/fixture; equations and units | Interactive nondimensionalization, splitting and boundary-work comparison |
| 03 Staggered geometry | Stored projection; browser affine sampler at the layout discussion | Weighted metrics, machine indexing and general geometry assembly |
| 04 Pressure correction | Stored fields and recomputed blend | Boundary assembly, component gauges, compatibility and a live pressure solve |
| 05 Linear solvers | Stored projection as context only | Jacobi/CG iteration histories, preconditioning and multigrid; correction fraction is not an iteration count |
| 06 Transport | Browser sampler and periodic scalar transport | Variable velocity, collision-aware traces, open/closed sampling policies and conservative liquid flux |
| 07 Corrected transport | Browser baseline/raw/limited comparison and resolution controls | Native corrected advection, invalid-trace fallback, self-advected velocity and general conservation |
| 08 Boundaries | Text/fixture; separate native static geometry/pressure records | General mesh boundaries, weighted affine-adjoint demonstration and moving geometry |
| 09 Liquid interfaces | Browser recomputed one-face axis-aligned swept-volume geometry; offline curvature fixtures | General fraction transport, arbitrary-normal reconstruction, reinitialization and surface-error comparisons |
| 10 Forces/viscosity | Stored hydrostatics and scalar shear; separate native wall/force records | Variable/nonlinear tensor stress and dynamic solid coupling |
| 11 Particle methods | Text/fixture; offline transfer and SPH-gradient checks | Browser PIC/FLIP/APIC transfer, support degeneracy, SPH/PBF dynamics and full memory costs |
| 12 Time/error | Browser recomputed periodic viscous mode; explicit/implicit/exact comparison at fixed final time | General step admission, coupled spatial/temporal error, incomplete intervals and long-horizon budgets |
| 13 Memory/parallelism | Text/fixture; explicit layout arithmetic | Interactive allocation estimates, measured peak memory, reduction order and sparse/GPU execution |
| 14 Guidance rendering | Browser native 24³ flow with orbitable accepted concentration volume; fixed opacity and complete 3D tensor | Camera calibration, physical scattering, depth/normals export, silhouettes and export-error comparison |
| 15 Finite arithmetic | Text/fixture; offline binary32/binary64 comparison | Interactive cancellation, precision replay, exceptional values and threshold behavior |
| 16 Lean contracts | Text/disclosures and exact source links | Interactive hypothesis/counterexample inspection; no browser proof execution |
| 17 Rust implementation | Browser native force/projection timestep: one fixed interval, exact cycle comparison, native precommit rejection/retry and reset | General stepping lifecycle, asynchronous cancellation, resource-failure UI, liquid integration and other host runtimes |
| 18 Validation | Text/fixture; individual lesson diagnostics | Unified error/refinement explorer, production performance and downstream guidance evaluation |
| 19 Force and wall work | Stored projection as context; separate native forcing records | Live force-work ledger, swept geometry and two-way coupled motion |
| 20 Collision meshes | Stored triangle crossings; separate native geometry records | General cut-cell meshes, robust predicates, swept motion and deforming geometry |
| 21 Density/volume | Stored analytic hydrostatics; separate forced-column records | Browser conservative amounts, evolving interfaces and variable-density momentum |
| 22 Viscosity/stress | Stored scalar decay; separate browser finite-strain operator | General tensor/free-surface composition and accepted strain timesteps |
| 23 Wetting | Stored constant-volume equilibrium caps | Gravity, reconstructed angles, relaxation, hysteresis and moving contact lines |
| 24 Wall slip | Stored one-wall Couette; separate native two-wall/no-slip/forced playback | General wall geometry and evolving-boundary traction |
| 25 Integrated qualification | Stored cap pressure; links to restricted native mechanisms | Dynamic capillarity, parasitic currents, oscillation, penetration and full coupled qualification |
| A Algorithm reference | Text/fixture; pseudocode and ownership contracts | Interactive lifecycle/diagnostics; pseudocode is not the current compiled API |
| B Reproduction | Text/fixture; source identities and commands | No interactive solver claimed; inspect fresh receipts for each executed build |
| C Bibliography | Text/source links | No interactive lesson needed; preserve primary-source attribution |
| D Notation/provenance | Text/definitions | No interactive solver claimed; retain units and historical source scope |
| E Matrix laboratory | Text/fixture; exact chain, cycle, residual and flux arithmetic | Future recomputed small-matrix comparison; existing 27-cell blend is a different fixture |
| F Contracts/sources | Text/disclosures; links to the six reference mechanisms | Full requirement coverage remains incomplete; historical theorem counts retain their source identities |

## CAD measurement research and proposed lessons

Chapter 14 and the [CAD visualization guide](../research-book/implementation/cad-visualization-contracts.md)
add a beginner progression, 44 primary-source records, a source-anchored native
field inventory and seven proposed analytic lessons. The companion checks
prescribed tensors, planar flow, synthetic refinement, poly6 support, trajectory
curves and pressure-load arithmetic. It calls no Rheon solver and adds no live
browser controls, so the 15/25 chapter placement count is unchanged. The existing
sticky example remains the bounded carrier-flow lesson. Material inspection,
stage-aligned pressure, quantitative history and CAD wall loads remain separate
implementation and scientific prerequisites.

## New lesson sequence and measurable limits

The generated `transport-labs.html` sequence adds **sampler** and **transport**
alongside the original six-mechanism `labs.html` tour.

1. **Find the correct lattice.** Select cell/x/y/z sample locations, inspect the
   eight donor weights, and compare the interpolated affine field with direct
   evaluation. Correct offsets reproduce the field up to finite arithmetic;
   deliberately applying the cell offset to the original x-face fixture gives
   a signed bias of **−0.07**. Inspect weight sum, donor range and interpolation
   error. Complete regular support is assumed; this does not qualify wall
   extension, clamping, obstacle traces or a general sampler.
2. **Measure what transport loses.** Compare linear interpolation, raw
   forward/backward correction and donor-bound fallback for prescribed periodic
   translation. Inspect shape, min/max, RMS error, sample-sum change and fallback
   activity. A sharper profile is not evidence of conservation or uniform
   second-order convergence. This is a 1D scalar mechanism, not a liquid step,
   self-advected velocity or a 3D fluid solution.

The sine uses analytic translation as its RMS reference. The other profiles
use translation of the initial periodic piecewise-linear interpolant, not an
exact discontinuous-interface solution. Sample sums are not a dimensional
liquid-volume ledger.

The existing companion gives a small diagnostic of the second lesson:
`q=[0,0,1,1,0,0]`, Courant `0.5`, produces raw extrema **−0.125/1.125**;
donor-bound fallback reverts two samples. This symmetric case happens to keep
its sum. The asymmetric `q=[0,0.2,1,0.4,0,0]`, Courant `0.25`, reverts one sample
but changes the sum from **1.6 to 1.61875**. These mechanism witnesses explain
why bounds and conservation need separate measurements; they are not new
historical manuscript results or native acceptance baselines.

The native [advection kernel](../../src/advection.rs) currently uses basic
midpoint semi-Lagrangian sampling. Its [sampler](../../src/sampling.rs) clamps
to the fixed box and each field's support; clamping invalidates an unrestricted
temporal-order claim. It has no corrected-transport fallback API. The browser's
periodic corrected lesson therefore demonstrates the book's companion method,
without extending native solver capability.

## Genuine 3D flow and display limits

`flow-lab.html` runs the unchanged native solver through WebAssembly in a worker.
A compact source injects appearance tracer for 30 accepted steps; a localized
oblique force drives nonzero x/y/z velocities. The fixed fixture ends at 80
accepted steps. The renderer samples all 24³ cell values with trilinear display
interpolation and integrates a fixed Beer–Lambert opacity law along perspective
rays. Smooth pixels do not increase numerical resolution.

The optional overhead lighting uses a vertical concentration integral to shade
colour while preserving opacity. Its optical-depth prefix is exact at cell
centres and interpolated elsewhere; this is a depth cue, not a scattering solve.
Six fixed seeds trace the current staggered velocity field with a fixed 0–1 m/s
colour scale. These instantaneous streamlines show directions and speeds at one
accepted state; they are not material trajectories or additional fluid detail.

The source region is an injection marker, not a collision object. The surrounding
sealed box is the actual domain. Static obstacle pressure remains a separate
operator; impermeable obstacle-aware advection and tracer composition are not
implemented by this lesson. Tracer amount may decline after injection ends;
its bounded transport is not conservative liquid volume. The run has no physical
buoyancy, viscosity, free surface, moving geometry or procedural smoke.

## Next theory visualizations

The [concept/control/prediction/limit roster](BROWSER_TIMESTEP.md#bounded-visualizations-for-the-remaining-theory)
identifies bounded interactive work for all 12 previously uncovered chapters
and the appendices. Chapters 14 and 17 now have the bounded flow volume and fixed timestep lessons; its broader
resource and ownership topics remain partial. Proposed visualizations are
backlog items, not implemented coverage or new physics.

## Proof boundaries and remaining blockers

[Transport.lean](../../proofs/Rheon/Transport.lean) proves convex bounds with
nonnegative weights summing to one, restricted upwind/diffusion positivity and
explicit rational counterexamples. It does not prove JavaScript/Rust sampling,
affine reproduction, corrected-advection convergence or arbitrary conservation.
The browser comparisons test concrete computations; no new Lean qualification
or theorem count is implied.

The [aligned-strain laboratory](../research-book/implementation/aligned-strain-laboratory.md)
already supports genuine finite-operator recomputation: 48 active faces, 210
retained rows and six zero rows. Its [browser verifier](aligned_strain_browser.py)
compares 46 control states with an independently derived rational operator when
a packet is included. Local rotation cancels selected shear rows, not global
loss under fixed walls. The rounded coefficient estimate `B=17` is unenclosed
and authorizes no timestep. Exact projection-composition claims require their
stated matched masses and exact full-residual solve.

The remaining blockers have different owners:

- **Browser integration:** the missing adapter/build path is now implemented
  for the fixed force/projection timestep and a bounded 24³ source-pulse sequence. General grid/parameter admission,
  longer trajectories, asynchronous cancellation, resource-failure presentation
  and other browser engines still need separate qualification. The existing
  liquid APIs remain outside this first adapter.
- **Packet availability:** native playback and strain computation require the
  original admitted packets or a newly qualified packet under their documented
  rules. Missing recordings are not permission to invent substitute results.
- **Geometry and stress:** bounded stationary obstacle pressure and reduced
  shear consume the shared geometry owner, but their boundary models differ.
  [AlignedStrain](../../src/aligned_strain.rs) is diagnostic, not a time
  integrator. General moving/interface/tensor composition remains separate work.
- **Liquid and contact physics:** restricted native liquid transport exists;
  arbitrary interfaces, dynamic wetting, hysteresis and integrated capillary
  validation are not supplied by the cap reference or these new scalar lessons.
- **Evidence scope:** mathematical fixtures and conditional Lean statements do
  not establish IEEE bounds, physical calibration, whole-program performance,
  arbitrary geometry or downstream image fidelity. Historical receipts and
  source citations remain unchanged.

This teaching work adds no pressure API, surrogate training, solver refactor,
main merge or Pages deployment.

## PR43 continuation: one-face liquid volume

Branch `research/book-pr43-continuation` starts at PR43 `66cc567`, preserving
its pending work. The new chapter 9 example raises placement to 16/25 partial
chapters; appendices remain 0/6. Earlier 15/25 counts above describe earlier
milestones. Solver/scheduler/CAD/AI implementation claims remain scoped to
inspected source; no contracts from concurrent workers have been assumed.

| Item | Implemented / researched / proven / validated |
| --- | --- |
| Geometric lesson | `volume_reference.js`, `volume_labs.js`: one axis-aligned face, empty receiver, independent browser calculation, controls and 3D geometry |
| Primary research | Official Basilisk `vof.h`, reconstruction/swept flux and multidimensional compression qualifications; source and limits in chapter 9 |
| Mathematical scope | Six new exact-real statements in `VolumeFlux.lean`; current kernel status comes from `native-sequence.json`, never from source inventory matching |
| Arithmetic validation | `test_volume_reference.py`: all 20,402 discrete slider states versus independent Fraction integration, plus invalid-input rejection |
| UI validation | `volume_browser.py`: generated chapter, desktop and 390/320px phone controls, snapshots, scroll bindings, WebGL and overflow; run receipt outside Git |
| Remaining | General PLIC, multiple-face capacity coordination, native/WASM liquid integration, convergence, floating-point proof, physical and performance validation |

The PR43 artifact-download 403 remains unresolved; this continuation does not
reuse or claim a visual review of that artifact.

## Unmerged implementation coordination, 2026-10-13

Descriptions were checked against each pinned draft's implementation and tests;
these are not newly activated capabilities in this book checkout. Chapters 11,
17 and 22 add prediction/comparison exercises without increasing the 16/25
scroll-linked chapter count or claiming new browser runtimes.

| Draft and exact inspected head | Implemented in that draft | Evidence and remaining integration |
| --- | --- | --- |
| [PR47](https://github.com/MrScripty/Rheon/pull/47), `1923d5bd3442a5369878a1a317b52f4a89026d5b` | Committed post-emission/pre-integration particle diagnostics; latest owned inspector state; 256-sample selected trace and CSV | Source and tests inspected; evaluation versus accepted time and later wall response distinguished. No native-window/clipboard qualification claimed here; not integrated with PR48. |
| [PR48](https://github.com/MrScripty/Rheon/pull/48), `f06a1270935b2ac285c68c0e594ab0742220913e` | Full-box transport session, borrowed constructor/accepted snapshots, terminal stop and cooperative deadline/cancellation | Source and contracts inspected. No hard deadline, Godot binding, automatic dataset qualification, particle/CAD/AI or experimental Stokes capability. |
| [PR45](https://github.com/MrScripty/Rheon/pull/45), `bbc6089888e257962b968e70ed84ec61b7668d5b` | Opt-in stationary aligned-box Stokes transaction and native/rational qualification driver | Ten exact mathematical controls independently passed in normal/optimized Python; 13 native one-step cases and two three-step sequences are the driver's roster, not a fresh native rerun here. Experimental Lean enclosures remain assumptions and are not imported into root proof counts. |

The book branch retains PR43's exact Cargo.lock binding: the admitted lockfile
SHA256 is `b6da773967a1aea1db3fe0a26e3e7e990a68f701777a9f261fdb4cf883ca6d7a`.
No physics-branch stale binding was copied, no admission rule was relaxed, and no
proof source or source-inventory byte changed in this coordination update.
Any later integration changing native inputs requires fresh source-bound native
qualification. Passing book proofs or describing a draft does not qualify a
merged solver. Moving-body/general-physics and unified API claims remain open.

## Modal time integration milestone

Branch `research/book-time-mode-continuation` starts at PR44 `793b4d6`.
Chapter 12 adds a recomputed viscous shear mode in a periodic unit cube, with
independent controls for integrator, fixed final time, step count and displayed
step. Fixed-scale velocity glyphs distinguish energy stability, sign preservation
and temporal accuracy. It advances a scalar amplitude, not a native fluid solver.
Coverage becomes 17/25 partial chapters; appendices remain 0/6. Earlier counts
above are historical milestone counts.

| Item | Implemented / researched / proven / validated |
| --- | --- |
| Research | Stam (1999), Stable Fluids, sections 2.2–2.3 and introduction; original author-hosted paper read, specific relation/limits in chapter 12 |
| Browser computation | `decay_reference.js` repeats explicit/implicit scalar updates; exact exponential and analytic energy; `decay_labs.js` draws physical sample velocities at a fixed scale |
| Mathematical statements | Eight amplification/energy/sign statements in `ModalDecay.lean`; PDE, sine integral, convergence order and IEEE implementation are excluded. Fresh kernel status is in `native-sequence.json`; PR44 receipts do not certify this extension |
| Independent arithmetic | All 402 allowed control states compare to exact Fraction powers and 60-digit Decimal exponentials; sampled field/energy and invalid-input probes pass in normal/optimized Python |
| Actual GUI | `decay_browser.py` checks real ArrowHelper origins, lengths/directions, controls, state-preserving camera, desktop/390px/320px layouts and six chapter deep links; qualification requires a fresh hosted run |
| Remaining | General diffusion-grid spectrum, spatial refinement, nonlinear/forced/moving-boundary physics, native integration, physical and performance validation |

PR49 `b8ca87cab7f74f64c1686714a33e843fbb178c3d` is a separate unmerged
pressure-provider draft; chapter 5 distinguishes its fixed model domain, full
residual/fallback gates and caller-owned corrected-velocity acceptance. PR50
`3f49a158a0f1bec93f47edf6cc42ad86242e4aa5` is a separate unmerged desktop
projection inspector; chapter 14 distinguishes its retained operator result
from live history, arbitrary CAD and wall-load measurement. Source and docs were
inspected; this book milestone does not rerun or claim their native qualification.


## Engineering finite controls: continuation of PR51

Chapter 8 now has a recomputed two-cell shared-flux and boundary-work lesson.
Chapters 4 and 22 add collocated checkerboard and affine stress probes. All are
prescribed finite references, not a new native CFD backend. The 126 bounded
states have independent rational checks; browser geometry/control qualification
and fresh Lean status are separate. See [plan](ENGINEERING_PLAN.md) and
[dossier](../research-book/engineering/mesh-faces-to-collocated-flux.md).

| Item | Implemented/researched | Proof/validation boundary |
| --- | --- | --- |
| Oriented shared flux | 81 two-cell controls, unit volumes, balanced external flux; boundary-work counterexample | Exact shared cancellation and affine pressure work; no mesh parser, positivity or native transport claim |
| Collocated probe | 18 constant/alternating periodic-strip controls; developer-authored coupling source read | Exact central blindness, compact response and periodic energy; no full Rhie–Chow/SIMPLE or general stability claim |
| Affine stress | 27 rotation/shear/extension controls; Batty–Bridson v2 section 3 read | Exact planar power; prescribed samples and face tractions, no evolving PDE or no-slip cube |
| GUI | Three 3D views, seven heading bindings, independent glyph verifier and mobile controls | Hosted qualification required; no screenshot visual review implied by automated checks |
| Dependencies | Existing locks unchanged; no new dependency, external mesh/data or copied implementation | Zero-change audit, not approval of a future parser/AMG/model dependency |


## Pressure compatibility and gauges: continuation of PR60

Chapter 5 adds seven recomputed reading states over two disconnected Neumann
pairs. Chapter placement remains 18/25 partial and 0/6 appendices; this deepens
an existing chapter rather than claiming a new covered chapter.

| Item | Implemented/researched | Proof/validation boundary |
| --- | --- | --- |
| Component compatibility | 288 finite controls, compatible and globally balanced/incompatible data; PETSc primary nullspace notes read | Necessary component sums and explicit compatible pair solution; no arbitrary-mesh kernel theorem |
| Gauges | Fixed-cell, arithmetic mean, volume mean and independent offsets | Exact jump invariance and mean identities; physical Dirichlet/traction conditions excluded |
| Residual and boundary work | Full vs retained-row residual, volume-scaled divergence, explicit source projection, local pressure perturbation | Original incompatible source stays rejected; gauge work retains component boundary terms |
| Research | Gebhardt–Romero v1 sections 1–2 read; Bochev–Lehoucq publisher abstract/metadata only | No imported regularization/convergence claim or inaccessible-full-text reading claim |
| Qualification | Rational oracle and false-acceptance mutations; actual cell/bar/arrow browser verifier | Fresh 15-theorem kernel and hosted UI/full-book checks pending; screenshot review separate |

See [plan](PRESSURE_GAUGE_PLAN.md) and
[complete derivation](../research-book/engineering/pressure-compatibility-and-gauges.md).
