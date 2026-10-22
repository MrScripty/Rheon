# Independent actual-P core/default hosted audit

Reviewer `/root/pr143_review`, 2026-10-22. PASS for the two named completed
actual-P hosted log scopes. Read-only raw-log inspection only; no test, workflow,
native fixture or scientific campaign was rerun.

| Retained own log | Bytes | SHA256 |
| --- | ---: | --- |
| `pr143-postmerge-core-20a0e1e.log` | 161146 | `dfa0a792191801fb0363bad3488e3d57ee1c450db1d7d16ccdbba06d172a3269` |
| `pr143-postmerge-default-20a0e1e.log` | 156729 | `539ebae0b458b59b10d7d63aaddc724c9ab8cc285b20166072e430e8cc3244e0` |

Both logs explicitly fetch actual push commit
`20a0e1e20538b859193c08eaf25a3b8cd26135a5` (P) into origin/main, check out
main and print actual P from `git log -1 --format=%H`. These are actual-P
executions, not pre-merge synthetic159 or earlier N results. P's reviewed full
source tree is d66c930d, as separately established by actual merge/source audits.
No equality of build, compiled declarations, environment, binaries or artifacts
is inferred from that source equality.

Direct counts, separating doctests:

- Core:88 Cargo `Running` targets and88 corresponding top-level summaries;
  764 passed,0 failed,13 ignored,0 measured and0 filtered. One separate doctest
  summary reports3 passed,0 failed/ignored.
- Default:89 Cargo `Running` targets and89 corresponding top-level summaries;
  772 passed,0 failed,13 ignored,0 measured and0 filtered. One separate doctest
  summary reports3 passed,0 failed/ignored.

No nested child summaries are present in these two counts. Strict core Clippy
uses `--locked --no-default-features --all-targets -- -D warnings`; strict
default Clippy uses `--locked --all-targets -- -D warnings`. Both paths proceed
successfully through their complete ordinary tests and cleanup, with no error
marker or failed Rust summary. The core path also reaches its dependency-tree
step. In each log, exactly the three original CV3 numerical preparation
contracts are explicitly ignored with their frozen execution-admission reason.
These ignores are not scientific passes or nine-case campaign execution.

The already audited actual-P executable path remains a separate result, and
desktop/aggregate completion remains pending at this checkpoint. Lead-owned
local VTK/book qualification under comment6330943452, its failed attempts or
successful substeps, and final-main hosted publication requirements keep their
own execution/outcome attribution. Actual-P hosted book/VTK remain NOT_TRIGGERED;
these core/default results do not replace or silently satisfy those claims.
Known later failures continue to block within the lead's disposition.

This audit accessed only the supplied permitted own logs. It did not retry the
previous unsupported optional singular-job metadata endpoint, use denied
logs/artifact/cloud routes or download artifact/ELF payloads. No native-window,
hardware, physical/PDE, performance or whole-application production acceptance
is claimed.
