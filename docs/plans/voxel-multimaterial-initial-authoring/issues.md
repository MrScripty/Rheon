# Issues and dispositions

| ID | Finding / decision | Owner | Current disposition and revisit condition |
| --- | --- | --- | --- |
| I01 | User requests voxel multi-material dynamics; prior particle-first research slice cannot satisfy it | Parent / plan owner | Resolved in candidate scope: no particle fallback, real weighted MAC/transport/stress work required |
| I02 | Full occupancy, incompressible gas and no surface tension materially limit the first model | Parent | **Open product review decision.** Proposed as minimum, not an approved user preference. If unacceptable, replan physical model before implementation |
| I03 | Existing constant-density pressure, semi-Lagrangian momentum and constant viscosity cannot implement distinct rho/mu by IDs alone | Numerical implementer | Mandatory M1–M2 work, pending evidence. No claim of existing support |
| I04 | Finite divergence can break donor fraction bounds; density/momentum consistency defect scales with rho_old | Numerical implementer + independent reviewer | Candidate contract specifies flux closure, constant-preserving realization and direct cumulative budgets. M2 stop/replan if ordinary supported cases cannot pass; no clamp/tolerance tuning |
| I05 | Weighted pressure could accidentally enter scalar fast paths | Pressure owner / integrator | Explicit full-path audit and heterogeneous dense fixtures before admission; Jacobi first, no untested SGS promise |
| I06 | Existing configured saved constructor permits only full-rest liquid; previews are not authoritative material input | Recording/native integrator | New explicit initial/run/profile variant and both-material view; no validator weakening, Debug replay or preset-name lookup |
| I07 | Numerical/native/environment evidence does not exist for this feature | Verification owners | A1–A6 pending. No simulation/benchmark/native run authorized or performed by planning. Identify graphics environment when M3/M5 exist |
| I08 | Shared operator/desktop/recording files may have other active owners | Parent/integrator | Reconcile exact current source/write set before each implementation slice. This plan reserves no production edits now |
| I09 | Independent complete-plan review required, distinct from adviser/self-review | Independent plan reviewer | Closed for plan review: complete round and targeted rereview passed; R1 gravity/viscosity order, R2 comparison-consumer scope and R3 keyboard center resolved. This is not implementation admission or A1–A6 acceptance |

No action was denied by approval review; no uncertain Git/publication operation exists. No push/PR/merge is requested or authorized before parent review of the completed plan.
