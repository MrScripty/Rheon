# Density acceptance limitations and follow-up

| Item | Owner and disposition | Reopen condition |
| --- | --- | --- |
| Source-stop RHS compatibility at 1000/m | Numerical owner follow-up; retained failure. This configuration change does not alter kernels, tolerances or compatibility admission. | A separate prospective numerical analysis and independently reviewed remedy, with original failure preserved. |
| Missing measured A1 package/runtime/custody | Original artifact owner; unchanged blocker. Configuration, analytic fixtures and zero-flow custody are not learned-model qualification. | Exact inputs in the [existing owner checklist](../configurable-solver-components/reports/measured-a1-owner-inputs.md). |
| Dense 10000³ allocation and larger Appearance | Explicitly unsupported by ordinary resource budgets/policy. Settings describe axis bounds, not capacity. Stored previews and numerical geometry are never silently reduced. | An independently owned resource/renderer design with new bounds and validation; no cap bypass here. |

The initial source-stop trial requested 8³ cells at 1 mm spacing, dt 1e-6 s,
two intervals and source active for the first interval. Jacobi accepted
generation 1, then rejected with `Pressure(IncompatibleRhs)`:
sum `2.03378186350074e-23`, rounding budget `1.7512013017958464e-25`.
The actual saved terminal records outcome `failed`, recorded count 2 (constructor
plus generation 1), accepted generation 1; the second comparison member never
ran. Do not count this as a successful two-step or model test.

The task-owned original terminal and SQLite prefix remain under
`/tmp/rheon-saved-worker-386115-1792135287085232231/` in this executor. That
working-source trial used an unstamped developer build; it is not a final-head
qualification receipt. Independently reviewed unchanged numerical bytes and
explicit physical intent bound its diagnostic relevance. A continuously forced
16³ CLI test at 1000/m passes separately; zero-flow cold reruns establish exact
geometry/storage custody with different previews. These claims remain distinct.
