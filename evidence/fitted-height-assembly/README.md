# Native fitted-height operator evidence

Implementation: `src/fitted_height.rs`; public export in `src/lib.rs`.
[Native contracts and limits](../../docs/research-book/implementation/fitted-height-native-assembly.md).
Fixed periodic fitted strip, all three momentum components, shared geometry and
face fluxes, sparse divergence-image pressure and full symmetric strain.
Instantaneous assembly/inspection only; existing advancing refusal unchanged.

`qualify_rust.py OUTPUT` executes actual pinned Rust formatting, complete
feature-matrix tests/strict Clippy and focused release contracts.
`qualify_native.py OUTPUT` builds/runs actual debug/release examples, hashes their
binaries and validates both with normal and optimized Python.
`verify_native.py NATIVE_JSON --negative-self-test` independently checks against
the preserved exact-rational formulation and rejects five actual corruptions.
`make_figure.py` draws actual native instantaneous geometry/mass-rate arrays.

`verify.py --negative-self-test --evidence-commit COMMIT` binds exact source,
every packet file including nested receipts, and all historical evidence,
proofs/research-book/education bytes at the merge base. Only this packet's root
receipt is excluded from its own map. Source and generated evidence freeze as
separate commits. PR17 main and all later frozen research lineage are preserved.

The 15 historical Lean identities are conditional finite exact-real algebra;
no new proof, physical trajectory, pressure solve, uniform stability or pointwise
traction qualification is claimed. Numerical gates here cover native assembly,
not an advancing liquid simulator.
