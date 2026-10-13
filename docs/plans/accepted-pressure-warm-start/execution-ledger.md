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
