# PR12 off-facet hit reconstruction repair

Based on frozen surface candidate `777a7339029e94f8dd82187b3817481623e3ff3f`.
The coordinator's [review finding](https://github.com/MrScripty/Rheon/pull/12#discussion_r4179805077)
is reproduced from the exact original collision source: triangle z=1 and segment
(.25,.25,-1e20)→(.25,.25,1e20) report t=.5, world z=0, but barycentric z=1.
`exact-red-witness.rs` is compiled with the frozen collision module in a disposable
directory; the actual assertion exits 101. Source/witness identities and output
are recorded in exact-red-receipt.json/exact-red.log. The separate Cargo red
regression also actually exits 101, exposing the axis-permuted false hit.

The 2D barycentric solve drops the dominant normal axis to avoid Gram loss. Its
containment result alone cannot validate the ignored plane coordinate after
segment interpolation has lost a small offset. Immediately before accepting a
candidate, compute q=(position-origin)/facet_scale and r=u*e1+v*e2 using the
already normalized facet edges. Require max_axis |q-r| <= relative_tolerance;
otherwise return the existing AmbiguousIntersection error without a partial hit.
The tolerance is facet-relative, not widened by the enormous segment length or
global coordinates. No exact/adaptive predicate or tolerance redesign is added.

One new Rust contract method tests six off-facet cases across three axes and both
plane-offset signs. Three huge-segment contacts on the origin plane remain valid
and retain t=.5/weights (.5,.25,.25), so travel magnitude alone does not reject.
Actual default/core/desktop totals are 63/58/69 tests; strict Clippy and formatting
pass all feature configurations. The exact release CLI replays all three original
Jacobi PNG/CSV fixtures byte-identically; original non-timing manifest fields agree.
The historical proof-source and PDF/input-byte gates pass. Existing book, proofs,
pressure/force/advection implementations and historical source-bound receipts
remain unchanged. The existing finite-facet Lean statements remain conditional
on correct candidates and do not certify IEEE reconstruction; no new proof claim
or complete historical root rebuild is made.

Reproduce the production checks with the established locked/offline all-target
feature suites, strict Clippy and replay commands. Reproduce the exact red witness
by writing `git show 777a7339029e94f8dd82187b3817481623e3ff3f:src/collision.rs` to
collision.rs beside a copied exact-red-witness.rs in a disposable directory,
compiling the witness with `rustc --edition=2024`, and running it. The witnessed
module is byte-identical to the frozen Git blob; it is not copied back into the
checkout. Rust/Cargo 1.92.0 and LLVM 21.1.3; dev tests/unoptimized witness and ordinary
optimized release CLI, one Cargo build job. Cloud VM exposes five Intel Xeon
Platinum 8370C CPUs; tests/replay and unrelated preserved feature work overlap.
No isolated timing benchmark or continuum claim is made.

Frozen milestones are composed with this exact shared repair on new integration
branches. Their original heads stay untouched; ordered parents and per-composition
qualification are recorded separately. The unfinished prescribed-box-flux work is
preserved. Coordinator owns PR updates, review and main integration.
