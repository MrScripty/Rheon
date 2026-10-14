# S1 native implementation — verification blocked on numerical qualification

The native periodic ideal-gas Euler path, public API, 13 focused contract tests,
eight-case actual-field producer and API documentation are implemented. **S1
acceptance is pending.** Actual fields expose a failed acoustic coarse-grid order
gate and an independent Sod initial-oracle roundoff issue. No threshold, CFL,
EOS or acceptance criterion was tuned to observed output. Canonical lifecycle
remains the planning worker's responsibility; this report owns only S1 execution.

## Admission, branch and ownership

Started from exact reviewed PR72 head
`67baa10576a2120f314ec8e56bd7b7129d18d9f0`, whose merge base with then-current main
was published `8baf459b91bb53ebd4a7230c4d9f3399dd5cb0bc`. Created
`implementation/cpu-compressible-s1` preserving `work` and
`implementation/rust-smoke-milestone` refs. Consumed parent-admitted reviewed plan
merge `919a8fde4bb10239055ae6fea3f4e6359a7c5c58` by fast-forward, without reset,
rebase, rewriting or touching ongoing code. Relative to the admitted plan head,
that update changes only canonical plan/handoff/ledger/verification documents.
Target is main; parent integrates; branch and checkout remain retained for review
until parent disposition. No merge/deployment/publication is performed here.

The write set is exactly `src/compressible.rs`, `tests/compressible_contract.rs`,
`examples/compressible_reference.rs`, `docs/api/compressible.md`, this report and
the solely delegated additive `src/lib.rs` export. No recording export, existing
solver, lockfile/dependency, workflow, source admission, Lean, desktop or dataset
schema change is included. PR69/71 heads were rechecked read-only as
`b1acf2e514b88b528cf43e873035a598a1080ae1` and
`ecfc741af733f909c79170530080a98fdc5c78eb`. No AGENTS.md/repository-local skill was
found in checkout/ancestors. Routed standards are source-guided Coding-Standards
`188beda1fa477d21d576c233dd8c7f4c4c267d23` Core/Router, Implementation,
Verification/Oracles, Proportionality, Commit, Documentation, Rust/API, Library,
Contracts/evolution, Architecture/Code Design, Resilience, Diagnostics,
Performance, Security and Dependencies. No Standards Engine certificate is claimed.
The plan already owns the bounded numerical implementation-versus-adoption choice.

## Exact API and output

[API documentation](../../../api/compressible.md) owns SI values, x-fast layout,
validation, lifecycle, errors and payload accounting. `IdealGas` owns EOS and
primitive/conservative/physical-flux conversion. `CompressibleSimulation::new`
constructs a uniform state; `from_fn` constructs nonuniform validated averages
at generation/time zero. Transactional `initialize`/`initialize_with`, synchronous
`step(requested_dt,cancel)`, immutable `view()` and `allocated_bytes()` expose
accepted state/report without mutable scratch. One coherent owner contains unique
periodic faces, incident CFL, candidate/retry/rollback and compensated budgets.
The example owns fixed fixture controls and disposable writers; validation owns
continuum/origin gates. Removing the solver owner would distribute its necessary
thermodynamic/face/publication reasoning into callers. No generic flux registry,
framework, recording adapter or model seam was added.

Exact additive export, leaving all prior blocks unchanged:

```rust
mod compressible;
pub use compressible::{
    COMPRESSIBLE_METHOD, CompressibleCause, CompressibleCheckpoint, CompressibleDiagnostics,
    CompressibleError, CompressibleFailure, CompressiblePrimitive, CompressibleSimulation,
    CompressibleStage, CompressibleStepReport, CompressibleView, ConservativeState, IdealGas,
};
```

Parent admitted independent validation PR73 revision
`4621d69747740a950a90d78755fe3bb4e220efe4` and its
`docs/plans/cpu-cfd-validation/reports/s1-export-proposal.md`. The final producer
aligns exactly: `compressible_reference --output FRESH_DIRECTORY`, eight fixed
uniform/differential/acoustic/Sod fixtures, `suite.json` published last,
`schema_version=1`, `method=llf-euler-periodic-v1`, SI EOS/geometry/controls and
**every** accepted frame. CSV columns are exactly `rho,mx,my,mz,E`, x-fast,
round-trip f64. Additional frame metadata preserves actual diagnostics/report,
rejections/last cause, kernel/step host times and managed capacity. No runtime
bundle or persistent production schema is introduced. The initial provisional
primitive-wave/quadrature output was replaced with the explicitly admitted
linear conservative analytic cell averages before qualification; the numerical
method and frozen criteria remained unchanged.

## Focused checks and independent implementation review

Pinned Rust 1.92.0 via `/workspace/rheon-setup/env.sh`, NumPy 2.3.5 already available.
Locked dependencies are unchanged. Commands run in the native checkout:

```
cargo fmt --all -- --check
cargo clippy --locked --no-default-features --all-targets -- -D warnings
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --no-default-features --features parallel --all-targets -- -D warnings
cargo test --locked --no-default-features --test compressible_contract --test geometry_contract --test simulation_contract
cargo test --locked --test compressible_contract --test geometry_contract --test simulation_contract
cargo test --locked --no-default-features --features parallel --test compressible_contract --test geometry_contract --test simulation_contract
python3 proofs/scripts/check_sources.py
python3 -O proofs/scripts/check_sources.py
```

All selected formatting/lint configurations passed. Compressible tests pass
13/13 in all three configurations, with 7/7 existing geometry and 9/9 simulation
contracts passing per configuration. The final added constructor regression was
rerun in all three modes; unchanged existing tests were not rerun unnecessarily.
An initial attempted target `core_contract` did not exist; corrected to the actual
`simulation_contract` target. No missing-target failure is counted as a pass.
Lean exact source/pin inventory passes normally/optimized; no source changes or
new theorem/kernel claim. The existing Rust CI covers src/tests/examples and
includes desktop checks; no workflow change is needed. Desktop/all-Pages and all
unrelated contracts are outside this focused evidence.

Independent GPT-6.1 Sol High reviewer `/root/s1_review` found a P2 retry error:
raw flux/divergence arithmetic cannot be fixed by halving dt. Repaired to direct
`Arithmetic`, with rollback regression. Reviewer independently found actual
finite high-Mach conditioning fixtures for successful retries, nine-attempt
exhaustion and cancellation during retry; no public test injection was added.
Their final review found no outstanding implementation defect and independently
reran all 13 core-only contracts. They parsed a fresh release suite with 142
frames/241,596 U rows, checked layout, generations, admissibility, controls, time,
report/capacity/attempt accounting and compensated budgets, and independently
integrated the acoustic initial data (component maximum discrepancies no larger
than 4.44e-16). Those review observations are not clean-source origin qualification.
Final reviewed numerical-source SHA256:
`dc0f2a3a8ebcb108c6a094ec59a467f1390b846dd27c505b16248a64dc1031b3`.
Reviewed example SHA256:
`0fea8ddb7dac844f86454df7b750537fffa5e99faa64ab252fe2d37fb5fd6ad0`.
The reviewer’s sole final documentation sentence-fragment finding was repaired.

## Actual-field observations and blockers

Ran the release core-only producer into fresh ignored
`.generated/s1-precommit-fields`, preserving all results. Validator files were
never edited: a separate checkout is pinned at `4621d697`. Both
`python3 -m unittest discover -s tools -p test_compressible_verifier.py -v` and
its `python3 -O` equivalent pass all 19 policy/oracle tests. These synthetic
policy checks are supporting evidence, not native acceptance.

The unmodified independent `assess_suite` rejects **`sod/initial-cell-average`**.
Native Sod-48 has exact initialized E=`2.5000000000000004`; the quadrature
oracle returns `2.5000000000000053` in cell 13, a `4.884981308350689e-15`
difference exceeding its `16*epsilon=3.552713678800501e-15` allowance. Native
initialization is the stipulated exact face-aligned constant conservative state.
Validator owner must disposition this initial-oracle arithmetic issue. No native
values were adjusted to reproduce oracle roundoff.

Separate diagnostic calls to the pinned verifier's existing functions on actual
fields (without modifying or bypassing `assess_suite`) observe:

| Quantity | 8³ / 24x2x2 | 12³ / 48x2x2 | 20³ / 64x2x2 |
| --- | --- | --- | --- |
| Acoustic normalized density L1 | .31274301614224453 | .2490710346665802 | .1712388220900807 |
| Acoustic phase error, mode (1,1,1) | .03303911615393751 | .014333542288840762 | .004958830271189152 |
| Acoustic amplitude ratio, mode (1,1,1) | .529143494065101 | .6524601719162378 | .7735667526483967 |
| Acoustic phase error, mode (1,-1,2) | .16224298820432215 | .0730523062672126 | .02620621763305564 |
| Acoustic amplitude ratio, mode (1,-1,2) | .29590075351392137 | .4328852507144175 | .6005889963102716 |
| Sod density L1 | .038436133232651666 | .028884473581600432 | .02823249991117378 |
| Sod pressure L1 | .05448703952865247 | .04271663871527951 | .037306094143767975 |

Acoustic observed orders are **.5614384036983551**, .7334771702681024. The first
fails the frozen `>0.6` requirement. Errors decrease and finest phase errors are
below 8%, but those do not replace the failed order gate. Parent/planning and
validator owners must determine whether the newly admitted cell-average convention
changes the deciding coarse-grid meaning; qualification stops pending that decision.
No center-sampled expectation, method/control tuning or threshold relaxation was used.

The one-step asymmetric differential maximum is `5.551115123125783e-17` within
the validator-owned gamma_512 roundoff model. Actual five-component normalized
periodic drift over all 134 accepted steps is at most `1.6148698540002275e-16`
versus the frozen `2e-13` gate. Sod density errors decrease and finest pressure
L1 is below .04; split 8/16-point conservative quadrature differs by at most
`1.3322676295501878e-15`. These individual observations do not promote the
failing full suite to acceptance. The validator owns root/quadrature/linearization
uncertainty and origin admission; the producer does not copy oracle evolution.

Clean-source capture, final native commit/PR and resulting origin pins are recorded
in the delivery follow-up below after the implementation is committed. The expected
native-source admission failure is a parent/publication dependency: direct
`aligned_strain_packet.source_core` rejects with `Refusal: reviewed native source
inventory added or missing path`. No inventory,
source receipt or gate is updated here. The existing NumPy runtime is used only
for validation, with no packaging, runtime distribution or GPL-free claim.

Remaining limits: first-order diffusion/attenuation; tiny periodic fixtures only;
no positivity theorem, arbitrary-scale conservation or unrestricted Mach claim;
no OOM-killer, hard cancellation latency, speedup or physical vehicle qualification.
No acceptance state is concealed by this report or by a draft PR.
