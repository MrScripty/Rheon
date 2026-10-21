# Independent exact CI budget guard repair review

PASS, delegated task `/root/pr142_source_review`, 2026-10-21.
Reviewed the complete six-path repair over reporting source
`5eb8bf5607c01f1d59242c604b1d1204882d2eac`; this review adds a seventh file.

The actual failed hosted executable log SHA256 is
`aec251631819cb28b421ef2e517ec4e13dd697949ba6c1328d40e2ef5e4cd404`.
It records checkout `7ce8a40` merging actual reporting head `5eb8bf5` into
`15026a9`, 219 Python tests, 2 errors and 27 skips, exit 1. Both errors are
the exact stale `feature-contracts` prefix check in `CiBudgetContract`.
No numerical-verifier test failure was observed. The failed source/result
remains recorded; the repair does not waive or relabel that gate.

Actual guard source SHA256 is
`46a9d538ef4680e983099a3dfdcb6eb98b7ca87a5f8c8ec80e02d64508142960`;
its Git diff over `5eb8bf5` has SHA256
`a0a861c2ac529da6492ee2f0185266c13d6f720d7619a162e0aae7fd1719300d`.
Independent source/AST comparison establishes that the sole existing verifier
change is the exact approved feature prefix: two comments and timeout 60.
Every old test method and every numerical-verifier method is unchanged.
The new fourth source-only test checks the unique feature budget and requires
the feature-specific ValueError for 30, 59, 61 and missing-budget mutations.
The existing all-30-to-60 negative remains, guarding executable/aggregate
budgets. No range, skip, assertion relaxation or generic normalization is added.

The frozen eight-step baseline, workflow, setup, action pins, workload commands,
feature matrix, aggregate and the other two job budgets remain unchanged.
The guard file lies outside the conservative 152-input native closure; all 152
actual hashes and frozen admission records remain exact. This is coordinated
contract maintenance for an already reviewed workflow change, not a weaker
acceptance criterion.

Read both actual focused logs: 4 tests normal and 4 under Python -O, all passed
without skips. Their SHA256 values are
`f4b0bc8cee953e5b29e79e1e007c4d093d73853a350ee94051b37b6500685085`
and `b835a38207a75d7d44c75f3d31d1727f585e5fc31f4e6c2559058accb76f3f6a`.
The selected class constructs no numerical fixture. This reviewer ran no Cargo,
full numerical module, sealed campaign or graphical execution.

The new repair Markdown/JSON, appended qualification summaries and ledger
accurately bind the failed job 119541101644/run 39692282682 and exact repair.
All prior qualification summary fields except status are preserved, with the
new failure/repair record appended separately. Original `dbd49d3` receipts,
cancelled all-local attempt, Library storage failure and sealed archive identity
are unchanged. Fresh next-head full CI is explicitly pending.

Reviewed file hashes:

| File | SHA256 |
| --- | --- |
| Plan-root `execution-ledger.md` | `08c1ab4a236ecad86f48fff4dc4fdcdeb7239633db39bb3e9cba667c302cd9b2` |
| `ci-budget-guard-repair.json` | `c6f444c9ba2b8ff4346d9ba4991320df49304fbfa3326a9bdba96be80579fce5` |
| `ci-budget-guard-repair.md` | `0a1890dfe54bad3a320db84377c42a2fa6231d0d00d0e454455cb9896e87a526` |
| `qualification.json` | `1cb6d2a771ebb0a3bcd4007f9ea640fe2070d2e41bdb347faf257631c648eacc` |
| `qualification.md` | `ea7c8f07a31977f30fa564fcd766b1c3c3ed5b92a9a17b3ecb349b8557c1b655` |
| `tools/test_column_interface_verifier.py` | `46a9d538ef4680e983099a3dfdcb6eb98b7ca87a5f8c8ec80e02d64508142960` |

Approve committing this exact repair and report set. No material source,
contract or reporting issue remains. Fresh full checks on the actual new head,
normal expected-head merge and post-merge verification remain required.
