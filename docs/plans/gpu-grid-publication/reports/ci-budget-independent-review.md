# Independent review of publication CI cancellation diagnosis

Reviewed 2026-10-21 against PR150 source head
`db6f33d4f57b0033cf866059f65f7b2033837108`. Read-only run:
`39626898860`; completed build job: `119314799555`.

**Judgment: the diagnosis is supported, and one coordinated 90-to-120-minute
job-budget trial is a reasonable smallest remedy.** This is not a completed
gate, a guarantee of completion within 120 minutes, or permission for this
reviewer to apply/dispatch it. No workflow or CI architecture change was made.

## Independent evidence checked

The reviewer successfully retrieved the completed job's decoded logs through
`github_fetch_workflow_job_logs` and current job conclusions through
`github_fetch_workflow_run_jobs`, both supported read-only connector actions.
The latter reports build `cancelled`, bounded-lessons `success` and deploy
`skipped`. The full decoded job log contains 2,551 lines and corroborates the
recorded source checkout and all decisive subcommand milestone timestamps.
The saved `timing-evidence.json` supplies the job/step start/completion summary;
its elapsed-time arithmetic is consistent with those milestone logs.

The configured cap is 90 minutes at
`.github/workflows/research-book-pages.yml:123`. The recorded job interval is
15:35:23–17:05:52 UTC, or 5,429 seconds including cleanup. The log emits
`The operation was canceled` at 17:05:49.0802842Z and immediately performs post-job
cleanup. No assertion failure or traceback appears before that cancellation.
All qualification steps before the final grouped browser-runtime step have
successful terminal conclusions; the full-site artifact upload was skipped.

Independently recomputed intervals are:

| Observed interval | Seconds | Interpretation |
| --- | ---: | --- |
| 15:44:58.994–16:06:21.756 | 1,282.762 | Build receipt to successful actual render-pdf result; includes browser qualification, not PDF printing alone |
| 16:10:43.502–16:31:57.081 | 1,273.579 | Explicit retained browser check |
| 16:31:57.081–16:52:10.645 | 1,213.564 | Runtime positive test including suite startup |
| 16:52:20.565–17:05:49.080 | 808.515 | Observed interrupted interval attributed to the next missing-hub test by ordering; no separately logged start timestamp |

The HTTP and JavaScript rejection results are present at 16:52:15.2059708Z and
16:52:20.5654326Z. The final runtime suite and both subsequent aligned-strain
suite invocations have no successful terminal result in this run. Passing prior
render/site checks or the separate preview artifact does not close those gates.

The official [GitHub timeout definition](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idtimeout-minutes)
states that the job cap controls automatic cancellation. The approximately
90-minute elapsed time strongly supports that cause. The accessible log/API
does not establish the deciding timeout annotation, so manual or platform
cancellation remains possible. The diagnosis correctly retains this uncertainty.

## Source control flow checked

`docs/education/test_browser_runtime.py:32` invokes the same complete
`verify_browser.verify` entry point for all six probes and checks retained
PDF/receipt bytes afterward. A probe may still fail early; entry-point identity
does not mean every negative probe finishes every qualifier.

`verify_browser.py:131–139` runs teaching, transport, timestep and flow before
native qualification. `native_browser.py:21–22` checks the mutated hub only
after loading `native-labs.html`. Thus the missing-hub test's long repeated
native-flow traversal is explained by actual order, rather than evidence of a
new GPU workload or a demonstrated transient stall. Default alphabetical test
ordering places it after the three completed tests and before mobile-error and
unresponsive-control probes.

The flow qualifier keeps two pressure methods and 80 accepted steps per method;
`flow_browser.py:841` retains the 600-second deadline for each method's resumed
trajectory loop. This is a bound on that phase, not the entire browser process.
The remaining mobile-error/control mutations can be detected before the long
flow path (`verify_browser.py:68–71`, `test_browser_runtime.py:51–61`). The
aligned-strain browser suites invoke their direct strain qualifier
(`test_aligned_strain_browser.py:60–78`), not the full flow traversal. Their
remaining completion time was not measured by this cancelled run.

The historical 60-to-90-minute cap change at commit
`900e30b5807497c3c2bb20e3d7d0fed6d9e42cc1` was inspected as source history. It
does not supply a comparable successful timing or prove the next cap sufficient.

## Final unapplied candidate review

Candidate: `budget-only-candidate.patch` in this scratch directory.
Final patch SHA256:
`7a950a9ee83d86d46e941f5ae16a46279e07670e72fb7591aa70298b13a0f58e`.
The candidate targets only `.github/workflows/research-book-pages.yml`.
Removing comment lines and reversing the single build timeout scalar from 120
to 90 reproduces the existing workflow exactly. The candidate therefore retains
every command, order, test, assertion, numerical threshold, pressure method,
trajectory length, per-method deadline, sandbox setting, artifact/deployment
dependency and error policy. A read-only `git apply --check` passed.

Original workflow SHA256:
`a4ab0fdddbd141d0323db8a2dd800c484c235dd23121028f5b9d14bfe20acc67`.
Prospective workflow SHA256 if that exact patch is applied:
`b1f6b5afd3be0e725808ddd9857460d5ebe111c68ee87061aeb8464dbc214f48`.

One accuracy correction was suggested and accepted: the explanatory comment
now says the run was **cancelled near 90 minutes**, rather than stating an
unconfirmed timeout cause as certain. No deciding review finding remains open.
The extra 30 minutes is headroom for the observed unfinished tail plus remaining
work, not an empirically established upper bound. One changed-cap run can test
that decision without weakening acceptance. An unchanged rerun cannot test it;
parallelizing generated-input/native/browser stages would require a larger
same-head custody and dependency review. If the bounded trial still fails or
expires, stop and re-plan rather than raising the cap automatically.

## Authorization and access boundary

The shared-workflow owner must coordinate application before a new exact head
is committed/pushed. The resulting normal CI must retain all required terminal
results and full artifact evidence; no merge is justified by this diagnosis.
The shared workflow remains unchanged at this review point. This reviewer wrote
only this scratch report, did not modify the repository diagnosis/report or
workflow, and ran no CI, scientific, numerical, GPU or browser workload.

The previously rejected check-run annotation and workflow-run-list endpoints
were not called, retried or reached through another route. Their rejection
leaves detailed cancellation annotation and historical-run comparison
unavailable. No reviewer action was denied or left with an uncertain mutation
outcome. Parent coordination, actual application and the changed-head CI result
are pending; the proposed patch is not an executed remedy.
