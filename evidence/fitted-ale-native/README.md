# Native fixed-bottom ALE transport on the preceding physical flow

Exact source/evidence identities and complete preservation are in `receipt.json`.
This is the nonzero-relative-transport successor of the frozen material-frame
flow, using the frozen finite-transport research reference. It is not a new
unrelated analytic family or a general pressure-coupled liquid simulator.

The public `FixedBottomAleFlow` reuses the existing accepted-state owner and
candidate/solver vectors. One additional fitted workspace is strictly candidate
scratch. Physical 8/16-point quadrature computes separated positive/negative
shared transfers; endpoint masses come from actual geometry; bounded GMRES
solves conservative transport and viscosity together. Liquid geometry, full
velocity, analytic zero pressure and clock publish at one final barrier.
The four-column owner payload is 136,352 bytes; no step allocates heap storage.

`qualify_rust.py NEW_DIRECTORY` performs the actual three-feature Rust test/
Clippy matrix, formatting and focused release tests. `qualify_native.py
NEW_DIRECTORY` builds six debug/release native trajectories, independently
replays each in normal/optimized Python and rejects nine real corruptions.
`bind_final_build.py NEW_DIRECTORY` rebuilds the exact final Rust/example bytes
and requires six byte-identical qualified payloads, recording final binaries and
complete Rust input SHA-256s. Output directories are never overwritten.

The validator uses the actual high-precision finite face antiderivatives from
`evidence/fitted-finite-transport`, rather than mass marginals or fitted transfers.
It independently detects a true zero-balance face circulation and wrong frozen
endpoint flux. It also verifies every stored mesh, nodal volume/mass, velocity,
actual dt/clock, all momentum components and the changing-mass work/residual ledger.

The recorded cap reaches offset 0.125 at time 0.5. Native temporal errors versus
the time-dependent semidiscrete ALE ODE are 0.00326517973851,0.00165313843184,
0.00083189297152. They demonstrate first-order temporal behavior on this spatial
model, not continuum spatial accuracy. All dt debug/release JSON is byte equal.
`accepted-replay.gif` and exported plots show actual accepted native geometry,
velocity and changing nodal masses; `render.py` invents no simulation frames.

The strict cap-path limit is retained. A nominal uniform-step development trial
overshot it by an ulp and was rejected without changing state. The example
requests the remaining last interval explicitly and serializes its actual dt;
for nominal dt=.025, the final interval is .024999999999999856. No owner clamp or
manufactured physical time is used. Raw trial failures, compilation/Clippy
findings, successful development data and the original research convex-bound/
face-provenance failures remain preserved.

Constrained velocity convex bounds remain false; a native inviscid basis fixture
confirms overshoot while momentum and work pass. No positivity theorem is added
for constrained velocities. Liquid masses remain positive from accepted geometry.
Pressure is analytic zero, and its image is checked by the existing constructor,
with no new uniform inf-sup or general free-surface pressure-solver claim.
The general advancing refusal, conditional Lean limits, original Jacobi fixtures
and every historical evidence/proof/book file remain intact. Nonuniform xy and
nontrivial pressure coupling remain the next integrated criteria.

Verify the frozen packet from the repository root:

```
python evidence/fitted-ale-native/verify.py --negative-self-test --evidence-commit HEAD
python -O evidence/fitted-ale-native/verify.py --negative-self-test --evidence-commit HEAD
```

Older source-bound packets should be verified in checkouts of their exact frozen
heads: this successor deliberately changes three Rust files to share the bounded
candidate workspace and accepted payload, while retaining their historical bytes
in Git and every frozen evidence file on disk.
