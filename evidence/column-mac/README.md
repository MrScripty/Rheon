# Fixed-flat MAC materialization qualification

Restricted prerequisite: accepted tangential column parcels are lifted into the
existing MAC owner, projected using matching liquid masses/fluxes, and atomically
published with held pressure and the liquid/geometry revisions. Export reads the
same accepted owner and reports omitted normal momentum/energy. Physical time,
phase fractions and heights are unchanged. Moving and viscous advances refuse
this mode. See `docs/research-book/implementation/flat-column-mac-bridge.md` for
equations, owner contracts, boundaries, memory and remaining dependencies.

`demo` contains 22 real native cases / 76 static publications and actual initial/
final velocity rasters. `tools/verify_column_mac.py` independently derives mass,
incidence, pressure, f32 stores, losses, phase mass, paired identities and pixels;
small cases also use an independent dense solve. Circulation refinement measures
instantaneous transfer error, not liquid time evolution. The figure reports
both its improvement and the growing stored-f32 divergence roundoff floor.

Qualification passes 194 default / 188 core / 200 desktop Rust tests, formatting
and strict Clippy in all three configurations; 34 Python tests pass in each
mode. Qualification retains the actual nine-command Rust matrix, normal and optimized
Python suite/oracle, pinned positive/negative/restored Lean build and axiom audit,
proof-source and education-PDF gates, and a fresh byte-identical replay of the
three original Jacobi fixtures and seven native example families. Fresh replay
outputs are produced under /tmp; the complete byte hashes and actual logs are
retained in `legacy-replay`. No old evidence is rewritten. `preservation.json`
binds every earlier evidence/proof/book/education file at the corrective base.

Thirteen finite conditional exact-real identities do not prove Rust execution,
IEEE correctness, PCG convergence, atomicity or moving-geometry simulation.
Bridge scale guards are checked; every internal PCG operation is not covered by
a corresponding scale proof. No pressure/divergence or accuracy bound is loosened.
The preserved original viscosity f32 refinement failure remains a failure.

    python3 tools/verify_column_mac.py evidence/column-mac/demo
    python3 evidence/column-mac/verify.py --negative-self-test
    python3 -O evidence/column-mac/verify.py --negative-self-test

Add `--evidence-commit COMMIT` to bind the root receipt and full tracked inventory.
Only the root receipt is omitted from its own hash map. Nested receipts are
included. The receipt specifies exact qualified source commit, tree, ordered
parents and source SHA-256s. Parent review and publication are separate.
