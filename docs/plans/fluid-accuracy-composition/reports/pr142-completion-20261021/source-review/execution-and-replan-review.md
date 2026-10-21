# Independent execution, evidence replan and archive review

Reviewed on 2026-10-21 by delegated task `/root/pr142_source_review`.
The actual qualified source is commit
`dbd49d3e41c482a753884abc8aaaff7d1b0d0653`, tree
`2f37db7c119b846003f5a01a82d46c4cf965b474`. This review accepts the scoped
composite evidence below. It does not turn the cancelled all-local attempt
into a successful all-local run.

## Completed local evidence

Independently read the actual native receipt, specimen summaries, raw Rust and
normal/optimized Python logs, packet receipt and driver journal. All 1,903
receipt source SHA256 values were checked against the actual qualified commit's
Git blobs and, before later documentation edits, the clean current files. All
152 conservative native pins match the approved independent source map. The
fresh receipt binds the approved v3 inventory, review and historical records.

| Completed local claim | Actual result |
| --- | --- |
| Unchanged native/rational recipe | 18 commands, each exit 0 |
| Native Rust contracts | 7 + 16 + 10 + 2 = 35 passed, no failures, ignores or filters |
| Native specimens | Five actual TSVs and summaries; 11 rejected mutations per specimen |
| Qualification gate tests | 5 normal + 5 optimized, all passed |
| Independent oracle tests | 10 normal + 10 optimized, all passed without skips |
| Real packet consumer/rejection tests | 26 normal + 26 optimized, all passed without skips |
| Strict local core-only Clippy | Exit 0 |
| Complete local core-only ordinary tests | 722 passed, 0 failed, 10 explicit ignores, 85 top-level harnesses, 0 filtered |
| Strict local default Clippy | Exit 0 |

Native receipt SHA256 is
`c295a810ccc480bd5fd267179f89f9815a419c03fc3ad8f5211d2c188caa3bf7`;
packet receipt SHA256 is
`59a2c46d1bb0bd2ae2255ab80d146acd1744be6e7e74aa018b22f61218c1d434`.
The actual exporter and retained ELF copy both have SHA256
`4b2c44d880cbada3ac031f81b68c0a120b84aebd8c320f6d20d8d90dfd84943c`.
Packet primary receipt bytes exactly equal the fresh native receipt; TSV/data,
source, manifest and executable bindings pass. Native artifact names and all
artifact hashes match the actual output. No inherited-head receipt is consumed.
The completed local core log SHA256 is
`54bbe5bccc9eb45509ee215a6a5903ae29ba58bad9474c302f8e2c066feb7107`.

## Source-identical hosted evidence and explicit replan

Independently inspected supported-tool-downloaded run/job metadata and all
three raw feature-job logs from
[Rust run 39688034222](https://github.com/MrScripty/Rheon/actions/runs/39688034222),
attempt 1, actual head `dbd49d3`. The run completed successfully. Each feature
job's strict all-target Clippy and complete ordinary `cargo test --locked`
commands and steps passed with its declared flags.

All three logs record actual checkout
`d5ef6e22322183f34be437a1e9427a2b7d53d9d2`. Independently inspected its Git
object: parents are `15026a966265cf9eaedf464e2b5a066bd0fef3d8` and `dbd49d3`,
and its complete tree equals the reviewed source tree exactly. This establishes
source identity despite the distinct synthetic pull-request merge commit.

| Hosted ordinary feature suite | Job | Passed | Explicit ignores | Top-level harnesses |
| --- | --- | ---: | ---: | ---: |
| Core-only | 119526972309 | 722 | 10 | 85 |
| Default | 119526972867 | 730 | 10 | 86 |
| Desktop | 119526972766 | 1,114 | 17 | 88 |

Every top-level summary reports zero failures and filters. Desktop emits four
additional nested child summaries, each with one pass and a nonzero filtered
count; they are deliberately excluded from the table to avoid double counting.
The full suites include the real B2/B3 differential paths and all three
compile-fail documentation contracts. Long selected-producer intervals and
real fallback cases were observed passing. No error/failed-test marker appears.
Executable job 119526972855 and aggregate job 119536864453 also have successful
source-bound metadata; their individual raw assertion logs were not separately
audited by this reviewer.

The raw feature-log SHA256 values are:

| Job | Raw log SHA256 |
| --- | --- |
| 119526972309 | `836c1482f5da4eea789a7b89288d7cb03cbdade69d918cb99ddaeb0c1dbca13b` |
| 119526972867 | `a6b2a716c69de8a2bd581516c62464aaf96e360fe021999c98ca4a546bd8bacf` |
| 119526972766 | `bf80a828f9e7be4dde2d3add12955e464df2d65c5af6569354dc2fb2ff1806c9` |

Under Development Proportionality and Verification, these completed,
source-identical hosted feature suites satisfy the ordinary default/desktop
claims without duplicating the whole suites locally. Before cancellation, this
reviewer explicitly approved that revised recipe: completed local native,
packet and full core evidence plus completed hosted feature gates. No command,
assertion, explicit ignore, threshold or required gate changed. The original
protocol and driver remain frozen; this is an explicitly recorded new recipe,
not a retrospective claim that the original recipe completed.

## Intentional local cancellation

The original local driver completed every command through default Clippy. Its
duplicate default ordinary suite was deliberately stopped during
`material_contract-4052afe5fb62db1a` with SIGINT, after the above approval. Its
original journal remains `failed`, last command `tests-default`, exit 101.
The raw log records signal 2; no assertion panic or failed-test summary was
observed. Local desktop execution never began and is not claimed as passed.
The explicit cancellation/reconciliation records retain these facts separately
from successful completed evidence.

Independently confirmed that owned driver PID 7921, Cargo PID 22421 and test
PID 26575 no longer existed before authoring this report. Qualification source
guards were not bypassed by documentation or workflow edits during execution.

## Final archive audit

Independently read and audited the completed local archive
`/workspace/rheon-pr142-dbd49d3-qualification.zip` and seal receipt. The archive
is 153,125,915 bytes, SHA256
`4d55283ec9894b5c4c742e743913b6bc63fc8fbf413a21b6c6e6098d8e1fa04e`.
It has 191 unique members with no absolute/traversal names; ZIP CRC checks all
pass. Every member's actual size and SHA256 matches `MEMBERS.json`, whose
SHA256 is `c26fe09fa7d6961415e0113e510e3360c7cd6ce367e6cd187d0e7234ae359bd1`.

Rechecked all 1,903 archived receipt source hashes against qualified committed
blobs, all 152 selected pins, all native artifact hashes, packet primary
receipt/data/TSV associations and every recorded command log hash. All 131
retained executable entries in the original journal identify actual ELF bytes
matching their recorded SHA256. This is scoped journal/executable custody, not
an exhaustive identity claim for every operating-system child or rustdoc
temporary executable.

The archive preserves the stale-admission refusal, the initial zero-test
exact-filter attempt, successful exact fixture regressions, the
unchanged original driver, failed progressive journal, interrupted default log,
hosted evidence, and explicit cancellation plus reconciliation. Reconciliation
status `passed-scoped-composite-evidence` accurately describes the new recipe;
it does not overwrite the failed/cancelled original attempt.
The earlier preparatory E0063 compile failure is retained in the execution
ledger and tool transcript; its raw compiler log is not a member of this archive.

No material evidence or source-identity blocker remains for these scoped
claims. Later workflow/result-document commits require their exact diff,
unchanged native-source identity and applicable new-head checks to be verified.
Every receipt remains pinned to its actual source. Ignored tests are not passes;
native-window, whole-application production, new physical/scientific accuracy,
GPU/model deployment, performance and release acceptance remain outside these
contract results. This reviewer ran no Cargo, numerical campaign or graphical
execution and accessed no denied cloud task or artifact.
