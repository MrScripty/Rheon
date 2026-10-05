# Flat-column shear quadrature prerequisite

This additive packet qualifies an isolated, fixed-flat-free-surface shear slice:
constant density, periodic lateral directions, liquid-only row-sum mass,
interior-interval strain quadrature and zero endpoint shear traction. It does
not extend the moving-surface carrier facade or implement adhesion.

Model, equations, basis and unresolved pressure/momentum compatibility:
`docs/research-book/implementation/column-shear-quadrature.md`.

    cargo run --locked --release --example column_shear -- /tmp/column-shear-replay
    python3 tools/verify_column_shear.py /tmp/column-shear-replay
    python3 -O tools/verify_column_shear.py /tmp/column-shear-replay
    python3 -m unittest discover -s tools -p test_column_shear_verifier.py -v
    python3 -O -m unittest discover -s tools -p test_column_shear_verifier.py -v

The Python verifier requires NumPy 2.3.5 and Pillow 12.3.0, also pinned in CI.
`demo` contains 24 cases / 1298 native intervals, including each nodal mass,
actual force, actual stored f32 update, and momentum/energy ledger.
`lean-qualification` contains actual positive, negative and restored builds and
axiom audits. Original proof pins/inventory are retained.
`qualification` contains actual Rust feature-matrix and Python logs.
`legacy-replay` binds every frozen native example and original Jacobi outputs.
`preservation.json` binds all frozen evidence/book/PDF/proof bytes, including the
prior viscosity f32 failure and every nested/master receipt.

    python3 evidence/column-shear/verify.py --negative-self-test
    python3 -O evidence/column-shear/verify.py --negative-self-test

Pass `--evidence-commit COMMIT` to bind the root receipt and complete packet to its
frozen Git commit. Only the root receipt is excluded from its own SHA map;
nested receipts remain included. Source/evidence identities are separate.
