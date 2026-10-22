# Actual merge hosted runtime checkpoint

All six actually triggered hosted checks passed on actual PR143 merge
`20a0e1e20538b859193c08eaf25a3b8cd26135a5`, full tree
`d66c930d091f8e84cf445f0920922d65afed8f8e`. The source/graph and actual-merge
sanity receipt remain in [postmerge.json](postmerge.json). The final actual
branch/merge/head checks are separate from the earlier actual159 qualification.

[Rust39707942188](https://github.com/MrScripty/Rheon/actions/runs/39707942188)
and [dense39707942235](https://github.com/MrScripty/Rheon/actions/runs/39707942235)
are completed SUCCESS, event push, branch main, exact actual-merge head.
Default, core-only, desktop, executable, aggregate and dense check runs all
succeeded, with no pending or failed triggered check. This is six checks on
the actual merge, not the pre-merge candidate's20 completed checks.
[postmerge-hosted-checks.json](postmerge-hosted-checks.json) preserves the
normalized actual tool responses and exact source/event/job identities.

## Independently audited actual-merge logs

All feature logs print the actual push fetch and checkout Git HEAD20a0e1e.
Strict all-target Clippy passed in each unchanged feature selection. Rust
counts below exclude separate doc tests and nested filtered child processes.

| Mode | Top-level targets | Passed | Ignored | Separate doc passes |
| --- | ---: | ---: | ---: | ---: |
| Core-only | 88 | 764 | 13 | 3 |
| Default | 89 | 772 | 13 | 3 |
| Desktop | 91 | 1156 | 20 | 3 |

Four additional one-pass desktop child summaries with223/223/11/11 filtered
tests are excluded from the top-level count. Every observed summary reports
zero failures. The original three CV3 numerical contracts remain explicitly
ignored, with their unchanged frozen-oracle/resource-admission reason, in all
three modes. No selected nine-case campaign or CV3 numerical contract was run.

The executable job passed bootstrap7 normal+7 optimized and220 harness cases:
193 passes with27 explicit conditional skips. Its actual size16,12-step,
source-off-at8 CLI run passed completion-manifest, all-step divergence and PNG
dimension assertions. This is the recorded bounded headless smoke; no physical
calibration, unbounded stability, hardware/native-window or performance result
is inferred. Artifact upload metadata is not downloaded artifact-content
acceptance. No loaded ELF/build/environment equivalence is asserted.

[Independent executable review](postmerge-executable-review.md),
[core/default review](postmerge-core-default-review.md) and
[desktop/current-hosted-state review](postmerge-desktop-review.md) passed.
[postmerge-runtime.json](postmerge-runtime.json) records raw decoded-log sizes,
SHA256 identities, exact scoped counts and independent review hashes. Logs stay
outside Git. A reviewer optional direct single-job metadata URL was rejected
by the wrapper as400 unsupported and stopped; no retry or alternate retrieval
of that optional action occurred. Supported complete-run/check metadata and
the already available own decoded logs supply the deciding evidence. This was
not a numerical failure or permission denial.

## Separate local book/VTK and publication gates

Book and VTK did not trigger on the actual merge's push because the exact
private-preparation/examples/tests/plans delta matches neither existing path
filter. The lead's [prospective amendment](https://github.com/MrScripty/Rheon/pull/142#issuecomment-6330943452)
retains those NOT_TRIGGERED outcomes and requires all actual-merge hosted
Rust/dense checks plus fresh source-bound local VTK and every named early book
build step through native fixtures, independently reviewed. Any known later
local static/PDF/browser failure also blocks further integration; final actual
main full validation/publication requirements remain intact.

The lead's [actual-merge VTK component audit](https://github.com/MrScripty/Rheon/pull/142#issuecomment-6331084452)
passed: nine Rust boundary contracts, strict Clippy, real independent VTK-reader
and recorded-carrier commands. This is lead-owned local execution, not a hosted
VTK job or this worker's execution. Its immutable SQLite evidence binds the
actual merge. Post-command ELF snapshots remain file snapshots, not observed
process/embedded-commit/reproducibility attestations.

The whole first local attempt failed before tests at Cargo DNS setup after the
controlled environment omitted required platform transport. The whole second
attempt failed at npm cache setup after its VTK component passed. Neither whole
attempt is called passed. Their source/output custody and both failures remain
preserved; the transport-only recovery was reviewed separately. The [reviewed
book-only continuation](https://github.com/MrScripty/Rheon/pull/142#issuecomment-6331134413)
uses a new writable task-owned npm cache/output3, retaining exact workloads,
source, sandbox, budgets and cleanup, and does not rerun passed VTK. Its book
results are not asserted passed by this checkpoint.

The earlier actual-N book run39704023712 completed build, bounded lessons and
Pages deployment successfully under N. It is not an actual-merge publication.
No connected-app workflow-dispatch action is callable. A separately checked
browser was signed out with no Run control and stopped without login, dispatch,
credential or settings changes. The existing action has never been attempted
or denied. If this merge remains final, completion needs an authenticated
supported Actions route for one existing workflow
`.github/workflows/research-book-pages.yml` on main, after verifying its exact
current expected source, or a later legitimate source-triggered final-main run.
No unrelated trigger commit, borrowed credentials, failed CLI retry or denied
artifact/log/cloud-task route was used.

The integration lead retains the final-current-main browser, separate bounded
lessons, upload and Pages deployment gate. The local book build is not a claim
that those hosted actions happened. [Owner runtime handoff](https://github.com/MrScripty/Rheon/pull/143#issuecomment-6331254102)
records this same separation. PR143 preserves bounded CV1/CV2 and CV3-A
preparation; it does not establish scientific nine-case, generic PDE/cut
stability, moving geometry/GCL, learned/GPU, native-window, performance or
whole-application production acceptance.
