# Prospective control analysis

Before any final timing, the independently reviewed analyzer implements the frozen
per-case uncertainty rule. It runs no benchmark and accepts exactly the existing
runner's12balanced shuffled pairs for each1000/4096single-worker case. Numeric
comparisons use unrounded recorded deltas. Invoke once on the retained final
particle-summary.json with a new output path:

```
python3 docs/plans/accepted-pressure-warm-start/integration-20261021/analyze_default_controls.py EXTERNAL/particle-summary.json EXTERNAL/paired-analysis.json
```

Exit0 means both cases meet the frozen strict rule; exit2 means a valid cohort
is adverse or inconclusive, with its output preserved. Invalid/missing data fails.
Never extend the cohort, remove pairs, overwrite results or substitute construction
evidence. Pointwise96.142578125% interval coverage is conditional on independent
representative pair observations and does not establish host stationarity or
simultaneous coverage. Source/binary/runner/host provenance and all raw outputs
must also be checked separately; this analyzer does not grant merge, whole-file
identity, universal performance or production acceptance.

Initial independent review noted the prior CLI's blocked-result exit0; its report
is preserved. The reviewed two-line follow-up returns2 for blocked outcomes without
changing any mathematics. Final frozen analyzer SHA256 is
`e42977351b65c2886f2b5a59429e1e1418651662c9c21c43a0c7206134e2b1f9`.
Syntax/static checks only; no real timing data processed or synthesized.
