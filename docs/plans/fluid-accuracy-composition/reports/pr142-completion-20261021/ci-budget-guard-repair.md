# Exact CI budget contract coordination

The first reporting head `5eb8bf5607c01f1d59242c604b1d1204882d2eac` exposed a
recoverable source-contract omission: the reviewed outer feature-job budget was
60 minutes, but `tools/test_column_interface_verifier.py` still required the exact
old 30-minute feature prefix. Executable job 119541101644 in run 39692282682
failed two `CiBudgetContract` checks with
`runner/budget/matrix/dependencies: feature-contracts`. Its full ordinary Python
run reports 219 tests, 2 errors, 27 skips; no numerical-verifier case failed. The
successful smoke export, original failed job/log and outcomes remain retained.
Actual checkout 7ce8a40 merges that reporting head into main15026a9.

The bounded repair updates only that source-only contract's expected feature
prefix to the exact approved two comments and 60-minute line. The historical
frozen eight-step workflow snapshot remains byte-identical. Every workload,
assertion, action pin, bootstrap, matrix, dependency, executable/aggregate 30-minute
budget and numerical verifier remains exact. No generic range, arbitrary
normalization, conditional skip or success fallback is admitted.

A new source-only regression proves the actual feature prefix contains exactly
one 60-minute budget, then refuses 30, 59, 61 or missing budgets with the specific
feature-prefix diagnostic. Existing negative cases still reject deleted tests,
changed flags/limits, early/skip success, setup drift and changes to the other
jobs' budgets. The four complete `CiBudgetContract` cases pass normally and
under Python -O, with no skips. They construct no numerical fixture and rerun
no sealed campaign.

The unchanged conservative native selector still has 152 exact inputs; this
workflow-source guard is outside that closure. Actual native/packet/core
receipts remain on dbd49d3, and the prior failed 5eb8 job is not relabelled. The
[machine receipt](ci-budget-guard-repair.json) binds the failed log, exact new
source and focused checks. Independent repair review and fresh full checks on
the next exact head remain required before normal merge.
