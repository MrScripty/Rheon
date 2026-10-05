# Native advancing translating viscous family

The exact qualified source and final evidence identities are in `receipt.json`.
The source milestone is followed by a separate evidence milestone. This packet
preserves its complete predecessor evidence/proofs/book inventory byte-for-byte.

`TranslatedViscousFlow` advances the restricted physical family U=(a,0,w),
H(x,t)=H0(x-at), analytic relative pressure zero and weak Neumann third viscosity.
The existing corrected fitted workspace supplies geometry/mass/embedding/strain.
One flow owns the common accepted geometry offset, velocity, liquid state and
clock; its 4-column heap payload is 59,184 bytes. General pressure-coupled liquid
advancing refusal remains unchanged. Actual relative transport is zero because
this is a material translating mesh, not a claim to have qualified general remap.

Run from the repository root using the selected pinned Rust environment:

```
source /workspace/rheon-setup/env.sh
python evidence/translated-viscous-flow/test_reference.py
python -O evidence/translated-viscous-flow/test_reference.py
python evidence/translated-viscous-flow/verify.py --negative-self-test --evidence-commit HEAD
python -O evidence/translated-viscous-flow/verify.py --negative-self-test --evidence-commit HEAD
```

`qualify_rust.py NEW_DIRECTORY` performs actual formatting, three feature test
suites, focused release tests, and strict Clippy in all three feature modes.
`qualify_native.py NEW_DIRECTORY` builds debug/release executables, records
three actual dt trajectories per profile, validates each with normal and
optimized Python, rejects eight real corruptions per validation and checks
byte-equal debug/release replay plus first-order semidiscrete temporal behavior.
It refuses to overwrite an existing output directory. The producer's validator
imports corrected **periodic-repair** rational spatial geometry; it replaces
all motion derivatives with the actual co-moving trajectory. It does not reuse
the old fixed-bottom instantaneous mesh-motion diagnostic. New printed labels
bind that corrected reference explicitly; frozen predecessor labels are retained.

At time 0.5 the cap offset is 0.125. Native mass-L2 temporal errors versus the
independent spatial semidiscrete exponential are 0.00204335573749,
0.00103601128697 and 0.00052177444300 for dt 0.05, 0.025 and 0.0125.
At dt 0.025, kinetic energy falls from 1.2738922366207124 to
1.1965310565836444; accumulated third-momentum drift is about 1.4e-12.
This is temporal validation on one fixed spatial mesh, not continuum accuracy.

`accepted-replay.gif`, `accepted-final.png/.pdf` and the convergence PNG/PDF
are rendered from recorded native accepted states. Regenerate with `render.py`
after qualification. No generated animation frame invents a physical time step.

`trials/` preserves the failed initial exact-rational mass assertion and earlier
successful baseline qualification, with its Rust source snapshots and SHA map.
Those are unfrozen development trials, not the final source qualification. The
failed assertion expected exactly 3.5625; the actual mass sum is
3.5624999999999996. Final tests require bitwise conservation of the accepted
sum and separately check its error against the exact rational reference. The
baseline precedes local physical-edge resolution guards and public liquid-volume
output. Its outputs retain their original schemas and logs; the final validator
expects the new schema. The final authoritative runs are the top-level
`rust-qualification/` and `native-qualification/`, not those trial directories.

All existing Jacobi fixtures, failed scale/accuracy evidence and conditional Lean
claims remain intact. No new Lean or continuum/free-surface pressure proof is
claimed. General finite relative flux/remap, pressure stability/solve, variable
density, adhesion and reconstructed surfaces are still separate unfinished gates.
