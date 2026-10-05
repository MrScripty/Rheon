# Newtonian sealed-box viscosity qualification

This additive packet qualifies the bounded model described in
`docs/research-book/implementation/newtonian-box-viscosity.md`. It is a fully
filled, constant-density, free-slip closed box. Moving free-surface viscosity,
adhesion and physical material calibration are not claimed.

Reproduce native cases in a fresh directory:

    cargo run --locked --release --example viscosity -- /tmp/viscosity-replay
    python3 tools/verify_viscosity.py /tmp/viscosity-replay
    python3 -O tools/verify_viscosity.py /tmp/viscosity-replay
    python3 -m unittest discover -s tools -p test_viscosity_verifier.py -v
    python3 -O -m unittest discover -s tools -p test_viscosity_verifier.py -v

`demo` contains nine isolated decay/refinement cases and four coupled box cases.
`refinement-limit` retains a completed native case that fails its accuracy oracle
at 2053 stored-f32 updates. It is not part of the passing refinement inventory.
`lean-qualification` contains positive, real negative (`sorry`/extra axiom), and
restored audits in an isolated pinned Lean project; original proofs are preserved.
`qualification` contains actual Rust and Python logs for the final source.
`legacy-replay` binds unchanged original Jacobi, transport, static free surface and
352-file column reconstruction replays. `preservation.json` binds all frozen
research/proof/evidence bytes, including every historical master/nested receipt.

    python3 evidence/viscosity/verify.py --negative-self-test
    python3 -O evidence/viscosity/verify.py --negative-self-test

Add `--evidence-commit COMMIT` to bind this packet's root receipt to its frozen Git
commit as well. The receipt records exact source/tree/ordered parents and all
packet SHA-256 identities; only itself is excluded from its own hash map.
