# Execution ledger

## 2026-10-15 — parent research and roadmap drafting

Parent's supplied draft ledger records reading pinned Rheon/lab interfaces, independent
review and source/authority corrections to the research report, and source-guided
Core/Router/Planning/composed-design/toolchain/feature/CI/output-policy review at
standards `188beda1`. No implementation, test, benchmark or publication occurred
in that drafting phase. Earlier execution setup failures were superseded by this
working repository task, not converted into successful historical evidence.

The parent supplied the 234-line draft roadmap, 13-line ledger and 16-line issue
record. Their content is integrated here: the canonical plan retains the detailed
M0–M3 handoff, proposed write boundaries and A0–A8 future gates, corrected for
actual current observations; the issue crosswalk preserves every draft finding.
Draft source remains proposal evidence, not implementation admission.

## 2026-10-15 — D1 documentation proposal

- Scope: authorized research/design, commit/push and public documentation PR;
  production implementation, model training, user-data changes and merge excluded.
- Inspected clean `/workspace/Rheon` on original `work` branch, fetched main at
  `43ed26b645dd42de906d0fac7d1ec7d5ab6e351a`, and created
  `docs/configurable-solver-experiments-20261015`. No new worktree or shared-history
  rewrite. Adopted standards fetched/read at `188beda1fa477d21d576c233dd8c7f4c4c267d23`.
- Reviewed parent-supplied research report transferred from Library in full
  (381 lines / 50,822 bytes) into `reports/research.md`; source review confirms
  baseline. Report is research evidence; plan owns proposal decisions.
- Cloned lab read-only at `a8426061380e82951b4dba1bbfa38ff93dc7ebb2`; inspected
  bridge/model/export source only. No model inference/training/build was run.
- `git ls-remote origin refs/heads/feat/live-ai-pressure-comparison` returned no
  branch; `git fetch origin 66e50388ec7a788dbda0ad1227e330996edf506e` returned
  `not our ref`. Q1 blocks implementation reconciliation, not documentation.
- Added typed contracts and standards-guided composed design, saved specs/sets,
  physical-time metrics and prospective implementation handoff under one directory.
- Parent selected canonical directory `docs/plans/configurable-solver-components/`.
  Relocated only this task's uncommitted authored documents; branch name remains
  `docs/configurable-solver-experiments-20261015`. Integrated the supplied roadmap
  with reviewed typed contracts and corrected stage-capture/diagnostic gates.
- Independent read-only GPT-6.1 Sol reviewer inspected source and documents.
  Initial findings added explicit stage capture and no-fallback diagnostic attempt
  requirements (Q3), and nonempty matched-interval evidence (Q8). Final review
  recommends documentation acceptance with no unresolved material design finding;
  [review](reports/review.md) records evidence and dispositions. No numerical or
  runtime acceptance claimed.

- Re-reviewed the integrated parent M0–M3 roadmap as a material design revision;
  independent reviewer again found no unresolved material design issue. Kept all
  implementation A0–A8 claims blocked/pending and future write sets prospective.
- Supporting validation: one-off Python standard-library check inspected all 7
  Markdown files, all 23 relative file/anchor links and balanced code fences:
  passed. `git diff --check` passed. Source report retained exactly; no generated
  fields, archives, binaries, logs, credentials or model artifacts included.
- Numerical, GUI, runtime and benchmark commands listed in the roadmap are future
  procedures and were not run for this prose-only change. Source inspection is not
  acceptance of these capabilities. No permanent validation harness added.

## 2026-10-15 — publication confirmed

- Reviewed exact staged diff and branch range `origin/main..HEAD`: seven authored
  Markdown files in the declared directory, no implementation or generated outputs;
  clean worktree and passing staged/range `diff --check`.
- Commit `50f43ce` (`docs(plan): design configurable solver components and saved
  comparisons`) pushed normally to
  `origin/docs/configurable-solver-experiments-20261015`.
- Confirmed `MrScripty/Rheon` is public and opened
  [draft PR #84](https://github.com/MrScripty/Rheon/pull/84), base `main`. GitHub
  returned the public PR URL and subsequent inspection confirms open/draft status.
  No merge, production implementation, training or user-data change performed.
- D-A1–D-A5 documentation delivery evidence satisfied. This final record changes
  publication status only; future A0–A8 implementation evidence remains blocked/
  pending. Parent owns review disposition and eventual branch retirement.

## 2026-10-15 — M0 reconciliation after integration publication

- User reported integration pushed and authorized documentation reconciliation on
  PR84. Inspected clean docs checkout, refreshed main (`43ed26b`), discovered and
  fetched `integration/ai-pressure-comparison-20261015` at
  `0fbadbe733b5b521e192dc8a2aa980d6d69a1e01`. Exact original `66e50388...` fetch
  now succeeds and `merge-base --is-ancestor` passes. Q1 access blocker resolved.
- Read complete engine/scene/model-process/worker/UI/record sources and matching
  lab Git objects at remote `a8426061380e82951b4dba1bbfa38ff93dc7ebb2`. No checkout
  of the lab clone was performed. Examined relevant instructions: no AGENTS.md or
  repository `.agents/skills` in the integration tree; workspace `.agents` empty.
  Used isolated ignored temporary source material; no implementation-agent edits,
  serving pointers, production code, runtime install, training or user data touched.
- Empty scoped diff verifies public session/provider/recording/simulation/provider-
  step/liquid-step owners unchanged. Diagnostic engine actually composes viscosity,
  native prefix and independent no-fallback continuation. [M0](reports/m0-reconciliation.md)
  maps exact source, scientific restrictions and limits; original research retained.
- Independent source/receipt review and a separate one-off Python stdlib audit both
  verify worker hash `b4f4e2fc...` and five log-wrapper plus decoded-text SHA256s.
  Retained publication observation is 17 steps / 18 frames cancelled then 16 steps /
  17 frames completed with 1 AI prediction, drained slots and Linux immediate-PID
  absence. Original assertions use ≥17/≥18. Source shows best-effort immediate-
  child kill/wait/readers, blocking writes and no hard teardown watchdog/descendants.
  These are retained local observations and source evidence, not fresh execution.
- Measured deployment, installed runtime and referenced scene/completion payloads
  are absent from the published Rheon snapshot. Ignored real-CPU test not rerun;
  no inference/GUI/numerical test or accuracy claim. Hash audit does not authenticate
  measurements or bind the entire executable/build graph.
- Revised plan handoff reuses actual engine/process/worker/record/UI owners.
  Independent review caught and resolved diagnostic-only M1 versus production A2/
  accepted-provider M2 dependency. A0/M0 source-design gate closes; production
  admission, generic contract/storage work and fresh environment evidence stay pending.
- Prior PR84 head `4176435` had successful scoped metadata CI and CodeRabbit status;
  integration `0fbadbe` had successful metadata run `38751946726`. Those are not
  measured CPU lifecycle or model-accuracy CI. Final update checks recorded below.
- Final independent review found no unresolved material finding. Its two low
  wording findings (historical inaccessible-branch label and one numerical owner
  per explicit mode) were corrected; no new implementation authority is implied.
- Local supporting checks passed: 8 Markdown files, 40 relative file/anchor links,
  balanced code fences, unchanged original research bytes, `git diff --check`,
  exact documentation-only stage/range inspection. No permanent test harness or
  generated numerical output was added. This update is one forward docs commit;
  normal push to PR84, without merge or rewriting earlier published commits.
- CI for this update is observed after push through PR84; no inherited scoped
  metadata success is represented as new lifecycle, runtime or model evidence.

## 2026-10-15 — M1 admitted and implemented

- Subsequent user authorization admits M1–M3, separate draft PRs, validated ready
  merges and safe retirement only after dependents clear. PR84 merged at
  `c85dfee`; fresh M1 branch `implementation/configurable-pressure-m1-20261015`.
  Selected comparison `6c38078` → `20b451d`, lifecycle `66e5038` → `d448563`;
  unrelated renderer/training commit `fe9d586` excluded. Stronger lifecycle
  assertions from `0fbadbe` retained at the original worker owner.
- Added borrowed pressure context and two independently authored stdlib CPU
  adapter packages. Existing Engine/ModelProcess/Worker/Record/UI owners remain;
  inert support checks precede execution, declared ports/output role configure
  package changes, prediction budget now covers stdin writing as well as reply.
  Legacy v2 frame consumer compatibility and honest generic provider identity
  are preserved. Core Session, accepted recording and physical gates unchanged.
- Independent GPT-6.1 Sol High source review closed four findings with actual
  negative/regression cases; no unresolved material M1 source issue. See review.
- Fresh focused command `cargo test --locked --features desktop --bin
  rheon-ai-compare --test pressure_context_contract`: 21 passed/1 explicitly
  ignored trained-deployment test plus 2 public context tests passed. Actual CPU
  children prove full/correction port selection, inert unsupported preflight,
  blocked-write timeout/drain, cancel after predictions/restart, and headless
  finished-worker preflight rendering. No model training, Torch/native-window
  pass or new trained-checkpoint accuracy claim.
- `cargo fmt --all --check` and desktop all-target Clippy `-D warnings` passed.
  Full desktop/core/default suites and clean-source educational qualification
  are still running/pending; no merge acceptance follows from focused results.
- Companion published M2 `755e2b4` (PR85) and M3 `c17efaa` (PR86), each four new
  isolated files. Fetched read-only; serial shared wiring and independent review
  remain here. Parent assigned strict educational admission refresh/qualification
  to this lead; PR86 comment records coordination. No gate weakening or stale
  receipts permitted.
- M1 clean committed native/rational qualification completed successfully on
  `33a083a1fdf63c169c9bf68c71009d495d505f64`, tree
  `34fc18bc29095450c2d8be31e5af68dc6457fb18`, in a separate detached checkout.
  Receipt `clean=true`, `status=passed`, `lean_checked=false`; native records hash
  `ecdd0877f17997c577d5183cd928c27c802f9ffdcee8324f0802fe3841fddb45`.
  Fresh education packet construction passed and its 26 adversarial tests passed
  both normally and under Python `-O`. Generated evidence retained externally at
  `/tmp/rheon-m1-qualification-33a083a` and `/tmp/rheon-m1-packet-33a083a`; no old
  receipts, qualifier changes or automatic source refresh used.
- Draft [PR87](https://github.com/MrScripty/Rheon/pull/87) publishes the M1 source.
  This receipt is pinned to the above commit; hosted whole-book/publication,
  broader Rust suites and unavailable model/window gates remain distinct.
- Companion next allocation is exact bounded immutable native comparison recipes/
  results in three new files, posted on PR86 comment6198894245. Shared worker/UI,
  source admission and merges remain lead-owned. Direct cloud session-ID messaging
  is unavailable; GitHub handoff plus parent relay supplies coordination.
