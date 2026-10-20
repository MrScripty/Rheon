# CV2 bounded conditioning/coupled-solve discovery

Explicit delegation authorizes continuing this gate after CV1, with original
fixtures/criteria and independent review. Currentmain718469b fetched safely;
isolated branch implementation/q2-cut-coupled-solve-20261020 at4f8bd22 (PR135).
CV1 observed square cond(M)=6.5331614839e11 remains immutable. It is neither
cond(A=M/dt+K) nor cond(KKT), and supplies no new pass/fail threshold.

Intended outcome: smallest native static planar Stokes-substep solver reference
with actual solution-error evidence. No production provider, transport, UI,
recording, source-admission, generic inf-sup or PDE-convergence implementation.
No essential user/physical input missing for these prescribed synthetic problems.

Exact write set BEFORE editing numerical implementation: NEW
examples/grid_cut_coupled_reference.rs, private modules under
examples/grid_cut_coupled_reference/, tests/grid_cut_coupled_reference_contract.rs,
and docs/plans/fluid-accuracy-composition/q2-cut-solve/*. Existing CV1 geometry,
operators, trace/export and all prior evidence are read-only. Parent owns root
plan lifecycle/source admission and all merges; no cleanup/rebase/merge.

Alternatives compared on the same original finite equations:
A. Existing-style unscaled partial-pivot dense KKT: smallest direct comparator,
   no normalized-block conditioning benefit; retain as diagnostic, not fallback.
B. SPD diagonal congruence, velocity Cholesky and pressure Schur Gram factor:
   chosen proposed experiment. Positive pivots only, no floors/deletions/gauges;
   original-equation compensated residual refinement and back-transformation.
C. Full exact A/Schur whitening: ideal bounded spectrum in exact arithmetic, but
   factor/assembly and back-transform errors remain; dense exact work not a
   scalable production preconditioner. B supplies these factor actions already.
D. Higher precision alone: useful independent solve oracle, cannot repair rounded
   operator/load perturbations. Do not mistake exported-block accuracy for exact
   basis-integral or PDE accuracy.

Native corpus/solver protocol remains prospective until independent review.
Standards0fc92a2: core/router; planning/discovery, implementation/verification/
oracles/proportionality/commit/build, Rust profile and directly affected API/tooling
routes. Private single experiment owner adds no production composition boundary;
composed-design review is not-applicable. Independent read-only evaluator adds
needed deciding value: residual alone cannot decide actual solution accuracy.
A separate visual agent maintains provisional local method-map.html under the
new research directory; primary mathematical review has separate evidence owner.
