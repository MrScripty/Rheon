# Independent CPU CFD validation

**Plan status:** `Active`
**Current phase:** reviewed plan/native pins received; explicit capture forwarding
**Next slice:** review/commit capture selector, then fresh actual prospective capture and pinned verification
**Acceptance status:** `blocked`
**Execution ledger:** [execution-ledger.md](execution-ledger.md)
**Issues:** [issues.md](issues.md)

## Objective and scope

Provide independently derived, executable CPU pressure and viscosity refinement
studies with exact-source receipts for future Tuldok dataset/model acceptance.
Own `validation/cpu-cfd/**`, `docs/plans/cpu-cfd-validation/**`, and, per parent
steering at reviewed plan 67baa105, `tools/verify_compressible.py` and
`tools/test_compressible_verifier.py`.
Do not edit solver cores, dependency declarations in the root, research-book
sources, or other workers' plans. Passenger-jet and ascending-submarine acceptance
is a staged research destination, not this slice's advertised capability.

## Objective acceptance

| ID | Observable criterion | Kind | Environment | Mode | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| A1 | Real Rust pressure/viscosity APIs produce decreasing continuum mesh errors and independently measured conservation/boundary diagnostics | integration | local CPU Linux, Rust 1.92 | automated | satisfied | reports/validation-results.md |
| A2 | Fixed-mesh time study approaches its independent semidiscrete reference; negative metrics fail acceptance | focused | local Python 3 and Rust 1.92 | automated | satisfied | reports/validation-results.md |
| A3 | Completion receipt binds source files, toolchain, inputs, command, rows and gates; a failed run cannot acquire an accepted receipt | contract | local fresh process | automated | satisfied | clean ef4f8e4 receipt; runner tests |
| A4 | Staged aircraft/submarine gates, solver requirements, primary sources and access/licensing dispositions are reviewable without imported datasets | focused | repository and primary sites | manual | satisfied | reports/staged-validation.md |
| A5 | Independent S1 cell-average oracles and intended numerical/origin negatives run normally and optimized | focused + contract | Python 3, NumPy 2.3.5, Rust 1.92 for protocol probe | automated | satisfied | 43 normal + 43 optimized across both studies; independent findings closed; reports/independent-review.md |
| A6 | Real clean pinned native fields pass unchanged S1-D/E/F/G checks with independent origin capture | integration + system | native CPU producer | automated | failed | native capture executed; analytic Sod repair preserves tolerance; frozen acoustic order 0.5614384 fails; reports/native-s1-diagnosis.md |
| A7 | Explicit versioned prospective study retains original population and unchanged failure gates | focused + contract | Python 3, NumPy 2.3.5 | automated | satisfied | 41 normal/-O tests; reports/prospective-study.md |
| A8 | Actual prospective native fields pass all unchanged gates under the admitted revised plan and source pins | integration + system | native CPU producer | automated | pending | parent admits plan fc2270a85e38080d9943048aaf689d99545d3715 and native 5a049ed604b99817a428d10bbc0140732d3421d6; fresh capture pending |

## Constraints and assumptions

User authorizes implementation, testing, commit, push and draft PR; parent owns
integration and compatible checkpoint/release tags. Original saved checkout is
clean at 6ba41c7 and preserved. Admitted base is current origin/main
8baf459b91bb53ebd4a7230c4d9f3399dd5cb0bc. No AGENTS.md or .agents/skills were
found in workspace or Rheon. Coding-Standards read at
188beda1fa477d21d576c233dd8c7f4c4c267d23: Core, Router, planning/discovery,
implementation, verification/oracles, proportionality, commit, documentation,
tooling, code-design, architecture, dependencies/licensing and Rust/tooling/
dependency profiles. This is a bounded engineering decision from already
explicit task scope, not substantial product discovery.

The public CPU APIs are assumed adequate for elementary fixtures only; compilation
and a real executable run must establish this before acceptance. Unsupported
body-load and moving-domain capabilities remain deferred. External hosting does
not establish redistribution or model-training rights; no third-party data is
incorporated. New direct dependency is only the existing Apache-2.0 Rheon path
crate with default features disabled; reuse Cargo and Python standard-library
CSV/JSON/hash/process facilities rather than adding a scientific runtime.

## Binding decisions and owners

| Decision | Owner | Evidence | Supersedes |
| --- | --- | --- | --- |
| Validation calls public CPU APIs; no copied solver implementation | validation worker | src/pressure.rs, src/viscosity.rs | none |
| Continuum analytic RHS is derived independently, never formed by applying the production pressure matrix to the expected solution | validation worker | analytic equations in boundary README | none |
| Separate mesh error, time error, experimental agreement and label readiness | validation worker; future dataset owner | existing book ch. 12/18 and requested scope | none |
| Hold body-force accuracy and aircraft/submarine claims until their capability and licensing gates pass | solver owner via parent | broader-physics-milestone pressure-load counterexample | none |
| Build a tracked-only source export; reject unsupported Cargo config/compiler overrides for native capture | validation worker | independent review R1/R2/R4 | building directly from the caller's clean checkout |

## Evidence and oracle admission

Existing contract tests exercise exact stencils and many bounded mechanisms, but
do not supply one cold-process refinement receipt with source/command identity.
The reachable failure is promoting a small algebra test or old receipt into a
physical/training acceptance claim. A small independent analytic fixture and
fail-closed result evaluator provide deciding value; retain while datasets need
these elementary gates, replace if a maintained acceptance system subsumes them.
Hashes bind precise source/output bytes for provenance, not mathematical truth.
No cryptographic signature or tamper-proof storage claim is made.

| Claim | Domain | Deciding oracle | Authority | Unsupported domain | Negative failure |
| --- | --- | --- | --- | --- | --- |
| Pressure mesh order | sealed Cartesian continuum Poisson | cosine solution and continuum Laplacian | explicit differentiation | exterior aerodynamics | wrong pressure sign/scaling gives error/order failure |
| Viscosity mesh/time error | free-slip divergence-free decay | analytic exponential; separately labelled semidiscrete exponential for time | explicit differentiation/eigenvalue | full nonlinear Navier–Stokes | zero decay, reversed slope or wrong boundary rejected |
| Receipt completeness | fixed elementary suite | exact expected row keys, finite metrics, declared thresholds | evaluator tests with mutations | arbitrary solver result formats | missing, duplicate, NaN, corrupted conservation/time rows rejected |

## Simplicity and ownership review

**Applicability:** `applicable`

- Independent concerns/dimensions: Rust measures numerical behavior; Python owns
  gate policy and run receipts; report owns external-case feasibility. Each has
  its own change reason and no production authority.
- State/identity/value/time/policy/mechanism: numerical buffers live inside one
  benchmark; receipt references Git revision plus input hashes and dirty paths.
  Schema v1 owns only this CSV/receipt format. No legacy schema or external
  consumer overlap exists; changing columns updates producer, evaluator/tests.
- Callers/root knowledge: one Python command invokes a fixed bounded Cargo
  binary, creates a new directory, and returns pass/fail. No solver registry.
- Representative changes: changing numerical reference affects Rust + README;
  changing acceptance budget affects evaluator + negative tests; external data
  access affects only report. Adding capabilities requires a new admitted slice.
- Stable interfaces: public Rheon APIs carry geometry, units, explicit failure;
  CSV carries named measurements. No private indices or mutable solver state.
- Independent evolution/failure/replacement: binary failure is saved as failed;
  evaluator failure retains raw rows without accepted completion. Existing
  solver tests remain independently executable.
- Deletion result: deleting Rust loses public-path measurements; deleting runner
  scatters provenance and pass/fail interpretation across operators; deleting
  report loses rights/capability gates. No new generic adapter/framework needed.
- Inherent complexity: independent spatial/time references and claim levels
  are necessary; contained in one validation boundary and one staged report.

## Milestone 1

**Goal:** reusable elementary benchmark, tested acceptance evaluator and staged plan.
**Allowed write set:** `validation/cpu-cfd/**`, `docs/plans/cpu-cfd-validation/**`.
**Tasks:** implement cosine pressure and viscous-decay studies; record independent
flux/wall/energy diagnostics; test evaluator failures and run lifecycle; document
forces/moments oracles and next physical cases; execute format, strict Clippy,
Rust/Python tests and a release receipt; inspect staged/cumulative diff; push draft PR.
**Gate:** A1–A4 evidence plus static checks. **Status:** `Accepted`; clean ef4f8e4 receipt observed.

## Milestone 2: S1 independent native validation

Parent steering prioritizes S1-A through S1-G in reviewed solver plan
67baa105. The implementation worker owns solver code, public Rust tests and
native example; this worker supplies independent cell-average continuum and
origin checks only. Allowed additional writes: the two named tools above and
this plan's reports. No canonical solver-plan writes.
Use unchanged S1-E/F/G thresholds; S1-D imports educational LLF only for the
explicitly labelled differential check. [Format proposal](reports/s1-export-proposal.md)
owns the parent-coordination request. Root/quadrature/linearization uncertainty
and target-specific negatives must be observed. Run Python normal and -O tests
and actual native fields after the pinned solver commit arrives. Parent admitted
the export format at 4621d69747740a950a90d78755fe3bb4e220efe4 and relayed it to the
implementation owner; no numerical threshold change was requested.
**Status:** `Blocked`; native source 1b62ae3 was captured with reviewed origin
repairs. The Sod t=0 oracle is repaired without tolerance change and all 24
normal/-O tests pass. An independent Fourier derivation reproduces the frozen
acoustic order failure. See [diagnosis](reports/native-s1-diagnosis.md).

Parent requested independent review and CI inspection after format admission.
Capture-origin findings are repaired within this owner and independently
rechecked. Numerical thresholds/native exporter format remain unchanged; the
first-mode-only speed scope is explicitly disclosed. See
[independent review](reports/independent-review.md).

## Blockers and replanning triggers

Native source is available and capture executed successfully. A6 fails the
frozen acoustic coarse-grid order gate (0.5614384 <= 0.6). Parent approved preparing prospective 16/24/32 support, subject to independent
methodological review and canonical-plan update. Final reviewed plan/native
pins remain required before prospective native acceptance can proceed.
The original failed capture is retained; no denied or uncertain action exists.
Parent permits the existing NumPy only as a separately installed
validation tool here. No runtime packaging/distribution is proposed or authorized;
component license disclosure remains accurate and no GPL-free runtime is promised.
Full physical cases depend on solver capabilities and exact asset rights/access.
Replan if public APIs cannot measure the elementary
fixtures, source changes overlap another worker, or a new consumer/schema claim
or permanent mechanism becomes necessary. Repair a failed check inside scope.

## Concurrent work and repository isolation

Validation worker proposes `validation/cpu-cfd-acceptance` against main; parent is
integration owner. Compressible worker owns solver planning/core; requirements
are relayed through parent in [staged-validation](reports/staged-validation.md).
Branch is a short-lived shared draft proposal; never rewrite after push. No tags.
Worktree `/workspace/Rheon-validation` is retained-protected for parent review,
owned by validation worker; remove only after parent records integration or
another terminal disposition and confirms reachable commits. Original worktree
and unrelated refs are outside cleanup authority. Native input worktree
`/workspace/Rheon-native-s1`, branch `validation/native-s1-input-1b62ae3`, pins
1b62ae3 for reproducible capture. It is read-only in this task and retained-protected
by the validation worker until parent checkpoint/disposition coordination; do not
remove its ref or worktree before that disposition. External failed capture
directories are retained as evidence, not committed repository artifacts.

## Final acceptance

- Acceptance status: blocked (A1–A5/A7 satisfied; A6 original study failed; A8 awaits final plan/native pins)
- Deferred follow-ups: experimental geometry/data rights, exterior/moving solver qualification
- Final status: Blocked

## Milestone 3: prospective resolution-study support

Parent approves preparing the 16/24/32 family, subject to independent
methodological review and the canonical solver-plan update owned by the planning
worker. This is prospective only: original 8/12/20 failures and origin pins stay
unchanged. Allowed writes remain the two verifier tools and this plan's reports;
no native exporter or canonical-plan edits. Before implementation, this slice
admits an explicit `study_id` in suite schema v1, a fixed family per admitted ID,
and study identity in assessment output. Missing ID means original v1 only for
compatibility; an unknown ID or mismatched/mixed population fails closed.
Keep every numerical threshold, CFL, mode, amplitude, cell-average convention,
final time and norm unchanged. Derive order denominators from the selected fixed
mesh family. Retain all original probes and run the same numerical/layout/clock
negatives against prospective synthetic policy fixtures. Synthetic fields never
qualify a native run. Gate: normal/-O tests, independent bounded review, scoped
diff, commit/push/draft update. Actual higher-resolution native capture remains
pending the parent's reviewed canonical plan pin and revised native source pin.

Milestone 3 verifier support status: implemented and independently reviewed;
41 normal/-O tests pass. Native acceptance remains blocked pending parent pins.

## Milestone 4: explicit native capture and prospective evidence

Parent accepts reviewed canonical plan
`fc2270a85e38080d9943048aaf689d99545d3715` and native source
`5a049ed604b99817a428d10bbc0140732d3421d6`. The native selector intentionally
preserves original default behavior, so owned capture must explicitly forward
`--study-id s1-cell-average-acoustic-16-24-32-v2`. Before implementation, admit
one optional capture selector, recorded in origin and checked against the native
suite identity, with legacy original-only behavior retained. Never edit a native
manifest to select another study. Allowed writes are unchanged verifier tools
and this plan's reports; no native/core/canonical-plan edits. Run normal/-O
contracts and a real tiny argument-forwarding probe, independent read-only review,
then commit/push the clean capture verifier. Create a separate read-only protected
worktree at the exact revised native pin; use a fresh external output directory.
Capture actual native fields and repeat verification with the external origin
pin. Preserve original failures. A8 requires all unchanged numerical gates from
764ca2c plus exact origin identity; report every result, even a failure. No S2.

Native prospective input worktree `/workspace/Rheon-native-prospective`, branch
`validation/native-prospective-5a049ed`, is retained-protected and read-only in
this task, owned by validation worker until parent checkpoint/disposition. The
older native1b62 worktree and all original failed outputs remain untouched.
