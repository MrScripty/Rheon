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
| Workflow visibility gap | New architecture requirement | Recorded local native workflow gates passed for configured sealed MAC in [G1 qualification](reports/g1-qualification.md). G1-C is reopened for iteration-count consistency disposition and integration; G1-U retains its recorded environment scope. Other families/editing remain outside acceptance; HTML remains a design aid. |
| Hidden provider state/side effects | Contract risk | C1 explicit prepare/commit/abort or stateless/disposable candidate; old pressure seam cannot roll back external side effects. Open implementation gate. |
| Hosted source admission | Integration blocker | PR122 Pages build stops before numerical comparison because the reviewed native source inventory is stale. Require independently reviewed complete current closure and fresh committed native/rational qualification; no hash-only refresh or gate skip. |
| G1 archived iteration-count consistency | Source-derived contract finding | New workflow validation appears to permit otherwise coherent accepted native iteration counts above the declared budget. Workflow/recording owner must reproduce and reject the new-profile case or justify its contract disposition; no solver failure is asserted. |
| Optimized Python evaluator guards | Evidence-tool defect | PR123/124 static evaluators accept incomplete synthetic input under `python -O`; PR126 has the same assertion-dependent input-check pattern, not separately reproduced here. Preserve frozen normal-mode results; separately qualify an always-on guard correction before treating the reusable gate as supported. |
| Merge authority | Integration blocker | Automatic approval review rejected PR119/120 merges under the earlier trusted no-merge delegation. No bypass/retry; direct approval requested while review and documentation continue. |
| MARIN numeric observations | Specific missing input | V1 physical validation only blocked; no block to reference/graph/contract work. |

Independent review findings and their dispositions are recorded in reports/reviews.md; no claim is accepted merely because this table proposes a fix.
