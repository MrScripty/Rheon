# Exact CI budget contract recovery

PR143 head `41c68c80d1d4a64b219c10dd280d2544ef439ea3` produced a retained
hosted executable failure in [run39692570591/job119542065950](https://github.com/MrScripty/Rheon/actions/runs/39692570591/job/119542065950).
Actual checkout `7e99fa3bc06ed032ead003ea0b4557642a8ccee5` has exactly the head's
tree `21b9d6fba7713215f835d4c505fa70aca7d393ac`. The harness ran 219 cases,
with two errors and 27 conditional skips. Both errors compare the full feature
job prefix against the old 30-minute header; the reviewed workflow uses 60.
The source-bound [failure record](hosted-ci-failure-39692570591.json) retains the
decoded log identity and exact case names. No result is relabelled as a pass.

Normal merge of lead-reviewed repair `3cd735fde7655aedffbd0ffb2f041ca6994961a4`
is `79d5a82f1f14879aa249590212dd1683b06a4ef7`. Its only executable source change
updates the exact expected header and adds strict feature-budget mutations for
30, 59, 61 and missing values. Existing numerical-verifier methods, old test
methods, frozen eight-step baseline, commands, matrix, runner, other budgets and
aggregate gates remain unchanged. No range matcher, normalization or skip was
added. The source-only helper is outside the unchanged 152-input admission.

On that actual clean merge subject, all four `CiBudgetContract` tests pass
normally and under Python `-O`, zero skips. The class creates no numerical
fixture; no Cargo command or scientific campaign was run for this repair.
Python 3.12.14 and the existing Pillow 12.3.0 import were observed. Both logs
happen to be byte-identical; their commands and interpreter flags are recorded
separately in [recovery custody](ci-budget-recovery.json).

The original successful ordinary qualification remains bound to `31c1958` and
its own scope. The failed composed hosted run remains failed; other job outcomes
are observed separately. Publish the reviewed repair on a new head for fresh
ordinary hosted qualification, then verify actual PR142-then-PR138 main,
complete source/admission identity and final exact-head review before the sole
lead's normal merge. Original cancelled attempts, preparation seals and scientific
limits remain unchanged; no CV3 scientific or production acceptance follows.
