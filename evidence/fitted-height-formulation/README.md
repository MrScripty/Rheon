# Fitted-height research qualification

Selected formulation: [book derivation](../../docs/research-book/implementation/fitted-height-liquid-formulation.md).
Fixed-topology Powell–Sabin fitted geometry, full-vector lumped momentum on nodal
median duals, actual shared fluid-minus-mesh fluxes, divergence-image pressure,
full symmetric-strain work and natural coupled traction. Production Rust and the
advancing-viscosity refusal remain byte-identical to the preservation base.

Run `python reference.py output.json` and `python -m unittest test_reference.py`
in this directory; repeat with `python -O`. `make_figure.py` makes a labelled
analytic-reference figure, not native rendering. The instantaneous moving-mesh
GCL and reduced third-component rate are exact rational assembly. The separate
finite donor algebra explicitly linearizes masses and freezes instantaneous
fluxes; it does not certify a finite material mesh trajectory. The flat-cap
manufactured predictor is derived from its BE equation rather than reused from
the desired endpoint.

`qualify_lean.py ISOLATED_PINNED_PROJECT FRESH_OUTPUT` compiles and axiom-audits 15
conditional exact-real identities with Lean 4.19.0 and original pinned mathlib.
It compiles actual sorry/extra-axiom variants, requires their audit rejection,
and rebuilds/re-audits restored exact source. It does not edit frozen proofs.

`verify.py --negative-self-test --evidence-commit COMMIT` checks exact qualified
source identities, every packet file (including nested receipts), and the entire
prior evidence/proofs/research-book/education inventory. Only this packet's root
receipt is excluded from its own SHA map. Source and generated evidence are
separate commits. No production build is claimed for this research-only change;
production source is checked identical to the already qualified frozen base.

The next slice is bounded fitted geometry/dual/operator assembly and native
instantaneous diagnostics; the space-time integrator, uniform pressure stability,
pointwise traction, temporal error, production budget and advancing/rendered
liquid simulation are not qualified by this packet.
