# Findings and decisions

The frozen audit remains authoritative for baseline observations; this table is a repair disposition, not a resolution claim.

| Finding | Severity | Bounded disposition / status |
|---|---|---|
| F1 missing hydrodynamic mesh walls | High | P1 coherent support/visibility/force/work selection; P2 geometry; Q2 grid alternate. Open, preview not reclassified. |
| F2 water rest transient | High | P1 compressed-EOS equilibrium, native equilibrium defect, sound/Mach/dt admission. Open. Uniform fill was not initialized hydrostatic equilibrium. |
| F3 no composed general grid path | High | Q2 unified Stokes + common geometry + conservative transport; explicit staged 2D→3D envelope. Open. |
| F4 center-sampled mass bias | Medium | P2 clipping/integrated mass with uncertainty and variable-volume quadrature. Open. |
| F5 fixed-neighbor viscosity bias | Medium | P1 kernel/support/perturbed-cloud study; no global normalization shortcut. Open. |
| F6 coarse channel/interface diffusion | Medium | Q1 measured accuracy envelope retains old failure; Q2 transport selection. Open. |
| F7 area-lumped obstacle pressure/load | Medium | Q2 common B/M/S and independent flux/traction/refinement; new weight alone insufficient. Open. |
| F8 uncalibrated cohesion/contact work | Medium | Q3 capillary stress/drop/wave + impulse/work ledger; moving/coupled capability separate. Open. |
| Workflow visibility gap | New architecture requirement | Closed for existing configured sealed MAC by [qualified G1](reports/g1-qualification.md): authoritative descriptor, real receipts, native live/cold interaction and independent review. Other families/editing remain outside that acceptance; HTML is still only a design aid. |
| Hidden provider state/side effects | Contract risk | C1 explicit prepare/commit/abort or stateless/disposable candidate; old pressure seam cannot roll back external side effects. Open implementation gate. |
| MARIN numeric observations | Specific missing input | V1 physical validation only blocked; no block to reference/graph/contract work. |

Independent review findings and their dispositions are recorded in reports/reviews.md; no claim is accepted merely because this table proposes a fix.
