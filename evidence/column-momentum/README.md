# Conservative tangential column-profile remap prerequisite

Borrowed varying-height column geometry, liquid-only dual-slab masses, explicit
cap admission/removal, conservative momentum and measured mixing/rounding energy.
This does not publish MAC velocity, pressure, phase, geometry or physical time.
The surface-viscosity facade refusal remains.

Model/derivation/limits: `docs/research-book/implementation/column-momentum-remap.md`.

    cargo run --locked --release --example column_momentum -- /tmp/remap-replay
    python3 tools/verify_column_momentum.py /tmp/remap-replay
    python3 -O tools/verify_column_momentum.py /tmp/remap-replay
    python3 -m unittest discover -s tools -p test_column_momentum_verifier.py -v
    python3 -O -m unittest discover -s tools -p test_column_momentum_verifier.py -v

`demo`: 13 native cases / 127 remaps, complete fields/ledgers and grayscale pixels.
`lean-qualification`: actual positive/negative/restored pinned builds and audits.
`qualification`: actual Rust feature matrix and normal/optimized Python logs.
`legacy-replay`: complete frozen Jacobi and six native example replays.
`preservation.json`: frozen book/PDF/proofs/evidence, including all nested receipts
and the preceding viscosity f32 negative. The root receipt binds exact source
commit/tree/parents and every packet byte except itself.

    python3 evidence/column-momentum/verify.py --negative-self-test
    python3 -O evidence/column-momentum/verify.py --negative-self-test

Add `--evidence-commit COMMIT` to bind the complete frozen evidence inventory and
root receipt to Git. NumPy 2.3.5 and Pillow 12.3.0 match the pinned CI environment.
