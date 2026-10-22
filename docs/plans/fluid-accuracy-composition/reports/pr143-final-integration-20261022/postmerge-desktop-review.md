# Independent actual-P desktop and hosted-runtime audit

Reviewer `/root/pr143_review`, 2026-10-22. PASS for actual-P desktop and all
currently triggered actual-P hosted checks. This is a read-only audit; no test,
workflow, scientific campaign or native specimen was rerun.

I independently fetched completed own desktop job119592284513 using the admitted
`github_fetch_workflow_job_logs` tool and retained the decoded log at a new
external path, `/workspace/pr143-postmerge-desktop-20a0e1e.log`. Its UTF-8 bytes
(including the leading BOM) total249,114, SHA256
`c2a07217e1a461a78993985b5fa00108ab2a78172f9dfc28db7b70630182e301`.
No existing raw log was overwritten and no artifact/ELF payload was downloaded.

The log explicitly fetches push commit
`20a0e1e20538b859193c08eaf25a3b8cd26135a5` (P) into origin/main, checks out
main and prints P from `git log -1 --format=%H`. Strict Clippy uses
`--locked --features desktop --all-targets -- -D warnings`; the ordinary
desktop test path reaches complete summaries and cleanup with no error marker.

Independent count separation:

- 91 top-level Cargo `Running` targets and91 top-level summaries:1156 passed,
 0 failed,20 ignored,0 measured,0 filtered.
- Four additional nested process summaries each pass1 test: two cold-rerun
  processes filter223 tests each, and two archive child processes filter11 each.
  They are excluded from top-level totals.
- One separate doctest summary passes3, with0 failures/ignores.
- Exactly the three original CV3 numerical preparation tests remain ignored
  with their frozen execution-admission reason. Neither those ignores nor the
  passing inherited unit checks establish nine-case scientific execution.

Fresh supported app GETs of the whole-commit check-runs collection and whole
workflow-run metadata independently show all six actual-P checks completed
SUCCESS: default, core-only, desktop, executable, Rust aggregate and dense scoped
contract. Rust run39707942188 and dense run39707942235 both conclude SUCCESS
on exact P; fresh branch-main API still equals P. Earlier independent raw audits
retain core764/13 and default772/13 with3 separate doctests each, plus executable
bootstrap7+7,193 passes/27 conditional skips and the actual CLI smoke assertions.
No source, event, compiled declaration, environment, binary or artifact identity
is inferred beyond those directly observed scopes.

This completes the named actual-P hosted-runtime evidence. Actual-P hosted book
and VTK remain NOT_TRIGGERED under their unchanged path filters. The sole lead's
prospective disposition6330943452 and continuation6331134413 separately own
fresh local VTK/book work, source/environment/output custody and independent
review. A VTK subcomponent pass cannot relabel a failed whole attempt. Any known
later local/static/PDF/browser failure remains blocking under that disposition;
bounded-lessons, upload/Pages and final-current-main full publication requirements
remain outside this runtime audit and are not silently accepted.

No unsupported singular-job metadata endpoint was retried; only permitted own
decoded logs and supported whole-commit/run GET metadata were read. Denied old
logs, artifacts and cloud routes were not accessed. Historical failed/cancelled
outcomes and pre-merge159/N receipts retain their own identities. No CV3 physical,
PDE/generic stability, moving geometry/GCL, hardware/native-window/GPU,
performance or whole-application production acceptance is claimed.
