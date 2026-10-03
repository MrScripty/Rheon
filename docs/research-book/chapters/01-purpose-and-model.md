# 1 The numerical contract

Rheon needs to place a moving three-dimensional fluid shape deliberately while leaving memory available for image-generation models. The repository describes a low-resolution, interactive simulation framework and a standalone demonstration application. This book develops the mathematical and engineering foundation for that framework. The recommended first implementation is a bounded Cartesian staggered-grid solver with explicit diagnostics, followed by a deliberately separate free-surface branch. Neither a rendered fluid image nor a successful pressure solve alone establishes the intended result.

The repository at revision 0cb48429789699578b640d30d9d5dc99c6921ce4 contains a README, license and ignore file. It has no existing solver whose behavior can be assumed. Accordingly, algorithms here are specifications and research recommendations until implemented and qualified. The numerical companion executes small reference experiments; the Lean companion checks selected exact discrete contracts. Proposed architecture must not quietly become a claim about software that does not exist.

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
