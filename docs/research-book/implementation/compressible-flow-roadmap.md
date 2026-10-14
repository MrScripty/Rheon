# Compressible flow: scope, coverage audit and implementation roadmap

The compressible extension is a research specification with exact finite contracts and a small original Python Euler reference. Rheon's Rust production APIs still do not implement a general compressible solver. The [state and waves chapter](../chapters/26-compressible-state-and-waves.md), [discretization chapter](../chapters/27-compressible-discretization.md), and [boundaries and validation chapter](../chapters/28-compressible-boundaries-and-validation.md) form a separate progression after the continuum and finite-volume foundations.

## Audited coverage

The audit inspected current main `c7e88d8dfd58969e6020d089a11fb96795f0f5e3` and PR67's `research/book-runtime-qualified-integration` at `24b38606e8c8354e56a2f83f1d0c44d6cd7b932e` on 13 October 2026. The audit is a source observation, not a statement about later revisions.

| Topic | Existing main / PR67 coverage | Added research scope |
| --- | --- | --- |
| Density, momentum and total thermodynamic energy | Incompressible density assumptions; conservative liquid mass/momentum remapping explicitly lacks thermodynamic energy | Conservative Euler/Navier–Stokes state, pressure recovery, energy flux and admissible state |
| Compressible gas/liquid EOS and waves | Weakly compressible particle pressure mentioned; continuum chapter explicitly omits acoustics and shocks | Ideal/stiffened gas and Tait limits, acoustic characteristics and jump conditions |
| Flux/reconstruction, entropy, positivity, CFL, low Mach | Scalar transport bounds and incompressible pressure/transport time controls | LLF/HLL/HLLC progression, distinct admissibility obligations and acoustic stiffness |
| Boundaries, viscosity and heat | Extensive bounded incompressible solid/stress contracts; no compressible thermal boundary state | Characteristic data, conservative ALE wall work, cut-cell and thermal requirements |
| Turbulence | Prescribed perturbations and visualization limits; no solved validated closure | Separate DNS/LES/RANS requirements and validation obligations |
| Computed compressible verification | No compressible Euler fixture or shock reference found | Small computed Euler volume records, independent reference comparisons and staged V&V ladder |

Searches covered book chapters, appendices, implementation guides, education scripts and Lean sources for compressibility, Riemann fluxes, shocks, entropy, Mach and turbulence; relevant passages were inspected in context. PR67's runtime/publication qualification is owned separately. This branch neither edits PR67 nor authorizes racing its deployment.

## Deliverable and claim boundaries

The [original numerical reference](../compressible/README.md) advances a small Cartesian ideal-gas Euler state with first-order local Lax–Friedrichs fluxes. The [3D laboratory](../../education/compressible-lab.html) selects computed records and changes the camera, field and slice. It does not recalculate physics when a slider moves. Planar shock-tube extrusion is explicitly distinct from a diagonal three-dimensional acoustic field. Neither demonstrates arbitrary solids, viscous heat transfer, turbulence or calibrated real-gas/liquid behavior.

The [Lean source](../../../proofs/Rheon/Compressible.lean) states its exact-real algebraic hypotheses. Kernel qualification establishes those declarations only. Numerical conservation tolerances and finite reference comparisons are separate evidence; neither establishes the correctness of all floating-point executions or empirical physical validation.

## Implementation sequence

1. **State and ownership.** Introduce a bounded cell-centered owner for density, three momentum components and total-energy density, with one configured EOS, unit convention, validated parameter ranges and resource budget. Keep appearance tracers independent. Candidate updates must report failures and preserve the accepted state; do not hide inadmissible states by unconditional clipping.
2. **First conservative flux.** Use one shared face flux per internal face and explicit boundary ledgers. Begin with a robust first-order LLF/HLL baseline, sound-speed and EOS checks, directional acoustic CFL and fixed Cartesian geometry. Verify uniform-state preservation, closed/periodic conservation and shock-tube profiles before adding high order.
3. **Accuracy and admissibility.** Add limited reconstruction and SSP time integration with explicit positivity treatment, EOS-compatible wave speeds and measured entropy behavior. Test near vacuum, strong rarefaction and shock cases as well as smooth convergence. Resolve high-Mach cancellation in pressure recovery and low-Mach pressure/velocity scaling independently.
4. **Boundaries and solids.** Count incoming characteristic information for each boundary regime. Share fluid volumes, oriented face areas, swept geometry and wall states across transport and force accounting. Add stationary fitted walls before arbitrary cut cells; qualify small-cell stabilization and moving-wall GCL/work against their own references.
5. **Viscous and thermal physics.** Add conservative stresses and heat flux with material transport laws, no-slip/slip and thermal wall conditions, viscous time restrictions or qualified implicit solves, and laminar verification. Extend the EOS and thermal model only with validated ranges and tests.
6. **Turbulence and physical validation.** Select DNS, LES or RANS for an explicit intended regime; supply resolution/filter/closure, wall treatment and statistical uncertainty evidence. Noise, confinement and unresolved Euler vorticity are not calibrated turbulence. Run experimental comparisons with parameter and discretization uncertainties recorded.

The five-stage ladder in Chapter 28 assigns a distinct purpose to shock tube, acoustic pulse, smooth vortex, oblique shock and laminar boundary layer. Production acceptance requires native Rust refinement evidence and physical benchmarks after these educational references; no delivery date or achieved production milestone follows from this roadmap.

## Reproduction, licenses and integration

The numerical README gives commands for normal and optimized Python tests and a fresh external packet. The education README gives the build, browser, PDF and link checks. Generated editions, packets and screenshots live outside Git. The existing pinned Lean 4.19.0/mathlib source inventory and allowed-axiom audit remain mandatory.

Original added source remains under the repository's Apache-2.0 license. Papers and agency theory are linked and paraphrased; their code, images and full text are not redistributed. No GPL implementation or linked GPL dependency is introduced. The existing NumPy numerical dependency uses its BSD license; the existing Three.js and KaTeX browser assets retain their MIT notices. The standards reviewed for this work are [revision 188beda1](https://github.com/MrScripty/Coding-Standards/tree/188beda1fa477d21d576c233dd8c7f4c4c267d23), including verification/oracles, frontend accessibility, licensing and coordinated release requirements.

The [20-source inspection ledger](../sources/compressible-sources.json) records source identities, actual inspection scope and the claims each reference supports. A metadata-only inspection is labelled as such.

Publication requires a reviewed PR, fresh checks on the integrated source, visual review of the rendered edition and coordination with PR67's owner. The research branch is a review artifact, not an independent production deployment.
