# Staged physical validation and solver requirements

Reviewed 2026-10-14 against Rheon main
`8baf459b91bb53ebd4a7230c4d9f3399dd5cb0bc`. This report proposes subsequent
gates; only the elementary suite linked in [plan](../plan.md) is executable in
this slice. Passenger-jet means flow around aircraft geometry, not a fluid jet.
No aircraft/submarine solution or experimental agreement has been executed.

## Three distinct acceptance decisions

1. **Numerical correctness:** independently derived PDE/manufactured references,
   conservation, boundary handling, convergence, solver errors and reproducible
   failure behavior. Pressure projection alone establishes none of compressible
   energy transport, shocks, turbulence, no-slip wall stress or moving bodies.
2. **Experimental agreement:** compare the same physical configuration and
   observables against measured data with uncertainty, support/transition/wall
   corrections and Reynolds/Mach/roughness matching. Do not tune on the held-out
   validation set. Agreement at one case does not establish the whole envelope.
3. **Training-label quality:** separate dataset-owner gate for field/mesh units,
   coordinate frames, synchronization, coverage, uncertainty, rights and leakage.
   A visually useful guidance map may be acceptable for image placement while
   failing calibrated pressure/force labels. Each label channel needs its own
   stated application tolerance and confidence mask; do not invent thresholds
   before the consumer and experiment uncertainty are specified.

## Progression with stopping conditions

| Stage | Supported elementary reference / required study | Deciding measurements and exit gate |
| --- | --- | --- |
| 0, implemented here | cosine continuum pressure, free-slip viscous modal decay, separate fixed-mesh time refinement | both public pressure methods; independent residual, stored f32 divergence, zero wall-normal velocity/flux, energy nonincrease; declared orders/budgets in validation README |
| 1, wall/load correctness | original box hydrostatics and affine pressure; planar no-slip Couette/Poiseuille once admitted | pressure and viscous traction separately; affine force `-V grad(p)`, hydrostatic buoyancy, moments about two origins, gauge invariance; analytic profiles and wall shear; mesh/time convergence and wall-location error |
| 2, elementary external bodies | original sphere/cylinder geometry, creeping-flow sphere where applicable; inviscid potential-flow comparisons only within their assumptions | pressure/Cp, drag/lift/torque, near-wall error, domain-size study, wake/velocity probes; do not use inviscid zero drag as a viscous drag target; require no-slip and fitted/cut geometry qualification |
| 3, compressible primitives, solver owner's scope | uniform flow, acoustic pulse, analytically specified shock tube and transported contact; multidimensional smooth manufactured flow | conservative mass/momentum/total energy versus integrated boundary/source flux, EOS/positivity, wave speeds/jumps, shock location, smooth-region order; compare time and mesh separately, record limiter/dissipation settings |
| 4, aircraft progression | low-speed airfoil before transonic wing, then transport-like wing/body | only rights-cleared matched geometry/data; surface Cp and Cf, CL/CD/CM, shock/separation location with spatial and temporal/domain uncertainty; no full-aircraft turbulent claim from 2-D laminar data |
| 5, submerged body at fixed depth | original analytic axisymmetric hull first; rights-cleared benchmark body later | pressure/friction drag separately, Cp/Cf/wake and moments, domain extent/blockage, Reynolds matching, resolved wall or documented wall treatment, mesh/time convergence |
| 6, forward translation while ascending | prescribed original hull motion, initially deeply submerged single-phase incompressible water | swept-volume/GCL, body-frame versus inertial-frame/Galilean comparison, moving-wall no-penetration, wall work, hydrostatic pressure and buoyancy, force/moment transforms; refine surface geometry, fluid mesh and dt independently |
| 7, target scenes and label acceptance | staged passenger transport and rising submerged hull | only capabilities already qualified; geometry/material/motion hashes, accepted actual times, probe/surface fields, independent uncertainty and rights records; no free surface/cavitation/propeller labels without their own qualification |

For stages 1 onward retain at least three nested meshes and three time steps,
including independent time refinement at fixed mesh and mesh refinement with
temporal error smaller than the spatial signal. Use weighted L1/L2/L∞ norms and
integral balances, pressure modulo the stated gauge, surface reconstruction and
wall-normal/tangential errors. Keep all coarse/fine points and floor/breakdown
observations. Predeclare acceptable changes in quantities of interest from
application tolerances; report observed order and extrapolation assumptions.
Use domain-size and wall-location variation to distinguish boundary errors from
discretization. For turbulence, use stationarity/transient exclusion and sampling
uncertainty rather than treating one instantaneous force as a converged mean.

Prescribe submarine inertial body velocity `(U,V,0)` with world +Y upward and
`U>0,V>0`; the body-fixed incoming flow is the opposite translation. Keep gravity
`(0,-g,0)`, reference pressure/depth, pitch/yaw and moment origin explicit. The
trajectory is prescribed initially; it does not claim positive-buoyancy dynamics
or two-way rigid coupling. Ascending toward a free surface changes the model and
requires free-surface/wave validation, not merely moving the solid in a sealed box.

## Requirements to relay to solver-plan owner through parent

- Export cell volumes, face areas/normals, material density, momentum and total
  energy where compressible, with pressure/EOS and units. State whether pressure
  is physical total/static/gauge or a splitting correction.
- Retain actual accepted dt, time/generation, scheme/limiter/EOS/turbulence IDs,
  boundary/source inputs and all rejected-step reasons. Validation must never
  infer an accepted step from requested time.
- Export independent budget ingredients: boundary fluxes, source work/heat,
  momentum/energy exchange with solids, moving-wall flux, swept volumes and GCL
  defect. Avoid accepting conservation using a residual computed by the same
  assembly path without an independent flux traversal.
- Provide surface pressure traces and viscous tractions at documented physical
  locations, area and normals, force sign convention and moment reference.
  Validate affine/hydrostatic force convergence before wall-load labels. Current
  `docs/broader-physics-milestone.md` records a factor-two affine load bias in a
  donor construction despite passing gauge/work checks; algebra alone is
  insufficient. This validation worker does not repair that solver path.
- Admit farfield/inlet/outlet, wall and moving-body capabilities individually.
  Return unsupported outcomes for absent physics; do not silently substitute a
  fixed-box free-slip projection for an external turbulent compressible scene.
- Keep deterministic, bounded CPU reproduction configurations and per-state
  export suitable for independent probing. No GPU or remote-worker evidence is
  required or claimed by this worker.

## Primary-source feasibility and rights review

These checks distinguish a web page, an accessible asset and verified
incorporation rights. No external geometry, measured rows, CAD or report is
copied into this branch. Newly authored analytic fixtures use original geometry
and derivations under the repository's Apache-2.0 license.

**NACA0012: conditional first aerodynamic candidate.** NASA's
[current TMR case](https://tmbwg.github.io/turbmodels/naca0012_val.html) supplies a
modified sharp trailing-edge definition, linked grids and measured comparisons.
It specifies essentially incompressible Re=6 million and suggests M=0.15 for
compressible solvers; transition and farfield treatment matter. It cautions
against treating available experimental configurations interchangeably. Its
[Ladson force file](https://tmbwg.github.io/turbmodels/NACA0012_validation/CLCD_Ladson_expdata.dat)
was linked but the web tool reported unsupported octet-stream content and native
retrieval returned proxy 403, so exact bytes were not verified. The
[NASA TM-4074 metadata](https://ntrs.nasa.gov/api/citations/19880019495) was readable:
public distribution, `GOV_PUBLIC_USE_PERMITTED`, no export control, available
download. This supports investigating the original NASA report; it does not
establish rights for every separately digitized TMR asset. Gate ingestion on
exact original/digitized identity, applicable notices and intended dataset use.

NASA's [TMR landing page](https://www.nasa.gov/nasa-turbulence-modeling-resource/)
confirms the move from turbmodels.larc.nasa.gov to tmbwg.github.io; preserve the
resolved address, not only the stale link.

**ONERA M6: conditional transonic wing candidate.** The primary
[NASA NPARC archive](https://www.grc.nasa.gov/WWW/wind/valid/m6wing/m6wing.html)
lists test 2308 at Mach 0.8395, Re 11.72 million, angle 3.06°, Cp sections and
measurement accuracy ±0.02. The
[section geometry](https://www.grc.nasa.gov/WWW/wind/valid/m6wing/airfoil.txt) and
[one Cp file](https://www.grc.nasa.gov/WWW/wind/valid/m6wing/cp1l.ex) were readable
through the web tool. It identifies ONERA/AGARD origins and a trailing-edge
modification; neither asset-specific redistribution nor training rights were
established. Hold copied geometry/data pending rights review. Solver feasibility
requires shocks, viscous turbulent boundary treatment and external boundaries;
the current elementary harness cannot run this experiment.

**NASA CRM: later transport-like geometry candidate.** The primary
[CRM site](https://commonresearchmodel.larc.nasa.gov/) and
[IGES listing](https://commonresearchmodel.larc.nasa.gov/geometry/iges-files/)
were readable and expose wing/body and several appendage/tail variants. Listing
availability does not verify CAD bytes or rights. No CAD download, exact
experimental pairing, aeroelastic/deformed configuration or asset-specific
dataset license was established here. Native retrieval was proxy-blocked.
Do not label CRM as an implemented passenger aircraft benchmark; coordinate
surface meshing, geometric closure, matched test configuration and rights first.

**SUBOFF: deferred, access/identity/rights unavailable.** The attempted
[DTIC candidate record ADA210642](https://apps.dtic.mil/sti/citations/ADA210642)
could not be opened by the web tool and native retrieval returned proxy 403.
No original geometry/report/data bytes, report identity, public-release terms or
redistribution/training rights were verified; do not substitute a remembered
title, third-party mesh, OpenFOAM case or unlicensed mirror. Parent/dataset owner
must resolve authoritative access and exact bare/appended configuration before
selection. Begin with original analytic hull/sphere fixtures in the meantime.

NASA hosting is not blanket permission for third-party content. The reviewed
[NASA usage guidance](https://www.nasa.gov/nasa-brand-center/images-and-media/)
also distinguishes protected identifiers and third-party material; do not infer
a dataset license from that general media policy. This report is a technical
provenance disposition, not a legal opinion.

## Tuldok and model acceptance receipt requirements

The elementary receipt intentionally marks experiment, body loads and label
quality `not_assessed`. A future physical dataset receipt must additionally bind
the exact geometry/data/mesh/license/notice bytes and permission determination,
solver configuration and source, experimental provenance and uncertainty,
material/EOS/turbulence/wall treatment, coordinate units/frames and moment origin,
prescribed/accepted motion and actual time, all balance ingredients, raw field
and surface exports, mesh/time/domain studies and evaluator versions.

Store separate per-channel label acceptance (velocity, pressure, traction,
forces/moments, rendered guidance); document interpolation, missing/unresolved
regions and uncertainty masks. Split training/validation/test by geometry and
flow/trajectory families to avoid correlated refinements or neighbouring frames
leaking into held-out acceptance. Freeze a versioned manifest only after parent
coordinates a compatible solver/validation checkpoint. No final release tag is
created by this work.
