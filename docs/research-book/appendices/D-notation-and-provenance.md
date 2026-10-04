# D Notation and provenance

Velocity, pressure, density, dynamic viscosity, kinematic viscosity, spacing and physical step are \(\mathbf u,p,\rho,\mu,\nu,h,\Delta t\). Advected scalar, level set and liquid fraction are \(q,\phi,f_i\). Units follow the model, not the letter alone.

In the graph model, B is cells by oriented internal faces, negative at tail and positive at head. Physical divergence is \(-B\mathbf u\) at unit spacing; pressure difference is \(B^Tp\); \(A=BWB^T\). Euclidean graph norms are unweighted. Physical integrals require volume or face metrics. MiB is 1,048,576 bytes; GiB is 1,073,741,824 bytes.

This research edition is credited to Puma at the user's request. Covers were generated specifically for the book and visually reviewed. They are illustrative, not numerical evidence. Original prompts and hashes accompany the artwork. Explanatory diagrams are independently generated from definitions and experiment data; no cited-paper figures are reproduced.

The native-equation publishing approach follows the project family's research-book style. Scientific content and theorem claims are Rheon-specific. The proof environment reuses a previously qualified pinned Lean/mathlib combination with independently authored fluid contracts.

The dense baseline, typed outcomes, component gauges and immutable snapshots are recommendations for Rust implementation review. Sparse grids, APIC, non-Newtonian materials and integrated diffusion evaluation remain extensions. No application GUI, calibrated presets or measured production performance is claimed.

The repository's original license is Apache 2.0. Cited papers retain their own rights and are linked rather than redistributed. Generated illustration provenance is documented without asserting identical copyright treatment in every jurisdiction. No publisher affiliation, ISBN, peer-review status or physical validation is claimed.
