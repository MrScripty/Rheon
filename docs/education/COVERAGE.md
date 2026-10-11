# Research-book interactive coverage

This ledger covers all 25 chapters and six appendices. The sampling/transport
follow-on expands the [540bdef teaching checkpoint](https://github.com/MrScripty/Rheon/commit/540bdef8fcbc0d0a55567325eba570589666bf80)
from 11 to **13 chapters with scroll-linked examples**: 3–7, 10 and 19–25.
No appendix has a scroll-linked example. These counts describe placement,
not complete coverage of each chapter's topics or qualification of a solver.
[Heading bindings](teaching.py) fail the build when an expected section moves.

The [layout screenshots](screenshots/README.md) show the earlier six-reference
checkpoint at `540bdef8fcbc0d0a55567325eba570589666bf80`; they do not demonstrate
the subsequent sampling/transport computations. Their provenance and the
separate, unchanged PR42 native-app captures must not be conflated.

## What executes

| Source kind | Meaning and source |
| --- | --- |
| Stored reference | The original six lessons select analytical or dense-solve records from [reference.py](../research-book/expansion/reference.py). [labs.js](labs.js) recomputes the labelled projection blend and readouts; it does not rerun the pressure solve. |
| Browser mechanism | [transport_reference.js](transport_reference.js) recomputes bounded affine sampling and periodic 1D scalar transport. Its references are the existing [sampling fixture](../research-book/companion/depth_experiments.py) and [transport formulas](../research-book/companion/experiments.py). It is an independent teaching implementation. |
| Browser operator | [aligned_strain.js](aligned_strain.js) applies immutable native strain rows and their transpose to chosen face samples. It recomputes force and loss, without a timestep or pressure solve. A qualified packet is required. |
| Native recording | Wall/force, static geometry, obstacle pressure and reduced-shear pages display admitted native packets. Browser controls choose saved cases, slices or times. Missing packets remain unavailable. |
| Text/fixture | Derivation, source links, proof disclosures or offline numerical fixtures; no corresponding browser computation is supplied. |

The [shared viewer](../../browser/viewer/README.md) supplies rendering, camera
controls and theme. Neither this branch nor the pinned GUI supplies a browser
fluid timestep engine. Native [Simulation::step](../../src/simulation.rs) and
[LiquidTransportSimulation::step](../../src/liquid_step.rs) do exist; the book
has no WASM or service binding to them. Browser scalar transport and finite-row
evaluation must not be described as invoking these native APIs.

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
| 09 Liquid interfaces | Text/fixture; offline axis-aligned flux and curvature examples | Browser fraction transport, reconstruction, reinitialization and surface-error comparisons |
| 10 Forces/viscosity | Stored hydrostatics and scalar shear; separate native wall/force records | Variable/nonlinear tensor stress and dynamic solid coupling |
| 11 Particle methods | Text/fixture; offline transfer and SPH-gradient checks | Browser PIC/FLIP/APIC transfer, support degeneracy, SPH/PBF dynamics and full memory costs |
| 12 Time/error | Text/fixture; transport lesson available separately | Step admission, temporal versus spatial error, incomplete intervals and long-horizon budgets |
| 13 Memory/parallelism | Text/fixture; explicit layout arithmetic | Interactive allocation estimates, measured peak memory, reduction order and sparse/GPU execution |
| 14 Guidance rendering | Text/fixture; native opacity exporter exists | Browser camera calibration, depth/normals, silhouettes and export-error comparison |
| 15 Finite arithmetic | Text/fixture; offline binary32/binary64 comparison | Interactive cancellation, precision replay, exceptional values and threshold behavior |
| 16 Lean contracts | Text/disclosures and exact source links | Interactive hypothesis/counterexample inspection; no browser proof execution |
| 17 Rust implementation | Text/fixture and current native source links | Browser accepted-state control, cancellation, resource failures and solver integration |
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

- **Browser integration:** native state, stepping and failure APIs exist, but
  no browser execution/binding lifecycle exposes them. Live stepping needs its
  own accepted-state, cancellation, resource and cross-runtime qualification.
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
