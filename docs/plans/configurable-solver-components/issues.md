# Findings and dispositions

| ID / severity | Evidence and relation to objective | Owner / disposition | Verification / revisit trigger |
| --- | --- | --- | --- |
| Q1 / blocker for M0 | Reported integration branch and `66e50388ec7a788dbda0ad1227e330996edf506e` cannot be fetched; no claim to review it | Parent/integration owner: provide actual accessible source before implementation admission; documentation proceeds | Fetch exact actual revision and reconcile changes, ownership, tests and existing deployment protocol |
| Q2 / high admission constraint | Main full-step provider excludes viscosity; lab deployment declares 0.001 Pa·s; scalar spacing/no-origin and strict rho/dt limits | Numerical + lab owners: defer admission, preserve exact envelope | Qualify physical plan or select a genuinely matching model; reject unsupported viscosity/origin/spacing/actual dt before inference |
| Q3 / high M3 dependency | Current accepted records omit RHS/residual; current provider rejection falls back and overwrites candidate | Numerical/API owner with recording owner: bounded stage capture + no-fallback diagnostic attempt required | Equal-request identities and retained rejection evidence; current production call cannot masquerade as diagnostics |
| Q4 / high M3 dependency | Accepted archive v1 cannot record rejected diagnostic trajectories | Recording owner: additive versioned schema/typed producer-reader, not weakened accepted decoder | Cold reopen rejected attempt separately; invalid diagnostic never accepted; continuation requires separate advance contract |
| Q5 / medium future constraint | Existing provider has no hidden-state rollback contract | Lab + session/runtime owners: explicit-context stateless first; defer recurrence | Final-gate rejection/retry preserves context, RNG/cache state when admitted |
| Q6 / medium trust/target constraint | Same-user subprocess is not a malicious-code sandbox; platform cleanup not observed here | Parent/runtime owner: confirm trusted-local profile at admission; reroute if stronger isolation required | Runtime crash/hang/cancel/descendant cleanup on each supported target; no Linux-only cross-platform claim |
| Q7 / medium reproduction constraint | Current archive is inspection data; opaque Debug configuration is not executable recipe | Application/configuration owner: typed spec revisions; no resume-from-frame | Fresh-process inspection without model, explicit rerun material availability, complete checkpoint only by future separate admission |
| Q8 / medium comparison constraint | Adaptive dt can leave all independent accepted intervals unmatched | Analysis + numerical owners: explicit unmatched coverage and at least one useful paired matched-interval fixture | Prove a nonempty native-field comparison with exact source publications; never fix coverage by silent interpolation |
| Q9 / low publication limitation | No exposed application engine MCP tool; local engine source/application qualification not executed | Documentation owner: source-guided route, no certificate claim | Revisit if a qualified application route becomes available or implementation changes selected routes |

These are a bounded missing-capability/authority family, not a claim of defects in
the inaccessible branch. Reachable consumers are numerical stage/session, adapter/
runtime, recording, saved experiment resolver and analysis/UI; the plan assigns each
one a disposition. No sibling API/code repair is authorized by this documentation.

## Parent draft reconciliation

The supplied draft issue record is incorporated into the owning rows above:
R1→Q1 (missing source); R2→ledger (execution setup now available, historical failures
not accepted); R3→Q2 (viscosity); R4→Q2 (spacing/origin); R5→Q4 (diagnostic recording);
R6→Q4 (separate diagnostic advance); R7→Q5 (provider state); R8→Q7 (rerun/restart).
Q3/Q8 add findings from this independent repository-side review. This crosswalk
retains draft dispositions without creating a second issue authority.
