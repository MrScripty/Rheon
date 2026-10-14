# S1 implementation-worker handoff

Canonical plan: `docs/plans/cpu-compressible-solver/plan.md`.
Parent accepted independent review and admitted operation `start` for S1,
assigning worker `01a13913-999b-7511-b9cd-f4677019e018` at reviewed head
`67baa10576a2120f314ec8e56bd7b7129d18d9f0`. S1 is now `Active`; subsequent
worker execution continues that admitted slice after checking current authority.
This plan-only branch contains no implementation. Publication prerequisite is
satisfied at main `8baf459b91bb53ebd4a7230c4d9f3399dd5cb0bc`.

## Usable outcome and write boundary

A Rust host can construct an ideal-gas five-component state on a bounded fixed
Cartesian grid, request a periodic Euler step, observe only accepted fields and
actual dt/diagnostics, and execute a small headless reference with machine-readable
records. A separate verifier reads those actual native fields and compares
continuum acoustic/Sod references. This is the first native inviscid reference
slice, not a desktop/vehicle/learned/viscous capability.

Exact implementation-worker write set:

- `src/compressible.rs` — coherent numerical owner, EOS, state/scratch, errors,
  flux/update/CFL/acceptance and borrowed view/report.
- `src/lib.rs` — solely the minimal additive compressible module/export block;
  explicit parent delegation names this worker as its sole S1 editor. Recording
  and all other module/export blocks remain untouched.
- `tests/compressible_contract.rs` — public-path acceptance and rollback cases.
- `examples/compressible_reference.rs` — native uniform/acoustic/periodic-planar-Sod
  source and actual evolved field export to fresh ignored storage.
- `docs/api/compressible.md` — truthful units/domain/lifecycle/public API scope.
- `docs/plans/cpu-compressible-solver/reports/implementation.md` — worker's
  authored API/export proposal and implementation/check results.

Canonical `plan.md`, `issues.md`, `execution-ledger.md` and other current plan
decisions remain this planning agent's writes, updated from parent authority
and worker evidence. Worker owns only `reports/implementation.md` inside the
plan directory. Its export changes and public API tests stay together on its
implementation branch; parent integrates serially at R1. The prior parent-only
S1 export-application step is superseded by explicit sole-editor delegation.
The plan-only branch itself remains unchanged by the implementation worker.

Independent validation worker `01a13906-b6ec-7398-820a-387f852c3015` owns benchmark
harnesses and acceptance research, including proposed
`tools/verify_compressible.py` / `tools/test_compressible_verifier.py`. They are
read-only for this implementation worker. Parent coordinates the output
interface, gates and non-overlapping exact write sets before implementation;
use the validator's pinned reviewed revision rather than moving main. Focused
Rust API tests stay with the implementation worker. Separate validation is a
real independent owner, not a second copy of implementation logic.

Read-only context: published Ch26–28, Lean `Compressible.lean`, educational Python
reference/oracles, existing geometry/simulation/API patterns and workflows.
Forbidden: all other existing solver/session/provider modules, recording/storage,
desktop, Cargo manifests/locks, browser, Tuldok, proof source/pins/audits, source
inventories/receipts, Pages/publication gates and workflow files. No new dependency,
GPU/remote worker, merge/deployment or destructive history/checkout action.

Owner is the separate implementation worker; parent integrates serially. Use an
isolated branch/current checkout pinned to the admitted S1 base above; do not
consume moving main as a tested checkpoint. Recheck status and AGENTS.md/standards, PR69/71 heads and shared `src/lib.rs` before edits.
If new shared edits or unavailable capability change this boundary, report and
re-plan with parent; do not absorb recording/API/schema work as a shortcut.

## Binding behavior

Names are proposed until the worker records its exact public API in the owned
API doc; semantics are binding. Use validated GridGeometry metadata/x-fast
cell indexing, f64 rho/m/E, constant validated ideal-gas gamma and R, finite
SI primitive derivation, checked buffer sizing and one caller payload cap.
Expose construction, validated initialization, synchronous cooperative step,
borrowed accepted snapshot, report and allocated-bytes observation. Mutable
scratch must never escape; initialization failure preserves the last state.

Implement the serial unique-face periodic LLF unsplit forward-Euler update and
actual incident-face CFL from design. Piecewise-constant only. Initially C<=0.5
in the two-face-per-axis volume bound, which gives the usual directional 0.25
at uniform speeds. Check every candidate; halve retryable inadmissible/arithmetic
candidate attempts at most eight times from the same accepted state. No floor,
pressure tracer, quiet energy repair or change of sound speed. Preflight failures
and cancellation do not retry. Return typed invalid-parameter/state, allocation/
limit, arithmetic/EOS, cancellation, nonadvancing-time and retry-exhausted outcomes
with stage/cell/component as relevant. On error preserve all accepted arrays,
time, generation and last accepted report; attempted diagnostics are separate.

Report requested/accepted dt, summed acoustic rate/bound and controlling cell,
mass/three momenta/energy totals and ledger residual, rho/p/internal-energy/T
minima, max speed/Mach, attempts/rejections/fallbacks (zero flux fallback in S1),
method/EOS/geometry identity and managed capacity. Conservation reduces with
an explicit compensated sum; declare arithmetic allowance independently.
Wall-clock step/kernel timings are host observations, separate from simulated time.
Cooperative cancellation has defined checkpoints; no hard deadline/real-time claim.

Example output must include raw U and actual report/controls for each accepted
step/output, not only images or final summary. A narrow fixture-specific format
may use existing std writers and finite JSON/CSV with documented units/layout;
it is disposable verification output, not a dataset/recording schema. Bind
outputs to source/build/executable identity externally. Do not call the copied
report an independent verification result. No frames/logs committed.

## Deciding acceptance gates

Before implementing, record these criteria and exact fixtures; additional cases
must address a named reachable failure, not mirror the implementation.

| ID | Criterion and independent oracle | Domain / evidence |
| --- | --- | --- |
| S1-A | Primitive/conservative SI round trip, rho/p/T/c consistency, all-axis physical flux incl. pressure work, orientation reversal, shared cancellation | public Rust API; independently computed formulas; arithmetic error bound, not exact real equality |
| S1-B | Uniform state stays within declared binary64 roundoff; asymmetrical grid/origin/spacing; transverse momenta and periodic wraps incl. one/two-cell axes | focused/native actual fields; all five weighted totals, no transpose ambiguity |
| S1-C | Zero/negative/nonfinite EOS/state/dt, overflow/cap/allocation, clock nonadvance, failed candidate/exhaustion and cancellation leave accepted bits/time/generation/report unchanged | one invalid condition per otherwise valid setup, exact typed outcome; cancellation before/during candidate and retry |
| S1-D | Step agrees with educational LLF at identical input/dt after explicit x-fast↔z-fast mapping, conservative relative/absolute tolerances derived from operation roundoff | differential translation only; choose dt satisfying both bounds on asymmetric small grids, never use this as continuum accuracy |
| S1-E | Two nonparallel small acoustic modes, amplitude 1e-6, grids 8³/12³/20³, t=0.08: decreasing normalized density L1 errors, observed order>0.6, finest isolated phase-speed error<8%; disclose amplitude attenuation | reproduce research's bounded sanity gate with independent analytic wave projection; not finite-amplitude exact Euler or asymptotic/all-speed result |
| S1-F | Periodic planar Sod, gamma=1.4, jump x=.5, left (1,0,1), right (.125,0,.1), t=.08, shapes 24x2x2/48x2x2/64x2x2: decreasing density L1 and finest pressure L1<.04 in .2<x<.8 | initial jump aligned to faces; integrate exact conservative cell averages, derive reference primitives from them; bounds adopted as initial coarse gate, confirm tighter oracle uncertainty |
| S1-G | All accepted fixtures finite/admissible; normalized volume-integrated five-component drift <=2e-13 for these small periodic cases, scale=max(abs(initial total),1) in their declared unit system | independently recompute from exported native U at every retained step; disclose this fixture-sized tolerance, not arbitrary-scale guarantee |

S1-E/F numerical thresholds are initial published-reference sanity bounds for
the explicitly tiny cases. The worker must measure reference root/quadrature/
linearization uncertainty and report errors under the declared cell-average
convention, rather than copying center-sampled expectations. If those inputs
change the deciding meaning or make thresholds inappropriate, re-plan before
qualification; never tune criteria to observed output. No universal accuracy
percentage or benchmark speedup is imposed. Sod is planar extrusion; acoustic
modes exercise actual 3D variation. Open-tube t=.2 with momentum .18 belongs
to S3, not this periodic case.

Normal and `python3 -O` analytic-verifier negatives must reject frozen fields
with wrong evolved phase/time, axis swaps, missing pressure energy flux and
nonfinite data at their intended numerical boundary. Analytic accuracy cannot
identify arbitrary copying of correct fields from the same case. Origin checks
are separate: execute a fresh pinned native binary, retain an independent run
manifest binding source/tree/executable, exact actual controls/times and artifact
hashes, then reject a specifically mismatched declared run or artifact identity.
Changing a manifest alone does not prove which physics executed. Analytic
oracles never import the native flux or use current outputs as expectations;
readers may share layout helpers only. Validation owner supplies the origin
admission and numerical checks independently, coordinated through parent.

## Verification and delivery procedure

Use pinned toolchain and NumPy 2.3.5. Existing native-source gate will reject
new unadmitted solver/`src/lib.rs` inputs; record that expected publication
dependency and do not modify it. No Lean source changes means no new theorem
claim; run exact inventory check and appropriate affected checks.

Run `cargo fmt --all -- --check`, strict locked Clippy and contract tests for
core-only/default/parallel supported configurations. Existing rust-rheon CI
already covers src/tests/examples changes; inspect its actual coverage before
adding any workflow. Execute the release native example on a clean committed
candidate, independently verify fresh records, run normal/optimized Python
verifier tests and rerun affected existing geometry/core contracts. Desktop
or full Pages checks cannot substitute for these fields. Run wider tests only
for actual affected interfaces or newly discovered concerns.

Get independent implementation/standards review; collect findings and repair
within the valid scope. Store authored commands, scope/results/limitations and
identities in `reports/implementation.md`; planning agent records canonical ledger
and plan changes from parent authority and worker evidence. Large generated data/logs stay ignored.
Review exact staged/cumulative branch diff, commit/push own branch and create
a review PR. Parent owns source admission and integration with PR69/71; do not
merge or refresh their refs. Report commit, PR, actual test results, missing
environment observations, numerical limitations and any denial verbatim.

Stop once S1's usable native path and gates pass. Do not implement S2–S7,
Surrogate Lab, training, aircraft or submarine geometry in this worker.
