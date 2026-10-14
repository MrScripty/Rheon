# S1 native implementation — diagnosed numerical qualification failure

The native periodic ideal-gas Euler path, public API, 13 focused contract tests,
eight-case actual-field producer and API documentation are implemented. **S1
acceptance is pending.** Actual fields expose a failed acoustic coarse-grid order
gate and an independent Sod initial-oracle roundoff issue. No threshold, CFL,
EOS or acceptance criterion was tuned to observed output. Canonical lifecycle
remains the planning worker's responsibility; this report owns only S1 execution.
Continued diagnosis found that the intended LLF operator predicts the coarse
acoustic failure. An independently reviewed resolution-domain correction is
proposed below and remains unapplied. Origin capture is now available at the
recorded pin; full numerical assessment still fails.

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

## Initial committed capture attempts and delivery (historical)

Reviewed native candidate commit:
`ca8c9361b7725993e080c92b0657fdb53ecd11d8`, tree
`3768af8702894133a9222ab0211f362795c5860f`. Source checkout was clean before
attempting the pinned independent capture. The capture commands ran from the
separate verifier checkout at `4621d69747740a950a90d78755fe3bb4e220efe4`:

```
python3 tools/verify_compressible.py capture --source-repo /workspace/Rheon \
  --native-commit ca8c9361b7725993e080c92b0657fdb53ecd11d8 \
  --output /tmp/rheon-s1-native-ca8c936
```

Outcome: **refused before any output creation**, exact diagnostic
`S1 validation failed: origin/output-inside-checkout`. `/tmp/.git` is an empty
read-only environment sentinel with no HEAD; the pinned verifier tests `.git`
existence rather than a real checkout. `/workspace/.git` has the same sentinel.
Neither directory is an actual ancestor checkout of the proposed fresh run.
This is an independent-validator environment/path-admission blocker. No sentinel
or verifier file was altered and no weaker origin check was substituted.

A fresh external-storage alternative was attempted with a narrowly granted
additional write path `/var/tmp/rheon-s1-native-ca8c936`. It failed before output
creation: `S1 validation failed: [Errno 30] Read-only file system:
'/var/tmp/rheon-s1-native-ca8c936'`. This is an environment capability failure,
not an automatic approval denial. `/dev/shm` is also mounted read-only. No
origin manifest exists, and no source/executable/artifact origin pin is claimed.
The exact operation is pending a validator-owned compatible path check or a
writable external storage environment; the candidate/fields and reviewed refs
are preserved. Numerical S1 qualification still separately requires disposition
of the Sod initial-oracle arithmetic and acoustic order failure above.

No auto-review approval rejection occurred. The failed capture outcome is known
not to have created the output; no uncertain remote mutation is pending.


## Continued diagnosis at frozen settings

Parent requested continued S1 diagnosis after draft delivery. The numerical source,
public API, production fixture convention, CFL=.5, amplitude=1e-6, end t=.08 and
all acceptance thresholds are unchanged. No solver defect was identified; no S2
method or new feature was added. This section supersedes the earlier environment
capture blocker only; both original numerical qualification findings remain open.

### Native/reference/symbol agreement

An independent GPT-6.1 Sol High reviewer reran the separate educational LLF at
**each actual native accepted dt**, starting from the same exported linear
conservative cell averages on 8³/12³/20³. Every retained step matches native U
within `4.440892098500626e-16`; actual dt satisfies the educational bound within
its stated roundoff. This is translation evidence, not continuum acceptance.

Separately derived the linearized shared-face LLF Fourier symbol at rho=p=1.
For a grid mode theta_d=2*pi*k_d*h_d, define

```
q_d = sin(theta_d)/h_d
D = c * sum_d((1-cos(theta_d))/h_d)
lambda_plus/minus = 1 - dt*D -/+ i*dt*c*|q|
a = dot(k/|k|, q/|q|)
rho_mode_multiplier = (1+a)/2 * product(lambda_plus)
                    + (1-a)/2 * product(lambda_minus)
```

Here c=sqrt(gamma), and the initial conservative eigenvector aligns with the
physical k, while the discrete acoustic eigenvector aligns with q. The second
mode therefore includes a small reverse-going component on coarse grids. Each
cell-average initial mode is also multiplied by the exact sinc product.
This symbol is independent of native flux code and follows from linearizing the
public conservative Euler equations, centered flux and LLF difference term.
Actual native dt sequences were used; nonlinear local alpha/flux changes are
O(epsilon²). Symbol density differs from actual native density by at most
`2.89e-13`, `4.03e-13`, `5.84e-13` on the deciding 8³/12³/20³ cases.
Symbol-predicted normalized errors are .31274293827, .24907096311, .17123876907;
its 8→12 order .561438498 also fails >.6. Thus a failure of the intended linearized
operator already explains the observed gate; changing native arithmetic cannot
repair that coarse-order incompatibility honestly.

Native refinement extended through 64³ at identical physical/control settings,
using a disposable Rust driver through the public API. It allocated no new solver
machinery and exported final actual U plus every accepted dt into ignored
`.generated/s1-refinement-v1`. Separate educational same-dt integration through
32³ agrees with native U within 4.44e-16. Additional grids are diagnostic evidence,
not changes to the current producer suite or accepted fixture population.

| n | Accepted steps | Native normalized density L1 | Linear-symbol L1 | Order from previous n |
| --- | --- | --- | --- | --- |
| 8 | 10 | .31274301614224453 | .312742938268773 | — |
| 12 | 14 | .2490710346665802 | .24907096311347726 | .5614384036983551 |
| 20 | 23 | .1712388220900807 | .17123876906860458 | .7334771702681024 |
| 24 | 28 | .14753314663362563 | .14753310085138188 | .8172721410031405 |
| 32 | 37 | .11529682712965958 | .11529679204132313 | .8569980147372906 |
| 48 | 55 | .08006258293338685 | .08006255931339802 | .8994640616396221 |
| 64 | 73 | .061275893184887666 | .06127587550249425 | .9295751635361987 |

The reviewer independently recomputed all added-grid errors and confirmed the
trend toward first-order behavior. The unmodified original published **center-
sampled primitive** educational fixture, compared with its original center-
sampled analytic density, gives errors .34512974829079, .2606703680469508,
.17418286495263227 and orders .6922020447013932, .7892144350823597. Its transferred
>.6 coarse gate passes in its original convention. With the admitted conservative
cell-average convention, sinc factors reduce the n=8 signals to .9254209450 and
.8549774701 for the two modes, while n=12 factors are .9662351042 and .9333114415.
Dividing errors by unchanged continuous epsilon therefore lowers the coarse-grid
error disproportionately. LLF damping is also substantial (n=8 mode amplitude
ratios .529/.296), so these are nonasymptotic coarse observations. This explains
the convention-sensitive order without a floor, phase correction or CFL tuning.

Preserved diagnostic scripts/data hashes (all artifacts remain ignored):
`diagnosis.json`: `2c0df47e2aee9f897e0fb66a204b6c5e4cd85466a0ada0e013adcf42ae63848b`;
public-API Rust driver: `c5f031137f2bf8922e60c5915767badb22e7672340052b241f43b44d3510da20`;
independent educational/symbol analysis script:
`f9559329669e386a04c95b53c1e02ddd05bf64ee567e72ded89b0e3e3d498a3d`.
They are reproducibility/diagnosis artifacts, not newly retained permanent verifiers.

### Proposed correction — requires parent/plan admission

Keep the existing 8³/12³/20³ observations and mark the original >.6 adjacent coarse
order gate failed under the newly admitted cell-average convention. Do not erase
that history, lower its threshold, normalize away damping, change CFL or return to
center-sampled production initial data. For a deliberately first-order method's
**convergence** acceptance, a justified proposal is to add 32³/48³ resolution
and apply the unchanged >.6 order requirement to 20³→32³ and 32³→48³, retaining
all existing monotonic-density, original finest mode-(1,1,1) speed, attenuation,
admissibility/conservation and Sod gates. These finer diagnostic orders are
approximately .8416 and .8995 and independently predicted by the linear symbol.
The purpose is to test an asymptotic first-order claim on better-resolved modes,
not make the original coarse gate pass. This proposal is NOT applied or accepted.

Planning owner must disposition S1-E's current acceptance meaning and write the
canonical correction before producer/validator case populations change. Validation
owner must independently freeze/review the corrected criterion and its negatives;
this worker would then update only the admitted native example/API/tests/report.
The extra all-step exports have material but bounded cost: at 48³ solver payload
is 14,155,776 bytes, and adding 32³/48³ remains within the current 2GB conservative
raw-output cap. Using 64³ as a required all-step case would exceed that existing
cap and is intentionally not proposed. No expanded permanent harness or S2 upgrade
is justified by this finding. Parent owns checkpoint acceptance.

### Fresh committed origin capture now available; full assessment still fails

Validation owner independently pushed revised capture protocol
`aea43a54d7c8e51b897b21f7e8119c2a22cd1605`: real-checkout detection handles empty
sentinels, and capture builds a filtered tracked-only Git archive with build-input
checks. This worker read the exact diff and used it unmodified in the separate
verifier checkout. Correctly sourced pinned Rust environment passes all **23**
verifier tests normally and under `python -O`. An earlier invocation omitted the
setup PATH and its real-capture test reported missing Cargo; corrected environment
reruns passed. No failed test is counted as a pass or verifier code edited here.

```
python3 tools/verify_compressible.py capture --source-repo /workspace/Rheon \
  --native-commit 1b62ae378c2aaa01327055f121ebc0d869f2c4ac \
  --output /workspace/scratch/rheon-s1-native-1b62ae3
```

This fresh writable workspace directory is outside actual Git checkouts. The
clean pinned source was archived/built with locked Rust 1.92 core-only, and the
native binary ran to completion. Source, build/run logs, every field, suite,
executable and failed assessment are preserved. Recorded subjects:

- Native source commit: `1b62ae378c2aaa01327055f121ebc0d869f2c4ac`.
- Native tree: `cbb635a89bba4927b16835444f882dd6c5751d3c`.
- Verifier revision: `aea43a54d7c8e51b897b21f7e8119c2a22cd1605`.
- External origin-manifest SHA256:
  `c6d76dd06781929d5d7466a2a57d88680fcb8fc5181fb92bb0731aa1a85200b8`.
- Executable SHA256:
  `fcc27114908e627149c2c6d090a222772750d8b4408895a5102a2bffc90ae7eb`.
- Bound field/suite artifacts: 143 (142 frames plus suite).

The command returns failure **`sod/initial-cell-average`** after successful
origin capture; `assessment-failed.json` records that exact failure and manifest
pin. A subsequent `verify` using the external origin pin fails at the same
numerical boundary. Origin capture is therefore available, while full numerical
acceptance remains failed. The earlier path-admission/storage blocker is resolved
by an independently owned verifier repair; no filesystem restriction was bypassed.
No native source edits were made during capture or this continued diagnosis.

The independent reviewer assessed the correction's mathematical/claim boundary:
mode-two maximum grid phase is pi/2 at n=8 versus .6283 at n=20, and its
semidiscrete diffusion D*t is 1.20085 versus .5469. This supports changing the
resolution domain for an asymptotic-order claim, not a native algorithm repair.
The reviewer also independently confirmed the Sod initial-oracle defect and
recommended exact piecewise-constant zero-time cell integration at the validator
owner; altering native fields or loosening its 16-epsilon allowance is unnecessary.
Neither proposal was applied here. All prior failed fields/assessments remain.

### PR74 CI observation at native/report pin 1b62ae3

GitHub Actions metadata confirms Pages run `38524774035`, job `115722429365`,
fails specifically at **Check reviewed native admission before expensive
qualification**; all subsequent build/qualify/deploy steps are skipped. This is
the expected separately owned source-admission dependency, not S1 numerical
qualification. Local `source_core` refusal was recorded above. Log download
returned HTTP403; the failing stage is confirmed by the job metadata rather than
an invented log traceback. No gate, manifest or workflow was edited.

At this observation, Rust desktop feature contracts and executable contracts
passed (run `38524774026`). Cloud acceptance, real pressure checkpoint native
CPU, VTK roundtrip, moving-plate coupling, aligned native/rational and conditional
kernel checks, reconstructed isolated strain, dense3D scoped metadata and bounded
lessons also pass. Default/core-only feature contracts remain in progress; their
completion is not claimed. Subsequent documentation-only delivery does not change
the tested native numerical/API/fixture input hashes. Parent owns final integration
and admission of a qualified checkpoint; no PR is merged or site deployed here.
