# Rheon exact discrete contracts

This project owns exact finite-dimensional reference definitions and 33 checked theorem statements in four modules. It does not own the production solver, geometry assembly, IEEE arithmetic, continuum physics or performance claims.

## Reproduce

Use the exact lean-toolchain and lake-manifest.json. From this directory run:

    python3 scripts/check_sources.py
    lake exe cache get
    lake build
    lake env lean AxiomAudit.lean
    python3 scripts/test_audit.py

The source inventory records reviewed bytes. Do not regenerate it in CI. Review changed statements and the audit inventory before updating digests. The kernel audit allows only propext, Classical.choice and Quot.sound. It rejects admitted and custom assumptions; negative fixtures verify both rejection paths. Native-evaluation axioms are outside the allowlist.

## Qualified evidence

Source head ecf97d3a943ebe30d20acc1cf01b94dc08ff96a2 passed hosted run https://github.com/MrScripty/Rheon/actions/runs/37141752646. The run compiled all modules and audited 31 declarations including generated equation/proof declarations. The retained log records the actual pull-request merge checkout.

## Assumptions and limits

Discrete.lean uses arbitrary finite real matrices. Balanced columns are required for conservation and constant nullspace; nonnegative weights for positive semidefiniteness; exact RHS and exact solution hypotheses for exact projection. Geometry validity, complete nullspace characterization, pressure existence/uniqueness and solver convergence are not proved.

Transport.lean proves convex interpolation bounds, one-dimensional upwind and explicit-diffusion positivity under stated step restrictions, and exact rational counterexamples. It does not prove general mass conservation for semi-Lagrangian transport.

Indexing.lean uses unbounded natural numbers. Machine integer overflow and allocation limits remain implementation obligations.

## Expanded physics contracts

Physics.lean adds thirteen finite statements for force work, positive coefficients, weighted energy, source/flux conservation, implicit dissipation, slip power and Young adhesion. `docs/research-book/expansion/proof-qualification.json` binds their current source inventory to local pinned Lean 4.19.0 qualification: all modules compile, 45 declarations pass the axiom audit, and negative/source gates pass normally and under optimized Python. The official cache returned HTTP403; a normal pinned source build completed. The historical evidence above remains unchanged.
