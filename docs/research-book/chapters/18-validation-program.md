# 18 Validation and remaining research

Formal algebra, numerical fixtures, integration tests and visual review prove different things. A proof build cannot replace camera-export validation; a beautiful render cannot replace divergence or volume checks.

## Executed evidence

The companion builds a closed 8-cubed graph with 512 cells and 1344 faces. It gauge-fixes pressure and projects deterministic random velocity. CG uses 78 iterations at the chosen tolerance. Divergence L2 decreases from 49.74366159 to \(4.4429\times10^{-10}\). Unweighted kinetic energy decreases from 642.9265376 to 401.9458778. The scalar update retains total 512 in the executed binary64 computation.

These are sparse-reference results, not Rust, cut-cell or laboratory evidence. Energy decrease is an observation consistent with projection, not a general proof. An exact rational three-cell flux example updates to 199/630, 96/175 and 61/450, summing exactly to one.

Transport and diffusion experiments are described in Chapters 7 and 10. Scripts assert stated properties and preserve data used by figures. No script timing is claimed as production performance.

## Production evidence matrix

Geometry tests cover shape arithmetic, transforms and orientation. Operator tests cover nullspaces, adjointness, symmetry and compatibility. Solver tests cover residual/divergence, termination and singularity. Transport tests cover smooth and discontinuous data, variable velocity and boundaries.

Forces need rest and decay tests. Liquids need sphere transport, volume drift, merging, thin sheets, rest and contact. Rendering needs calibration, invalid depth, normals and temporal rest stability. Integration needs coherent state versions, resource rejection, cancellation and replay. Each class requires an owner and diagnosis.

## Convergence and performance

Report grid, time step, norm, duration, boundaries and reference with each error plot. Several refinements reveal whether slopes stabilize. Coarse points can be pre-asymptotic and fine points can reach solver or rounding floors; neither should be discarded without explanation. Limiters and topology changes can lower observed order.

Choose representative sparse and dense scenes, obstacles, sources and output resolution. Measure simulation, pressure, rendering, readback and full-frame distributions, plus peak memory with host model allocations. The README supplies no numeric budget or target hardware. Until chosen, memory arithmetic is an estimate and real time remains an objective.

## Next research

The highest-value integrated study links coarse simulation error to spatial conditioning quality. Measure how displaced interfaces, missing thin features and flicker affect a pinned diffusion pipeline. More simulation detail may consume memory without improving final guidance.

Numerical priorities are component-aware pressure, obstacle sampling and bounded liquid volume. Formal priority is connecting array assembly to the abstract model. Non-Newtonian stress, APIC and sparsity follow needed capability rather than the project's name.

Every result should retain its level of certainty: derived identity, checked theorem, executed fixture, measured implementation or proposed design. That distinction makes a small reusable solver dependable.
