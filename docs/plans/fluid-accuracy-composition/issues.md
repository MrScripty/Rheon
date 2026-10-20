# Findings and decisions

The frozen audit remains authoritative for baseline observations; this table is a repair disposition, not a resolution claim.

| Finding | Severity | Bounded disposition / status |
|---|---|---|
| F1 missing hydrodynamic mesh walls | High | P1 production selection stopped after retained failed gates; Q1→Q2 is the selected existing-grid route. Particle preview remains unqualified; F1 stays open until the shared geometry/wall suite passes. |
| F2 water rest transient | High | P1 wall/EOS candidates failed and are not production-admitted. Existing particle preview is unchanged; a future particle admission must meet equilibrium/work/sound/Mach/dt gates. F2 remains open. |
| F3 no composed general grid path | High | Q2 unified Stokes + common geometry + conservative transport; explicit staged 2D→3D envelope. Open. |
| F4 center-sampled mass bias | Medium | P2 clipping/integrated mass with uncertainty and variable-volume quadrature. Open. |
| F5 fixed-neighbor viscosity bias | Medium | The bounded perturbed-cloud/viscosity studies are retained without admitting a method. Particle production selection is stopped; F5 remains open, with no global normalization shortcut. |
| F6 coarse channel/interface diffusion | Medium | Q1 source/evidence delivered in PR126, but transport accuracy qualification is incomplete: n64 source exhaustion and Full-control refusals remain. Original coarse failure preserved. Q2 common geometry is the next prerequisite; F6 remains open. |
| F7 area-lumped obstacle pressure/load | Medium | Q2 common B/M/S and independent flux/traction/refinement; new weight alone insufficient. Open. |
| F8 uncalibrated cohesion/contact work | Medium | Q3 capillary stress/drop/wave + impulse/work ledger; moving/coupled capability separate. Open. |
| Workflow visibility gap | New architecture requirement | Recorded local native workflow gates passed for configured sealed MAC in [G1 qualification](reports/g1-qualification.md). G1-C iteration consistency is repaired at PR131/34e2e73 and locally verified on fixed stage `9c04278`; hosted integration remains pending. G1-U retains its recorded environment scope. Other families/editing remain outside acceptance; HTML remains a design aid. |
| Hidden provider state/side effects | Contract risk | C1 explicit prepare/commit/abort or stateless/disposable candidate; old pressure seam cannot roll back external side effects. Open implementation gate. |
| Hosted source admission | Hosted integration pending | The fixed 135-path stage `9c04278` has independently approved admission and fresh native/rational/real-packet qualification; see [staged result](reports/native-admission-20261020/qualification.md). Old PR122 Pages refusal remains historical; hosted Pages/browser/publication must pass on the new integration head. |
| G1 archived iteration-count consistency | Repaired and locally verified | PR131 reproduced six standalone/cold-terminal validation failures and fixed native caps/zero candidate counts at `34e2e73`. Matching RCP4 forgeries already failed the legacy inner gate. Fresh23 workflow tests pass on `9c04278`; old-profile semantics remain. |
| Optimized Python evaluator guards | Corrected tooling contract | PR130/c2fbbea retains all four original evaluator bytes and promotes all 15 predicates to always-on guards. Owner qualification reports 129 checks in normal/-O/-OO; independent full-AST reversal preserves formulas. Frozen scientific campaigns/failures are unchanged; this adds no exhaustive schema or physics validation. |
| Q2 geometry topology | Numerical promotion blocker | PR127/7d7306a independent review found five positive native apertures on exact-zero-measure source faces. Finite-error integration does not admit topology/DOF/B/M/S/W assembly. A principled boundary repair is active and excluded from fixed stage `9c04278`. |
| Merge authority | Confirmed for ready reviewed PRs | After direct confirmation, PR120 merged as `193d86c` and PR119 as `718469b` after fresh green readiness checks. Earlier denials remain retained. Implementation/hosted gates remain required; no source branch was deleted. |
| Future particle and Lagrangian roadmap | Deferred user priority | Qualify voxel/grid first, then a particle solver and particle-to-surface mesh generator, with a Lagrangian finite-element-style family later. Surface reconstruction is separate from material volume/connectivity meshing. Both future phases are unimplemented/unqualified; failed P1 evidence remains. |
| MARIN numeric observations | Specific missing input | V1 physical validation only blocked; no block to reference/graph/contract work. |

Independent review findings and their dispositions are recorded in reports/reviews.md; no claim is accepted merely because this table proposes a fix.
