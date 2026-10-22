# Execution ledger

- 2026-10-13: drafted proposal while owner contract pending; no production edits.
- Parent relayed PR55 owner agreement. Recorded binding contract; chose coupled
  accepted-pressure reuse as smallest first slice, avoiding standalone seed array.
  Frozen base3525c979 (PR55) fetched; separate local branch moved to that head.
  PR55/57, PR58 and rejected-assessment branches remain unchanged.
- Next: implement explicit seeded numerical solve and owner-local provenance/commit
  behavior within agreed contract, then test before performance admission.

- Implemented borrowed seeded solver, coupled accepted provenance and opt-in Session
  path. Independent read-only review found no blocking defect; unsupported-path
  and preferred-provider tests added in response. Strict work-observation clippy
  passed; numerical/owner/surface/column/provider scoped tests passed (real-model
  checkpoint test remains the existing ignored external-input test).
- Pilot32³/1worker/4steps: steady cold0.9571s/warm0.3763s; localized1.0434/0.9047;
  switch0.7403/0.8107; rest0.1676/0.1492. These single runs are not qualification.
  Switch removal incurred277warm iterations vs34cold; retain this adverse result.
  Saturated fraction1 failed identically on first step for both policies, under
  unchanged fraction-bounds gate. No successful shortened horizon is claimed.
- Full pre-recorded matrix is still required. No performance admission yet.
- Full release regressions passed: default371/core364/work-observation381, zero
  failures (three external-checkpoint tests initially ignored). Strict default,
  core and desktop clippy and formatting passed. Python executable suite119tests,
  21skips. All three real-checkpoint contract targets then passed with the verified
  export (provider8, solver-provider6, pressure-session4); checkpoint export hash
  902ad8d77b16ae42f22f90bf71c22c19ae2e3cbe0cc0f4cc3d770930d4049aea.
- Completed corrected primary-small matrix:18cases ×6permutations ×3variants,
  no numerical/memory/replay/horizon analyzer violations. First64³ complete
  triplets retained; no final acceptance while controls/reference remain pending.
- Paused64³ matrix by explicit SIGTERM to close an uncovered saturated-state
  boundary. Completed triplets remain, interruptedr4 is archived with reason and
  excluded from ratios. Added16-step postaccepted saturation check: Jacobi both
  policies16/16; SGS bothsame first FractionBounds0/16, rollback/retry exact.
  New independent Neumanncosine refinement test passed both preconditioners and
  initializations across8/16/32/64 cells. No gate/tolerance or production edit.
- Resumed the interrupted triplet from scratch, preserving its partial predecessor.
- Qualification stopped after reproducible64³/w4/localized lost horizon: five
  completed warm trials fail step1 at3000iterations/maxresidual2.323e-9, whereas
  cold/base complete16. Candidateec535bc is rejected for admission. Remaining
  matrix/reference/particle stages were not executed. Preserve partial next
  trial, exclude from ratios. Independent review found no seed ownership defect;
  investigate gauge-only true-residual rejection causing repeated PCG restarts
  in isolated diagnostics before considering a fresh candidate. No fallback or
  loosened tolerance authorized. Default-cold confirmation remains pending.
- Isolated diagnostic atiteration400: reduced true7.885e-12 agrees with recursive
  norm below3.141e-11 threshold, while gauge/full residual2.3299e-9 rejects.
  Repeated restarts persist to3000. Opt-in reduced-system replacement criterion
  preserves conjugacy without altering full acceptance; instrumented correction
  completes16/16 actual64³ localized steps. Reviewer independently supports
  mathematical distinction. This is a new candidate requiring fresh matrix.
- Diagnostic copies initially shared the production target directory. Although
  frozen binaries remain immutable, isolate diagnostic target before comparing
  regression tests and rebuild production into its own fresh target to prevent
  same-package artifact reuse from contaminating qualification.
- Reviewer audited evidence helpers during corrected64screening. Hardened record,
  receipt/horizon/finite-value checks, explicit errors under Python-O, final full
  matrix coverage, case-level suppression on incomplete repetitions, raw overwrite
  refusal and archive nested-content rejection. No binaries, timings, case order,
  horizons or numerical thresholds changed. Initial small/64groups ran original
  runner; later groups use hardened runner, whose hash is retained per manifest.
  Existing records are revalidated by the stricter analyzer; no discarded trials.
  Tighter reference is explicitly diagnostic, not an independent accuracy oracle
  or automatic numerical pass without justified bounds. Physical acceptance uses
  unchanged solver gates plus independent hand/continuum/flux/hydrostatic tests.

- Environment-disconnection notice did not interrupt session 89767. Checkout,
  frozen binaries and raw measurement records remain intact; qualification continues.
- Reproducibility audit corrected an overstatement about harness equivalence. The
  frozen baseline also predates optional tight-reference/terminal-dump support.
  Retained exact baseline source and diff; normal timed loops and tolerances match.
  No binary, workload or timing result changed.

- Completed corrected screening: 40 cases, 240 balanced triplets, 720 runs, zero
  analyzer violations. All nonsaturated horizons complete; all saturated attempts
  retain identical rejection. Tighter-reference and particle screening complete.
- Independently selected and froze one final 228-run confirmation cohort before
  execution. All runs complete with exact outputs/horizons; no postcohort extension.
  Main warm gains persist. Mixed cold controls and positive 1000-particle medians
  remain disclosed; no equivalence or universal no-regression claim is made.
- Final independent review recommends scoped opt-in admission with unresolved modest
  particle runtime-regression risk. All local lints, actual-model contracts, fresh
  CLI/Python regression, helper negative checks and formatting pass. Lossless archive
  round-trip analysis reproduces corrected evidence exactly under Python -O.
- PR 55 base rechecked unchanged at 3525c979 before publication. No owner branch,
  collision mesh, book binding, main merge or deployment was modified.

## 2026-10-21 fresh authorized PR59 integration

Fresh user instruction authorizes completion and normal serial merge of this PR;
previous evaluation-only publication limits are superseded. Preserve original
7f1733f head/ancestry, source-bound receipts and all adverse outcomes. Main fetched
at15026a9; no AGENTS found. Core/router and applicable current coding standards read.
Merge commit0b3e3fc resolves force/configured-provider conflicts without rewriting
history. Warm source is now an explicit non-default Cargo feature, per the user's
allowed transparent narrowing to qualifying path; historical particle +2.20% is
unchanged and not excused by a runtime opt-in. Actual default controlled evidence
remains required. Session new entry point also updates current-main interval and
configured metadata atomically. Independent exact-source review found no blocking
source defect;21 material hashes bind0b3e3fc. Preliminary compile, focused35-pass/
1model-required ignored, and warm strict Clippy outcomes are retained in
integration-20261021. Inherited EOF blank lines and raw command-output trailing
blank lines are not altered; scoped source diff-check and formatting pass.
Native gate refuses added inventory path on committed source, as expected; no
inventory refreshed or gate bypassed. Final #142→#138→#143 composition, qualified
source admission, default controls, required CI and lead merge remain pending.


Preliminary source-frozen regression completed: release default610pass/10existing
skips, feature-enabled626pass/10same skips, warm library32pass; no failures. Strict
core/warm all-target Clippy passes. Seven default skips remain explicitly bounded
upstream force integrations; three require the real exported checkpoint. Real AI
hosted qualification39687940496 passed at0b3e3fc. Preliminary book39687940422 fails
at the source-admission check; preserved, not rerun. Existing code-source review
remains bound to0b3e3fc; final dependency composition still requires assessment.

Before any timing, separate-root binaries showed embedded path-string confounding;
identical two-prefix remapping was frozen after independent review. An inspection
mistake (objcopy --dump-section without output object) rewrote inspected inputs.
Those artifacts/reviews/hashes are retained as diagnostics, explicitly corrected.
Raw outputs were regenerated with unchanged source/flags, original raw hashes
reproduced, and fresh read-only review bound those raw compiler artifacts. Code,
ordinary data, load layouts match except e_shoff/build-ID metadata; whole-file
identity does not hold. No timing ran, so no measured cohort was contaminated.
No evidence exception is approved: proposed narrow image criterion awaits lead
agreement before eventual final-source fixed controls. No strip/patch/layout tuning.
