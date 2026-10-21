# Publication CI cancellation diagnosis and proposed bounded remedy

Prepared 2026-10-21 for parent coordination. Initial diagnosis checkpoint:
shared-workflow remedy proposed, not applied or dispatched. PR150 remains draft
at `db6f33d4f57b0033cf866059f65f7b2033837108`. Its GPU book content and independent
preview-PDF review are unchanged. Cancellation is not a successful full gate.

## Recorded cause and uncertainty

Run [39626898860](https://github.com/MrScripty/Rheon/actions/runs/39626898860),
build job [119314799555](https://github.com/MrScripty/Rheon/actions/runs/39626898860/job/119314799555),
started `15:35:23Z` and completed `17:05:52Z`: 90 minutes 29 seconds including
cleanup. Its workflow sets `jobs.build.timeout-minutes: 90`. The last command
was cancelled at `17:05:49.080Z` with `The operation was canceled`; cleanup
followed immediately. The job conclusion is `cancelled`, not `failure` or
`success`. All earlier steps have successful terminal conclusions. No assertion
failure appears before cancellation.

This is strong evidence for the configured job wall-clock timeout. GitHub's
[official timeout documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idtimeout-minutes)
states that the job is automatically cancelled at its configured limit. The
accessible run/job metadata and log do not contain the deciding timeout
annotation, so manual/platform cancellation cannot be excluded absolutely.
The connector rejected the check-run annotation endpoint as an unsupported URL;
no alternate route was tried. A workflow-run-list endpoint was also rejected;
no alternate listing route or historical timing claim was substituted.

## Measured per-step and subcommand timing

All times below are runner log/API timestamps on 2026-10-21, not locally
executed workloads or a performance claim about the solver.

| Phase | UTC interval | Elapsed | Terminal evidence |
| --- | --- | --- | --- |
| Setup through native browser fixtures | 15:35:23–15:43:48 | 8m25s | Successful |
| Static edition, PDF and site/lesson verification | 15:43:48–16:10:43 | 26m55s | Successful |
| Build receipt emitted to actual PDF render result | 15:44:58.994–16:06:21.756 | 21m22.8s | Render result, no page/network errors |
| Site verification | render result–16:06:33.766 | about12s | 47sections,1776math,9010localtargets;918ZIP links;467portablePDFlinks |
| Additional lesson probes and optimized timestep suite | 16:06:33.766–16:10:43.467 | 4m09.7s | All passed |
| Explicit retained browser check | 16:10:43.502–16:31:57.081 | 21m13.6s | Passed, Chromium155.0.8059.84 |
| Runtime positive probe, including suite startup | 16:31:57.081–16:52:10.645 | 20m13.6s | `test_current_six_labs_pass_without_pdf_or_receipt_writes ... ok` |
| HTTP rejection probe | previous result–16:52:15.206 | 4.6s | Passed |
| JavaScript rejection probe | previous result–16:52:20.565 | 5.4s | Passed |
| Pending missing-native-hub rejection probe | previous result–17:05:49.080 | 13m28.5s observed | Interrupted; no terminal result |

The rendered/site-verified book and explicit retained check therefore passed.
The final runtime suite did not finish. The other runtime probes and both
normal/optimized aligned-strain suites have no terminal evidence in this run.
The final full-site artifact upload was skipped. Preview artifact12357433282
remains separate valid evidence for its own exact source and PDF.

## Source explanation and alternatives

`test_browser_runtime.py` calls the complete `verify_browser.verify` for every
probe, copying the edition and checking that retained PDF/receipts do not change.
The complete qualifier includes `flow_browser.qualify`, whose admitted native
Wasm trajectory exercises 80 accepted steps for each of two pressure methods.
This is existing browser qualification, not a GPU backend experiment. The
missing-native-hub mutation is checked by `qualify_native` only after teaching,
transport, timestep and flow qualification. Thus its negative test legitimately
repeats much of the roughly20-minute positive path before observing the error.
Alphabetic unittest order places that missing-hub test after the three reported
successful tests. Its start is inferred from that order and the last successful
result; no separately emitted start timestamp is claimed.

The three completed full-browser traversals are consistently about20–21minutes;
there is no measured transient stall supporting an unchanged blind rerun. At
the observed rate, finishing the interrupted late negative path alone could
need roughly another7minutes, before remaining runtime and aligned-strain tests.
This is an estimate, not an observed completion or a guaranteed bound.
The repository already raised this same job from60 to90minutes in commit
`900e30b5807497c3c2bb20e3d7d0fed6d9e42cc1` after an earlier publication overrun;
that history establishes intent to retain coverage, not a comparable timing
measurement for this source.

## Chosen smallest remedy for parent coordination

Propose changing only this publication job's wall-clock cap from90 to120minutes,
with a comment naming this run and the measured repeated browser traversals.
The extra30minutes covers the observed interrupted tail plus bounded uncertainty
for the unmeasured remaining suites. It is a capped resource decision, not a
promise that120minutes must succeed. No new solver cohort, trajectory length,
iteration limit, acceptance threshold, per-method600-second bound, sandbox
setting, test, command order or deployment gate changes. No `continue-on-error`,
reduced fixture, numerical fallback or unqualified artifact acceptance is added.

A parallel-job design would add generated-input and native-packet transfer,
same-head provenance, freshness restoration and final deployment dependencies.
Concurrent browsers on one runner could instead introduce CPU/memory contention.
Those are larger changes than this evidence currently justifies. They remain
re-plan options if one corrected bounded run still exhausts120minutes.

Exact prospective write set: `.github/workflows/research-book-pages.yml` budget
and explanatory comment, plus this GPU publication plan/ledger/issues/report.
The parent must coordinate ownership of that shared workflow before application.
The full test commands and coverage remain byte-identical. Independent review
must check that normalized diff before a normal commit/push. A rerun at the old
head cannot test the changed cap because it uses the old workflow revision.

After coordination: apply the reviewed budget-only change, record a new exact
head, let matching normal CI execute once and retain the terminal results and
full-site artifact. Do not manually dispatch unrelated successful cohorts or
retry this cancelled run unchanged. Preserve prior failures and preview/source
reviews with their original identities. If the120-minute run fails, cancels or
times out, stop and re-plan rather than automatically raising the budget again.
No merge until every required gate completes and parent artifact/publication
verification is satisfied. This report executes no science or browser workload.

## Access and execution status

Read-only completed job logs, run metadata and job/step timestamps were retrieved
successfully through the permitted GitHub connector. The following requests
were rejected by the connector allowlist with HTTP400 and were not retried:
`/repos/MrScripty/Rheon/check-runs/119314799555` and
`/repos/MrScripty/Rheon/actions/workflows/research-book-pages.yml/runs?per_page=15&status=completed`.
Those rejected reads caused no mutation and leave the detailed cancellation
annotation and recent-run comparison unavailable. No workflow/source change,
CI rerun, push, merge, toolchain installation, local science, GPU workload,
browser launch or security change was performed for this diagnosis.

## Authorized application checkpoint

After the diagnosis and independent review, the parent explicitly authorized
the exact90-to-120-minute publication-job cap trial and explanatory comment,
plus these owned admission records. The reviewed candidate patch SHA256 is
`7a950a9ee83d86d46e941f5ae16a46279e07670e72fb7591aa70298b13a0f58e`.
Its applied workflow SHA256 is
`b1f6b5afd3be0e725808ddd9857460d5ebe111c68ee87061aeb8464dbc214f48`.
Normalization againstdb6f33d proves all other workflow bytes unchanged.
The [independent candidate review](ci-budget-independent-review.md) retains its
pre-application identity and access limits. A normal new-head commit/push and
one matching automatic CI run are the next execution, not a manual duplicate
rerun. Their actual head/tree/run identities belong in the handoff rather than
being predeclared here. No terminal success or new artifact is claimed at this
checkpoint. If this trial times out, stop and re-plan from its timings.
