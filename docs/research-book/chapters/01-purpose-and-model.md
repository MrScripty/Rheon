# 1 The numerical contract

Rheon needs to place a moving three-dimensional fluid shape deliberately while leaving memory available for image-generation models. The repository describes a low-resolution, interactive simulation framework and a standalone demonstration application. This book develops the mathematical and engineering foundation for that framework. The recommended first implementation is a bounded Cartesian staggered-grid solver with explicit diagnostics, followed by a deliberately separate free-surface branch. Neither a rendered fluid image nor a successful pressure solve alone establishes the intended result.

**Historical baseline.** Revision 0cb48429789699578b640d30d9d5dc99c6921ce4 contained a README, license and ignore file. The original research therefore specified algorithms before a solver existed. **Correction, 20 October 2026:** audited main `718469b` contains implemented grid components, a particle material preview and separate bounded experiments. A research recommendation is not evidence that every described method was implemented. The numerical companion executes small reference experiments; the Lean companion checks selected exact discrete contracts.

The [claim-to-evidence audit](../appendices/G-paper-audit-and-corrections.md) distinguishes mathematical error, interpretation, implementation defects, unsupported generalizations and unresolved questions. It finds no demonstrated causal chain from a mistaken original Poisson formula to the observed water failures. [Chapter 29](29-liquid-solid-interaction.md) connects collision geometry to hydrodynamic boundary and coupling requirements. Puma retains authorship of this book; dated corrections preserve the original source history and evidence identities.

## Spatial accuracy has several meanings

A fluid can be physically plausible while occupying the wrong silhouette, or match a silhouette while violating mass conservation. We distinguish geometric error, numerical error, model error and downstream conditioning error. Geometric error measures interface position against a defined target. Numerical error compares the discretization against the selected mathematical model. Model error compares that model with the real phenomenon. Conditioning error measures how a generated image responds to a guidance representation. These quantities can move in different directions.

Let the simulated state be \(s\), a camera be \(C\), and the deterministic guidance renderer be \(R_C\). An image-generation system applies a map \(F\) to that representation and additional inputs \(z\):

\[
s \longmapsto R_C(s) \longmapsto F(R_C(s),z).
\]

A bound on the first map says nothing automatic about the sensitivity of the second. ControlNet demonstrates spatial conditioning with several image types, including depth and edges [S14]; it supplies no guarantee that any particular fluid boundary will be preserved exactly. Rheon's measurable responsibility should end at a documented guidance image and metadata unless the integrated diffusion model is separately evaluated.

For an orthographic camera with world width \(L_x\) and image width \(W\), a transverse position error \(\delta x\) produces approximately \(W\delta x/L_x\) pixels of displacement. At \(L_x=2\) metres and \(W=1024\), a one-centimetre error is 5.12 pixels. A 64-cell domain of the same width has 31.25 mm cells, or 16 pixels per cell. These arithmetic facts constrain what positionally accurate can mean. Interpolated surfaces vary continuously inside a cell but cannot recover arbitrary unresolved topology.

## A staged scope

The first stage should solve incompressible velocity in a stationary box with smoke-like scalar transport. This isolates pressure, advection, boundaries and memory ownership. The second adds solid obstacles and measured geometric error. The third adds liquid interfaces, volume diagnostics and liquid-specific pressure boundaries. Non-Newtonian stress and particle-grid methods are extensions whose cost must be justified by a concrete failure of the baseline.

A source mask, obstacle mask and fluid mask are different fields. A source injects state according to a schedule. An obstacle restricts motion. A liquid mask identifies pressure unknowns and free-surface neighbors. Treating all three as one occupancy array creates contradictory boundary behavior. Visual density is an advected tracer unless a model explicitly equates it with mass density; smoke brightness cannot be used directly as the coefficient in an incompressibility equation.

The first acceptance suite should establish finite state, controlled divergence, stated transport behavior, stable boundary semantics and reproducible guidance. A frame-time target remains a design target until a workload, hardware class, competing model-memory load and variability policy are chosen. Real time is not a theorem and a short reference script is not its benchmark.

## Reading the evidence

Each chapter derives discrete contracts before discussing implementation. Worked examples use small systems so sign, scale and conservation failures can be inspected by hand. The experiments extend these checks to a three-dimensional graph and periodic transport. Chapter 16 maps every formal theorem to its assumptions. Chapter 17 defines a Rust implementation sequence; later language ports should inherit verified behavior rather than independently reinterpret the equations.

## Gating a pressure proposal

Start with two cells, unit volumes and conductance, and source b = (1, −1).
The candidate p = (s, s−1+e) produces shared flux 1−e and full residual
(e, −e). Set e = 1/4 and tolerance 1/8: the proposal is rejected and the exact
fallback p = (0, −1) supplies flux 1. The displayed used field is the fallback,
so its residual is zero. This is a synthetic acceptance example; no AI model
runs in the browser. A real provider must additionally bind model/schema,
physical units, source identity, component compatibility and resource limits.

## A gauge shift is not an error

Set e = 0 and vary s. Flux and residual remain unchanged because only the
pressure jump enters this two-cell operator. Set tolerance 1/8 and e = 1/16:
acceptance now permits a nonzero residual. A passed tolerance therefore means
an explicitly bounded error, rather than an exact solve. Inspect the used
residual and the candidate contract together. A nonfinite or wrong-unit
proposal fails before numerical acceptance. Camera and reading controls leave
the candidate and used state unchanged.

<details><summary>Practical study proof scope</summary>

[PracticalStudy.lean](../../../proofs/Rheon/PracticalStudy.lean) states the
exact real-arithmetic premises. The pinned [kernel build and 330-declaration audit](https://github.com/MrScripty/Rheon/actions/runs/38478346517) passed; the integrated proof files match that checked source. The independent finite oracle checks the browser formulas; it does
not prove a Rust refinement, IEEE solver, continuum prediction or runtime speed.

</details>
