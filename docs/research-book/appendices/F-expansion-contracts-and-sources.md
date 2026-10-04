# F Expansion contracts, sources and evidence

This edition adds original finite contracts and local numerical references. The historical 20 theorems retain their source-bound evidence. New theorem qualification is recorded separately in `expansion/proof-qualification.json`; all 33 public theorems compiled, 45 declarations passed the transitive axiom audit, and negative audit/source-integrity fixtures passed. All new derivations and implementations are Rheon research work. No cited researcher reviewed or endorsed this edition.

## A map from requirements to evidence

| Requirement | New exact statement | Executed local reference | Remaining production gap |
|---|---|---|---|
| External forces | force_work_identity | signed finite-step work | spatial force interpolation and IEEE execution |
| Moving boundary | sealed_flux_compatibility | balanced/unbalanced component flux | swept mesh and geometric conservation |
| Collision mesh | no geometric theorem claimed | original cube segment/triangle queries | robust arbitrary-mesh predicates and cut cells |
| Positive density | positive_transmissibility, weighted_pressure_energy | layered hydrostatic increments | phase-interface coefficient assembly |
| Liquid volume | volume_source_balance | unequal-volume rational ledger | bounded multidimensional interface transport |
| Viscosity | implicit_energy_identity, implicit_energy_nonincrease | dense solve versus periodic eigenmode | actual tensor stencil and free-surface traction |
| Slip | slip_power_nonpositive | affine Couette boundary equations | curved wall frame and assembly |
| Wetting/adhesion | young_adhesion_identity, adhesion_bounds | fixed-volume spherical caps | dynamic angle, hysteresis and microscopic law |
| Surface tension | no full capillary theorem claimed | pressure jump and independent volume integral | balanced moving-interface force and parasitic currents |

## Assumptions of the new theorems

The domain is a finite set and values are exact reals. Internal_amount_balance and volume_source_balance assume balanced incidence columns. Sealed_flux_compatibility additionally assumes every cell constraint is solved, proving a necessary global compatibility condition rather than existence. Weighted_pressure_energy assumes positive areas, densities and distances and bounds a finite weighted-square energy. It does not establish that geometric assembly yields these quantities.

Hydrostatic_increment assumes nonzero density/distance and a prescribed exact pressure increment. The implicit energy results assume an exact discrete work equation; nonincrease additionally assumes nonnegative masses, time and dissipation. They do not prove solver convergence. Young_adhesion_identity assumes Young balance; adhesion_bounds treats the cosine as a bounded scalar, without proving trigonometry or interfacial thermodynamics. Slip dissipation assumes a nonnegative coefficient. Force_work_identity is a finite-step algebraic identity without a stability assertion.

No theorem proves mesh orientation, robust intersection, wetting existence, contact-line convergence, liquid volume bounds, production Rust correspondence, floating-point error or performance. The educational reference has no general liquid solver.

## Shared data and reproduction

Run `python3 docs/research-book/expansion/reference.py` from the repository. It writes `reference-data.json`, original figures and a source/data-bound receipt. The JSON schema identifies every lab as an analytic or local numerical oracle. The site consumes it directly; the figures use the same values. Numerical acceptance uses independent dense/modal, geometric and rational checks, not an aesthetic comparison.

Build the teaching edition with `python3 docs/education/build.py`; its README records math, browser and PDF steps. The generated full Markdown preserves all chapters and appendices, including this source map. Historical evidence files are not rewritten.

## Primary sources added

[E1] Batty, Bertails and Bridson (2007). A Fast Variational Framework for Accurate Solid-Fluid Coupling. [Primary source](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf). Prescribed/dynamic coupling, adjoint reaction, wall separation; Sections 2 and 4.

[E2] Robert Bridson. Author-hosted mesh_query and Tunicate implementations. [Primary source](https://www.cs.ubc.ca/~rbridson/). Original geometry implementations linked under Downloads; independently written Rheon segment fixture uses no copied source.

[E3] Basilisk contributors. Embedded boundaries, original source. [Primary source](https://basilisk.fr/src/embed.h). Geometry metrics, interpolation and small-cell policies; current online code, accessed 2026-10-04.

[E4] Batty and Bridson (2008). Accurate Viscous Free Surfaces for Buckling, Coiling, and Rotating Liquids. [Primary source](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf). Section 3: normal/tangential traction, viscous variational construction.

[E5] Basilisk contributors. Implicit viscous diffusion, original source. [Primary source](https://basilisk.fr/src/viscosity.h). Coupled stress, residual and relaxation implementation; current code, accessed 2026-10-04.

[E6] Basilisk contributors. 3D sessile drop, original test. [Primary source](https://basilisk.fr/src/test/sessile3D.c). Constant-volume spherical-cap comparison; no code/figures copied.

[E7] Afkhami and Bussmann (2009; author preprint dated 2008). Height functions for applying contact angles to 3D VOF simulations. IJNMF 61, 827–847. DOI 10.1002/fld.1974. [Primary source](https://web.njit.edu/~shahriar/Publication/IJNMF2.pdf). Section 2.5: wall frame and 3D height-function geometry.

[E8] Basilisk contributors. Contact angles, original source. [Primary source](https://basilisk.fr/src/contact.h). 3D orientation and shallow-angle support restrictions; current code, accessed 2026-10-04.

[E9] R. G. Cox (1986). The dynamics of the spreading of liquids on a solid surface. Part 1. Viscous flow. JFM 168, 169–194. DOI 10.1017/S0022112086000332. [Primary source](https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/dynamics-of-the-spreading-of-liquids-on-a-solid-surface-part-1-viscous-flow/97CAB1BF3439F4B1AA429FFA37C80C42). Publisher abstract: contact-line stress singularity, microscopic regularization and small-capillary-number asymptotics. No full-paper reproduction.

[E10] Basilisk contributors. Surface tension, original source. [Primary source](https://basilisk.fr/src/tension.h). Interfacial potential and capillary restriction; current code, accessed 2026-10-04.

## Verified expertise, without endorsement

[Christopher Batty](https://cs.uwaterloo.ca/~c2batty/). Author university page lists liquid/gas simulation, viscous flows, surface tension and embedded boundaries. This verifies the relevance of the source author, not an external review of Rheon.

[Shahriar Afkhami](https://web.njit.edu/~shahriar/). Author NJIT page identifies computational interfacial flows, dynamic contact lines and complex liquids. This verifies the relevance of the source author, not an external review of Rheon.

[Robert Bridson](https://www.cs.ubc.ca/~rbridson/). Author UBC page lists fluid simulation research, original implementations and primary publications. This verifies the relevance of the source author, not an external review of Rheon.


## Additional primary-source ledger

The focused contribution inspected these original works and implementations on 2026-10-04. R identifiers avoid collisions with the historical S bibliography. Access depth is retained; moving code URLs are references, not vendored qualified dependencies.

[R01] Batty and Bridson, Accurate Viscous Free Surfaces for Buckling Coiling and Rotating Liquids, 2008. [Primary source](https://www.cs.ubc.ca/labs/imager/tr/2008/Batty_ViscousFluids/viscosity.pdf). Full original text inspected, especially sections 3–5. Coupled symmetric-gradient viscosity, free-surface traction, variational dissipation, rigid-rotation counterexample.

[R02] Brackbill, Kothe, and Zemach, A Continuum Method for Modeling Surface Tension, JCP 100, 335–354, 1992. [Primary source](https://www.ljll.fr/~frey/papers/Navier-Stokes/Brackbill%20J.U.%2C%20A%20continuum%20method%20for%20modeling%20surface%20tension.pdf). Original-paper text surfaced through academic-hosted PDF. Continuum surface force and wall adhesion boundary treatment. DOI 10.1016/0021-9991(92)90240-Y.

[R03] Young, An Essay on the Cohesion of Fluids, 1805. [Primary source](https://catalogues.royalsociety.org/CalmView/Record.aspx/Record.aspx?id=L%26P%2F12%2F86&src=CalmView.Catalog). Original archival metadata verified; the journal DOI endpoint returned 403. Used only for historical attribution of material-dependent contact angle, not to attribute modern notation verbatim. DOI 10.1098/rstl.1805.0005.

[R04] Huh and Scriven, Hydrodynamic Model of Steady Movement of a Solid Liquid Fluid Contact Line, 1971. [Primary source](https://www.sciencedirect.com/science/article/abs/pii/0021979771901883). Original abstract inspected. Creeping-flow contact-line stress/dissipation singularity under adherence. DOI 10.1016/0021-9797(71)90188-3.

[R05] Cox, The Dynamics of the Spreading of Liquids on a Solid Surface Part 1 Viscous Flow, JFM 168, 169–194, 1986. [Primary source](https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/abs/dynamics-of-the-spreading-of-liquids-on-a-solid-surface-part-1-viscous-flow/97CAB1BF3439F4B1AA429FFA37C80C42). Original abstract and bibliographic details inspected. Small-Ca matched-asymptotic/slip assumptions verified. Full publisher text was not retrieved; do not present the cubic shorthand as its general finite-angle equation. DOI 10.1017/S0022112086000332.

[R06] Qian, Wang, and Sheng, Molecular Scale Contact Line Hydrodynamics of Immiscible Flows, PRE 68, 016306, 2003. [Primary source](https://sheng.people.ust.hk/wp-content/uploads/2017/08/Molecular-Scale-Contact-Line-Hydrodynamics-of-Immiscible-Flows.pdf). Original full text inspected. Generalized Navier boundary condition, Cahn–Hilliard framework, molecular comparison and fitted dynamic parameters. DOI 10.1103/PhysRevE.68.016306.

[R07] Batty, Bertails, and Bridson, A Fast Variational Framework for Accurate Solid Fluid Coupling, 2007. [Primary source](https://www.cs.ubc.ca/labs/imager/tr/2007/Batty_VariationalFluids/). Original project, paper text, and sample-code descriptions inspected. Kinetic projection, static irregular-boundary samples, and author hindsight on improved weights. Sample archive license not verified.

[R08] Popinet, An Accurate Adaptive Solver for Surface Tension Driven Interfacial Flows, JCP 228, 5838–5866, 2009. [Primary source](https://www.sciencedirect.com/science/article/pii/S002199910900240X). Original publisher abstract/conclusion inspected. Balanced-force VOF with height-function curvature and stationary-drop equilibrium. Author PDF endpoint timed out. DOI 10.1016/j.jcp.2009.04.042.

[R09] Basilisk original contact-angle implementation. [Primary source](https://basilisk.fr/src/contact.h). Source and explicit limitations inspected. Height-function normal fallback and shallow-angle limitation are visible in the implementation.

[R10] Basilisk original surface-tension implementation. [Primary source](https://basilisk.fr/src/tension.h). Source inspected. Explicit capillary timestep restriction and curvature potential construction.

[R11] Akinci, Akinci, and Teschner, Versatile Surface Tension and Adhesion for SPH Fluids, TOG 32(6), 2013. [Primary source](https://cg.informatik.uni-freiburg.de/publications/2013_SIGGRAPHASIA_surfaceTensionAdhesion.pdf). Original text inspected. Cohesion and boundary attraction as practical SPH models. DOI 10.1145/2508363.2508395.

[R12] Weiler, Koschier, Brand, and Bender, A Physically Consistent Implicit Viscosity Solver for SPH Fluids, CGF 37(2), 2018. [Primary source](https://animation.rwth-aachen.de/publication/0558/). Author abstract and accepted-paper text inspected. Conservation, refinement, and spurious-viscosity evaluation criteria.

[R13] Bender et al., Implicit Frictional Boundary Handling for SPH. [Primary source](https://animation.rwth-aachen.de/media/papers/67/2020-TVCG-ImplicitBoundaryHandling.pdf). Original manuscript passages inspected. Boundary viscosity/friction extension and separate boundary coefficient. Filename uses 2020; final issue metadata was not verified.

[R14] SPlisHSPlasH Weiler2018 implementation. [Primary source](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/blob/master/SPlisHSPlasH/Viscosity/Viscosity_Weiler2018.cpp). Source inspection confirms μ = configured viscosity × rest density and separate boundary coefficient. No build/execution.

[R15] Macklin and Müller, Position Based Fluids, 2013. [Primary source](https://mmacklin.com/pbf_sig_preprint.pdf). Original algorithm inspected. Positional density constraints, artificial pressure, velocity postprocessing, and performance/accuracy tradeoff.

[R16] Koschier and Bender, Density Maps for Improved SPH Boundary Handling, SCA 2017. [Primary source](https://animation.rwth-aachen.de/publication/0554/). Original author abstract inspected. Precomputed continuous boundary density contribution.

[R17] Bender, Kugelstadt, Weiler, and Koschier, Volume Maps An Implicit Boundary Representation for SPH, MIG 2019. [Primary source](https://animation.rwth-aachen.de/media/papers/65/2019-MIG-VolumeMaps.pdf). Original text inspected. Kernel-independent geometric-volume contribution, rigid boundaries, comparison with density maps.

[R18] Guendelman, Selle, Losasso, and Fedkiw, Coupling Water and Smoke to Thin Deformable and Rigid Shells, SIGGRAPH 2005. [Primary source](https://graphics.stanford.edu/papers/thin_shells_fluid_coupling-sig05/). Original abstract and paper passages inspected. Infinitesimally thin surfaces, ray-based visibility, and incompressibility treatment.

[R19] Basilisk original embedded-boundary implementation. [Primary source](https://basilisk.fr/src/embed.h). Source inspected. Volume/face geometry, small-cell CFL restriction, tracer redistribution. Static implementation details are not asserted to establish arbitrary moving-boundary correctness.

[R20] Basilisk core license text. [Primary source](https://basilisk.fr/src/COPYING). GPL version 3 text verified. Per-file scope and downstream compatibility still require checking.

[R21] SPlisHSPlasH repository license. [Primary source](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/blob/master/LICENSE). MIT notice verified. Repository README separately warns that dependencies have their own terms.

[R22] Basilisk original 2D sessile-drop test. [Primary source](https://basilisk.fr/src/test/sessile.c). Source and analytic geometry comparison inspected. The implementation tests 15°–165° and notes shallower limitations.

[R23] Basilisk original 3D sessile-drop test. [Primary source](https://basilisk.fr/src/test/sessile3D.c). Original source inspected as the 3D comparison. No local reproduction performed.

[R24] Basilisk original parasitic-current test. [Primary source](https://www.basilisk.fr/src/test/spurious.c). Source inspected. Constant density, diameter 0.8, Laplace-number-controlled viscosity, viscous-time observation, resolution study.

[R25] Basilisk original drop-oscillation test. [Primary source](https://basilisk.fr/src/test/oscillation.c). Source/diagnostics inspected. Frequency error and equivalent numerical viscosity are separate quantities. Its dimensional setup must be retained when reproducing it.

[R26] Washburn, The Dynamics of Capillary Flow, Physical Review 17, 273–283, 1921. [Primary source](https://journals.aps.org/pr/abstract/10.1103/PhysRev.17.273). Original abstract inspected. Cylindrical-capillary square-root penetration formula. Full text is access controlled; no access bypass attempted.

[R27] Gong et al., Contactless Surface Tension Measurement of Molten Oxides Using Oscillating Drop Method in an Aerodynamic Levitator. [Primary source](https://pmc.ncbi.nlm.nih.gov/articles/PMC11470527/). Original measurement equations and experimental cautions inspected. Supports Rayleigh frequency convention and the need to control external forcing, mode splitting, and rotation. No material values imported as Rheon presets.

[R28] François, Cummins, Dendy, Kothe, Sicilian, and Williams, A Balanced Force Algorithm for Continuous and Sharp Interfacial Surface Tension Models within a Volume Tracking Framework, JCP 213, 141–173, 2006. [Primary source](https://www.researchgate.net/publication/222522360_A_Balanced-Force_Algorithm_for_Continuous_and_Sharp_Interfacial_Surface_Tension_Models_Within_a_Volume_Tracking_Framework). Original author-uploaded text inspected; publisher endpoint failed. Exact-curvature balanced-force experiment is an appropriate control before testing estimated curvature. DOI 10.1016/j.jcp.2005.08.004.

[R29] SPlisHSPlasH Akinci2013 implementation. [Primary source](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/blob/master/SPlisHSPlasH/SurfaceTension/SurfaceTension_Akinci2013.cpp). Original source inspected. Separate fluid and boundary force coefficients, with boundary-method branches. No assumption that every branch reproduces every original-paper conservation claim.

## Next operator proofs, not compiled claims

The nine focused targets are stationary implicit strain/wall dissipation; moving-wall work with residual norm; affine pressure constraint work; two-way impulse cancellation; rigid strain null modes; conservative redistribution/geometric conservation; balanced capillary rest; work-compatible surface energy; and contact-line friction/pinning. The current checked Physics module covers finite energy identities, conservation, positive coefficients, relative slip power and Young algebra. It does not supply these larger assembly or surface-evolution proofs.

In particular, a nonnegative stiffness is insufficient for a rigid-rotation null mode. A conservative redistribution matrix needs column sums one, but positivity and barrier connectivity need additional conditions. Balanced rest assumes M is symmetric positive definite, an exact solve, and a capillary force actually in the chosen gradient range. Surface-energy conservation requires a discrete gradient satisfying its exact work identity; triangle area is not globally convex in arbitrary vertex coordinates. A scalar contact-coordinate dissipation proof cannot establish a 3D contact-line PDE.

The contributed `expansion/contact/sanity_checks.py` executes 28 algebra/formula checks, including 100 synthetic SPD systems for each of the stationary, moving-wall and affine-projection identities. These remain numerical sanity checks, separate from Lean compilation and from a Rheon liquid run. No third-party research code or paper figure is bundled.
