# Fluid portals: sources, mathematical scope and implementation proposal

[Chapter 40](../chapters/40-fluid-portals-and-open-boundaries.md) derives periodic transport and open hydrostatic reservoirs as distinct domain models. It includes original control-volume and weak-form arguments, weighted MAC pressure/compatibility/nullspace and projection-work proofs, hydrostatic cancellation, continuum backflow control, particle requirements and proposed verification. It contains prose and equations, no pseudocode or executing solver. Rendering the chapter establishes no native CFD property.

The [implementation proposal](../../plans/fluid-portals-open-boundaries/plan.md) owns stages, API/face/recording boundaries and acceptance. Its [standards review](../../plans/fluid-portals-open-boundaries/reports/standards-and-design.md), [ledger](../../plans/fluid-portals-open-boundaries/execution-ledger.md), [issues](../../plans/fluid-portals-open-boundaries/issues.md) and [verification report](../../plans/fluid-portals-open-boundaries/reports/verification.md) distinguish actual authoring/publication observations from future numerical gates. These linked Markdown sources are packaged with the downloadable companion; they are not a second live implementation authority.

## Source and license audit

Rheon `LICENSE` and `Cargo.toml` verify Apache-2.0 at main `f95cac0dbc7b283370fb0e9a845599db613d79de`. The [repository inventory](../sources/fluid-portals-repository-audit.json) records inspected source/policy hashes. The [external implementation audit](../sources/fluid-portals-license-audit.json) records pinned root license and selected source/NOTICE paths/hashes. No third-party code, figures or substantial paper prose is copied into this publication; no dependency is added.

| Candidate / exact inspected revision | Observed authority and scope | Reuse disposition |
| --- | --- | --- |
| incflo `ccb90bc41d9def18dddc4e1c4122a7d517add630` | Root BSD-3-Clause; selected boundary documentation/source. | Permissive source candidate for a later exact-file/notice review; not a drop-in Rheon method or dependency clearance. |
| MFEM `96c4c956093ec6a2551d4664f8b915a7425cbd1b` | Root BSD-3-Clause, mesh-source notice and root NOTICE inspected. NOTICE identifies separately licensed components. | Periodic-topology reference; no whole-dependency or fluid-stability clearance. |
| Oceananigans `b0c931e6836e8e12f9cd8757d2e415371f344134` | Root MIT; selected normal-radiation, tracer-reservoir and net-transport sources. | Reference for separate incoming state/global compatibility. Its transport correction is not authority to mean-shift Rheon's pressure RHS. |
| PySPH `1cd3dea9cfaac7d9f1bda2a2491ce4b42a31dbe5` | Actual LICENSE.txt has BSD-style terms and a local override rule; API label is NOASSERTION; selected inlet/outlet files inspected. | Conditional only, excluded from approved copying in this task until exact scope/terms are confirmed for reuse. Its primary SPH paper remains a scientific source. |
| DualSPHysics `ef3721a861fda961f0e2f9ec4cd317b19de99086` | LGPL-2.1 root license/metadata inspected; implementation not inspected/copied. | Excluded as a copied/translated implementation source for this task. This is not a claim that LGPL software is universally incompatible or that associated mathematics cannot be cited. |
| Unverified supplement/manuscript code | Exact implementation terms not audited. | No copying or implementation-specific behavior claim. Cite verified papers and independently derive methods separately. |

Scientific citation does not import an implementation into Rheon. An implementation citation does not license copied expression or satisfy required notices. The audit evaluates this publication's actual activity and identifies possible future candidates; it grants no blanket linking, redistribution, patent or dependency clearance.

## Primary mathematical references

Chapter 40 links author manuscripts and official method references directly: Bridson/Müller-Fischer's MAC notes; Dong/Shen's generalized energy-stable pressure correction; Dong's boundary-inertia/Robin formulation; Ni/Yang/Dong's dissipation-scale analysis; Negi/Ramachandran/Haftu's SPH outlet comparison; Dong/Wang's two-phase conditions; and MITgcm's official regional boundary documentation. These sources have different variables, stress conventions, weak spaces and timestepping. Their continuum/numerical results are qualified to those formulations.

Orlanski's institutional scanned PDF endpoint was verified, but its body was not extracted; it is historical follow-up, not a reproduced discrete scheme. Brown/Cortez/Minion2001 and Tafuni et al.2018 are additional follow-ups whose attempted full text was unavailable. They do not support claims of directly inspected algorithm details here. No inaccessible supplementary code is approved by association with a paper.

## Proof, implementation and publication limits

Conditional proofs in Chapter 40 assume the stated stationary geometry, traces, regularity, constant density and exact algebra. The pressure-work defect identity explains the additional terms from actual divergence/correction error; it does not qualify f32 assembly or CG. Periodic mean acceleration under nonzero net force is physical. Hydrostatic support is pressure/gravity balance, not a force filter. A full-vector viscous traction formula cannot be assigned generic inviscid validity by setting viscosity to zero.

No new Lean theorem/receipt, numerical baseline, native example result, model-provider domain, surface reconstruction or particle boundary is implemented. The first proposed numerical milestone includes complete conservative periodic MAC transport and the existing user workflow. Open-water approximation requires the separate hydrostatic reservoir/backflow milestone and domain-sensitivity evidence. Particle support follows its own kernel/EOS/lifecycle requirements.

FEM 30–36 remain intact in their merged scope. GPU 37–39 remain reserved and unpublished by this task. Parent owns serial main integration and the normal validated Pages deployment; a draft PR or successful Markdown build is not evidence that the live site changed.
