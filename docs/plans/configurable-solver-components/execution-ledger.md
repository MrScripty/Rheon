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
- Full `cargo test --locked --features desktop` completed successfully with no
  failing tests, including existing numerical, recording, worker/camera and
  desktop regressions; environment-specific model tests remained explicitly
  ignored. Public context tests also passed in no-default and default builds;
  corresponding library Clippy checks with `-D warnings` passed. No model/window
  claim follows from ignored tests. GitHub M1 native-cpu, native/rational, cloud,
  VTK and executable checks are green; remaining hosted book/features are pending.
- Evidence correction: this environment's /proc mount is not aligned with child
  PIDs; earlier negative /proc observations are insufficient OS liveness evidence.
  Replaced all six checks with fixed kill-zero probes, requiring positive liveness
  while resident and negative after owned wait/join. Independent source review
  approved; focused CPU suite freshly passed 21 tests/1 ignored, and scoped Clippy
  passed. This strengthens immediate-child evidence without changing production
  code, numerical gates, original retained M0 receipts or descendant guarantees.
  The changed test-source admission requires fresh clean qualification below.
- Native X11 software display is now available through workspace-local Xvfb.
  A real window rendered inert support feedback and a complete 64-step analytic
  fixture run with failed pressure diagnostics visible; this is not trained-model
  or hardware/GPU qualification. Native lifecycle observation is still underway.
- Final M1 isolated-target qualification on clean `f75a714`, tree
  `38026daa5f4eaa7633fe01c1e5e02c01ba3db264`, passed all 18 native/rational commands;
  packet construction and 26 adversarial tests passed normally and under -O.
  Separate target rebuilt Rheon source, without any Rheon artifact/fingerprint
  seeding. Focused CPU suite passed 21/1 ignored plus 2 public context tests;
  scoped Clippy passed. External receipts: `/tmp/rheon-m1-isolated-f75a714` and
  `/tmp/rheon-m1-isolated-packet-f75a714`; no receipt transfer from another tree.
- Real software-X11 native-window fixture observation completed. The analytic
  package includes an explicit delay for interaction, not a trained checkpoint.
  Inert support did not create observations; the window showed native qualified
  versus AI failed-pressure fields, an initial 64-step completion, cancellation
  after predictions, restart32 completion, then Close at step17 with2 predictions.
  Observer saw child liveness positively and later all four observed child IDs
  absent via OS kill-zero; app exit0. Binary SHA256
  `ad5838a409bc68419f923c47a8eef58e1dc1b4f187085de42a8622c72d9fe7d8`,
  manifest `fc93e8867a3f71bf45f3b25882f86876c9be257fbe7608df263ddee1123b6907`,
  observer JSON `5f5b06b40a94125eac2881dfbfaa0fad87e46fa58ecce8d80cef125b853ff830`.
  Generated screenshots/observations remain in `/tmp/rheon-m1-native-window`.
  This binds observed software-display/fixture material, not source authenticity,
  trained-runtime parity, a physical GPU, descendants or a hard teardown bound.
- Required measured deployment/Torch input was requested explicitly and remains
  unavailable. A1 trained-runtime and full generic/provider acceptance stay pending;
  CI's separate frozen-checkpoint check is not relabelled as this absent bundle.


## 2026-10-15 — native M2 serial composition

- Isolated branch `implementation/configurable-native-m2-integration-20261015`
  selects companion `755e2b4` → `372e013` onto M1, with no unrelated renderer work.
  Registered native specs/sets and a cold manifest view; existing worker/Inspector/
  recorder own execution and accepted capture. No numerical kernel/schema change.
- Native controls freeze before execution. New immutable intent/start/finish/final
  revisions preserve parent identities and acknowledged-path handoff. A legitimate
  shortened dt is recorded then reported unsupported by fixed-intent v1, retaining
  raw archive. No uncertain terminal result becomes a pin. Cold rerun uses captured
  intent from zero with fresh set/run/attempt identities; no integrator restart.
- Independent source review closed terminal path and attempt identity findings;
  current delta has no unresolved material finding. Scoped real App::poll worker
  test passed two variants, cold egui, captured-intent rerun/native bits/no overwrite
  and cancellation after publication with a Ready cancelled prefix + NotRun member.
  Existing saved contracts: 12 passed, including real producer process exit. Scoped
  desktop/contract Clippy `-D warnings` and formatting passed. Fresh clean-source
  native/rational and broader affected desktop checks remain pending.
- Admission review explicitly selects all 92 current paths and preserves strict
  gates/oracles/predecessors. This supports qualification attempts, not trained
  model, accepted-provider A3, native-window or complete M2/M3 acceptance.
- Selected later companion storage contracts `33a689d` → `97344a2` and typed
  constructor capture `c1079bc` → `31733ef`; independent review confirms retained
  bounded/no-overwrite semantics and actual capacity/terminal-busy failure coverage.
  Replayed M1 `f75a714` with its nonvacuous OS liveness oracle. Resolved documentation
  append conflicts by retaining both evidence histories and explicitly refreshing
  the final 92-path admission, not selecting a stale side's hashes.
- Earlier M2 clean `67186ce` native/rational + packet 26 normal/-O passed on its
  exact subject. A parallel desktop run hit the unchanged live-emitter gate's
  10-second timeout during heavy simultaneous UI/build work. No timeout/test was
  weakened; final source is rerun serially. The first attempted exact-name command
  lacked its module prefix and ran zero tests; it is not counted as evidence.
- Integration worktrees now use unique targets seeded with dependency artifacts
  only. A shared-target library-export mismatch exposed the risk of cross-checkout
  Cargo reuse; final fresh source qualification/tests are run in isolated targets.
  Earlier source-specific observations are retained as history, not transferred to
  this new combined source or substituted for required final passes.

- Final clean M2 `d8b0eb11c85de2efdeb522865d90a3b86fb52985`, tree
  `797022111c896ee5cef026b6ba22db1dcdb5ce18`, passed all 18 native/rational
  qualification commands with `lean_checked=false`; the fresh education packet
  passed 26 adversarial tests normally and under Python -O. External receipts:
  `/tmp/rheon-m2-isolated-d8b0eb1` and `/tmp/rheon-m2-isolated-packet-d8b0eb1`.
  Its 17 saved-contract tests passed. A strict desktop run hit the unchanged
  live-emitter 10-second wait under concurrent build load; the exact test then
  passed alone (8.79 s), and a complete idle serial rerun passed 61 tests,
  1 intentionally ignored model-runtime test, 0 failures (30.67 s). All-target
  desktop Clippy with warnings denied passed. No timeout or test was weakened.
  This evidence is pinned to d8b0eb1, not a trained bundle or a new source tree.


## 2026-10-15 — native M3 analysis and immutable comparison evidence

- Selected companion numerical analysis and saved recipe/result ownership into
  the serial M3 branch. Public recording::comparison is the actual app/test API;
  the desktop aliases it and registers its saved-comparison owner. No unrelated
  renderer ancestry, second numerical worker or accepted-schema change.
- Independent source review found public derived-divergence overflow; fixed with
  explicit ArithmeticRange and finite-face/extreme-spacing regression. Review
  confirms no remaining material source finding. Independent SQLite/native-bit
  oracle now fails closed normally and under -O/PYTHONOPTIMIZE for wrong identity
  and metric; no production numerical bounds/formulas were weakened.
- Unique-target validation passed 16 numerical +12 saved-comparison contracts and
  scoped desktop Clippy/formatting. Native desktop comparison-page interaction is
  still pending the companion new-file handoff; no GUI acceptance is inferred.
- Explicit final M3 source admission includes all 94 selected paths, preserving
  both historical inventories and current conservative selector. A fresh clean
  qualification starts after this commit; earlier receipts are not transferred.
- Native software-X11 M2 flow on d8b0eb1 created a two-solver 8-cube set with
  2 steps at0.003s, closed producer, cold-opened Ready members, audited ordinal2,
  exported actual VTI and reran exact intent from a fresh material-default App.
  Native bits/time and originals matched, but process-local source IDs1/2 were
  reused. This is a finding, not freshness acceptance. Exact failed-freshness
  observation SHA256 `37f32ab2dd80bb0a3dd5cdffaf0fdc27325e8ec544c171eb9d9de3071139e6e4`
  remains outside Git in /tmp/rheon-m2-native-window/observation-before-fix.json.
- Fixed root-only direct-source exclusion. Independent review approved source and
  regression fidelity: actual separate producer/cold-rerun processes, full source
  ID disjointness, original-file BEFORE snapshot, typed intent/native bits/time.
  Focused fresh regression passed. Refreshed92 source admission precedes renewed
  qualification; corrected binary/native-window observation follows separately.

- Clean M3 bc1fc90360ffae12559a80624c4f15fbd3f8a67d (tree
  6f9a52d024942c100c29777bb183e3b86de8c06a) passed 18 native/rational commands,
  fresh packet construction and all26 adversarial tests normally/-O with real
  packet supplied (0skips). 12 saved-comparison +16 numerical contracts passed;
  the16 also pass core-only. Scoped desktop Clippy passed. External evidence:
  /tmp/rheon-m3-isolated-bc1fc90 and /tmp/rheon-m3-isolated-packet-bc1fc90.
- Selected companion bb48cee new cold page and7 headless interaction fixtures.
  Lead wires actual root/sidebar/run-set navigation and actual public analysis in
  tests. Independent source review found no material page/navigation finding.
  Seven integrated page tests passed; initial Clippy caught unused imports after
  replacing the private test numerical path with the public API; imports removed.
  New95-path source admission committed before fresh qualification. Companion
  harness/window evidence is historical and does not qualify this root binary.

- Integrated page source3584749 (tree62f0957488476a4b4bc0faa1bebbc78641f9f3f7)
  passed clean18-command native/rational, packet26 normal/-O0skips,12 saved+16
  numerical+7 page contracts, scoped Clippy and native binary build. Receipts in
  /tmp/rheon-m3-page-3584749 and /tmp/rheon-m3-page-packet-3584749. These pin that
  source only, not subsequent values/renderer changes or native root interaction.
- Selected independently reviewed06ccf43 exact-value and explicit materialization
  hooks, retaining public raw samples/overflow test. Fixed its pointwise Python
  oracle assertion stripping with explicit failures and negative perturbation in
  all3 interpreter modes.20 numerical+9 page tests and scoped Clippy passed.
  Explicit95-path admission refresh precedes fresh qualification of this subject.


## 2026-10-15 — partial diagnostic recording owner

- Separate branch implementation/diagnostic-recording-m3-20261015 contains agent
  f87a7a2 typed storage plus fb8405a review fixes, serial accepted-kind guard and
  latest native M3 source. New run/request/attempt/host-observation/native-reference/
  transition/terminal types stay outside accepted RecordedStep/Session ordinals.
  Reuse SQLite owner/budgets; no engine/worker/model-process or dependency changes.
- Independent review closed signed-zero domain round-trip, rejected-seed lineage
  and impossible unrecorded-terminal generation findings. Actual native teacher
  fixture retains Accepted priors and unchanged native evolution despite rejected
  biased model observations. Explicit native-prefix declaration is not preceding
  trajectory capture or qualification proof. Seventeen diagnostic+15 accepted+4
  carrier tests and scoped Clippy passed before final serial source composition.
- Fresh96-path admission preserves historical inventories and conservative gates.
  Final-owner clean qualification and affected tests follow; partial storage does
  not close A6. Runtime attempt capture/policy/record-before-UI wiring, typed saved
  diagnostic intent, trained bundle and linked projection remain pending.

- Final partial diagnostic owner f56a756662711413a4339357956443479425bfc3,
  treef2d38604329906ed1133f152214ee1e80ac04476, passed clean18-command
  native/rational qualification (`lean_checked=false`) plus fresh packet26 tests
  normal/-O with0skips. Combined core tests passed17 diagnostic+15accepted+4carrier
  +20nativecomparison=56; scoped core Clippy with warnings denied passed. Evidence:
  /tmp/rheon-diagnostic-owner-qualified and /tmp/rheon-diagnostic-owner-packet.
  Draft PR92 publishes partial storage, not teacher/stop runtime completion.
- Corrected native M2 b24e246 passed clean qualification+packet26 normal/-O0skips,
  full desktop63pass/1model-runtime ignored,17saved contracts and all-target
  desktop Clippy. Real software-X11 cold rerun from retained producer source now
  excludes ALL source IDs1/2; both tracks retain native bits/time and source files,
  and Close exits0. Observation027b75ef3ef193083e3b16c27759865cc53167e2d8235ec9967358b804624689,
  binary59b5c5e88c59c833c6016f2cdb6357f7bb846da7a106c4018337dee76c4d97b7.
  Generated evidence stays in /tmp/rheon-m2-native-window-fixed; original failing
  freshness observation remains separate. PR88 body binds exact results.
- Native M3 b38c3fe passed clean18 commands+packet26 normal/-O0skips,20 numerical
  contracts core-only in addition to the20desktop+9page+12archive focused matrix,
  scoped Clippy and actual binary build. Integrated root-window flow opens exact
  set→comparison, selects pressure ref1..2/candidate1 (1matched interval plus
  1unmatched reference), shows actual times/intervals/Pa/gauge/common bounds,
  exports without overwrite, reopens Unverified then verifies. Hiding an owned
  test source shows Missing and clears stale metrics; exact bytes restored.
  Window Close exits0. Actual exported512-sample metrics pass independent stdlib
  SQLite/native-bit oracle normal/-O/PYTHONOPTIMIZE=1. Observation
  36e06591f177ed7b72ef9afd0b00c0f8a76bdda999c7086362701740601af550;
  result0febc3e9004856bab33c7abc946b5a19f3c79364d098ac382e0d1f256bf2d41d;
  binarye1e5306288bccb32f1437c3caec2847712419deb130730a38596ba4e8968ffdc.
  Generated material /tmp/rheon-m3-native-window; exact clean receipts
  /tmp/rheon-m3-values-qualified and /tmp/rheon-m3-values-packet. Software display,
  bounded native sources and artifact identities only; no trained/GPU/authenticity
  or linked physical slice/probe claim. PR91 body retains exact evidence.
- Documentation-only follow-through does not transfer a receipt to another tree.
  Required hosted checks are still pending; no implementation PR is merged.
  Preserve every active stack/companion branch. The measured live-water deployment
  bundle/Torch is still unavailable, and the existing frozen trajectory-v2 CI
  checkpoint is a distinct consumer. A1/model parity is blocked on that input.
- Companion06ccf43 exact-value/page hooks are integrated. New linked view module,
  tests and docs were allocated via PR89 comment6200315546, clarified in6200446821;
  no new renderer file handoff has been published. Parent can resume the separate
  companion thread01a140a3-ebca-7436-82cf-94fedc42b036 on that concrete allocation.
  Existing shared worker/root/admission/merge ownership remains serial.
