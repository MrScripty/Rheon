# CV1 private reference owner

`examples/grid_cut_variational_reference.rs` is the only executable owner. It
accepts one fresh output directory and runs the four frozen protocol fixtures;
it never advances state or solves a pressure system. Material constants are
rho=1 kg/m³ and mu=.1 Pa*s, with 1m reference length/depth. GridGeometry supplies
native MAC indexing; its single z layer carries no physical end-cap condition.

`clip` classifies represented rational geometry and refuses positive pieces that
collapse in native arithmetic. `basis` defines both velocity fields on complete
staggered nodal hulls. `assembly` integrates full mass, divergence, symmetric
strain and physical moment maps on common halfcell pieces. `trace` independently
integrates the same basis on actual source edges and signed Cartesian apertures.
`observe` evaluates the fixed work/rigid/affine/pressure-control diagnostics.
`export` flushes source inputs before admission, then completed raw operators,
geometry, basis and observations before numerical gate evaluation. A refusal
leaves existing files in place; the caller retains stdout, stderr and exit status.

The private test adapter includes exact-geometry contracts read-only from Q2.
It adds support-hull/coincidence, positive rational triangle, quadrature, basis
and nodal pressure-sign contracts. No production API or algorithm is changed.
No dependency is added beyond the already reviewed exact-reference dev deps.

The independent evaluator reconstructs polynomials from source floats using
Fraction integration; it does not reuse native Gauss samples to assert exactness.
Its source hashes must precede first data inspection. Method/source admission
and native source freeze precede selected tests and the one fresh CPU campaign.
