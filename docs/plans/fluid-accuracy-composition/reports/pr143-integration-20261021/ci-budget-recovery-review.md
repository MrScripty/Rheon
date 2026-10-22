# Independent exact CI budget recovery review

Reviewed committed subject `79d5a82f1f14879aa249590212dd1683b06a4ef7` plus the
pending authored recovery documents. Reviewed document identities:

| File | SHA256 |
| --- | --- |
| `ci-budget-recovery.md` | `8d371af8479734cb474a6e290d645a21844fc149b55930bcb6c8eda32f5c01a0` |
| `ci-budget-recovery.json` | `ce2407fe84e51042288a8e6d8dba83736bc752e00acc223694acf08fb844eee9` |
| `hosted-ci-failure-39692570591.json` | `86f1534ae6067d2ab137a442ca0981791bd89c848d9eac0cf26f508fbf317cb9` |
| Pending `plan.md` | `860ebb3c3464901537f2ee4dfadef3d1d0dfb0e0f27eff454e958a841cdbed81` |

The retained decoded executable log has exact SHA256
`971e1790afe431ab5e9535bbf23feecfc422e90d13f9929e09d2c0f26384b8cd`.
It identifies checkout `7e99fa3bc06ed032ead003ea0b4557642a8ccee5` and shows
219 tests, two errors and 27 skips. Both error traces identify the exact feature
header comparison in `verify_split_ci_budget`. A read-only GitHub commit API
lookup independently confirms that checkout's tree
`21b9d6fba7713215f835d4c505fa70aca7d393ac` equals the actual tree of failed
head `41c68c80`. The failure remains a failure; other jobs are not inferred passed.

I inspected the actual 15-line helper/test diff. AST comparisons establish that
every prior top-level method outside the helper/class and every old
`CiBudgetContract` test method is unchanged. Within `verify_split_ci_budget`,
only the exact feature-header string changes: the reviewed two comments and
60-minute value replace 30. The remaining helper AST is identical. The sole new
test first accepts the actual exact header and then rejects 30, 59, 61 and a
missing value through the existing strict prefix comparison. There is no
normalization, range admission, assertion weakening or numerical-method change.

Current guard SHA256 is
`46a9d538ef4680e983099a3dfdcb6eb98b7ca87a5f8c8ec80e02d64508142960`.
Workflow SHA256 remains
`80f732505778a8f7746ca42c52e919c5d7d2cbdf0fe2432c8695079663c8e300`.
All 152 selected input hashes remain exact; production, Cargo, qualifiers,
private examples/tests, CV3 preparation payloads and admission records have no
delta against ordinary executed source `31c19585`. The changed source-only
workflow guard is outside that native admission selection.

Both retained local guard logs have their recorded SHA256
`8b1e10158d32c5c32e8b421ca010cadb069fa0183e175a960d0020ce21215aad`
and report four passing cases with zero skips. The recovery record separately
attributes normal and `-O` argv; identical log bytes do not independently prove
distinct invocations, and are not treated as such. The owner records the separate
executions. Source inspection confirms these four class methods create no
numerical fixture or Cargo/scientific workload. This review reran no test.

No blocking source, custody or reporting finding remains for this bounded
recovery. Approve committing the reviewed recovery documentation and publishing
the resulting head for fresh ordinary hosted qualification. This does not
accept the failed old head, transfer the `31c19585` receipt to changed source,
or authorize scientific execution. If the known-failed old run is still active,
its status must be checked after the new reviewed head is published before
scoped cancellation; every failure and cancellation stays recorded. Verified
PR142-then-PR138 main integration, final exact-head review, required hosted gates
and sole lead merge authority remain outstanding.
