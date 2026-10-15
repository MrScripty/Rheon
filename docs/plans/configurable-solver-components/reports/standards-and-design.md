# Standards route and documentation review

Authority inspected directly: [MrScripty/Coding-Standards at
188beda1](https://github.com/MrScripty/Coding-Standards/tree/188beda1fa477d21d576c233dd8c7f4c4c267d23),
also its fetched current main on 2026-10-15. No ancestor or Rheon AGENTS.md was
found. No application standards-engine MCP tool is exposed in this session.
The standards repository contains engine/CLI source, whose application content
starts unqualified; its authoring skill targets standards maintainers rather than
ordinary application agents. No application qualification was run. This is
source-guided standards review, not an automated conformance certificate.

The actual outcome is authored research/design and a public documentation PR.
The proposal discusses future public/runtime/persistence boundaries without
changing their implementations. Core then STANDARDS-ROUTER selected Planning and
Discovery, required Implementation/Verification/Development Proportionality,
Documentation and Commit. Applicable design topics: Architecture, Code Design,
Contracts/Evolution/Protocols, immutable authority/replay, Dependencies, Licensing,
Concurrency, Resilience, Security, Diagnostics and Performance; IPC and Persistence
profiles guide the proposed contracts. No Rust/TypeScript/framework mechanism,
generated executable schema, build, release packaging or GUI behavior changes in
this PR. Those routes must be selected for actual implementation. Graphical/runtime
and numerical checks below remain future evidence, not silently passed exclusions.

## Decisions caused by the route

| Standards concern | Concrete design consequence / evidence |
| --- | --- |
| Core authority and configuration | Only host-qualified plan slots vary; adapter ports do not create missing physical fields or admit changed equations. Existing operators/acceptance remain authoritative. |
| Planning/lifecycle | [Plan](../plan.md) owns decisions, D1 scope and exactly one next slice; [ledger](../execution-ledger.md) owns outcomes, [issues](../issues.md) follow-ups. Documentation authorization does not confer Planned implementation status. |
| Architecture/Code Design | Full eight-part composed-design probe in plan: independent owners, change paths, hidden knowledge, replaceability and deletion/cumulative cost. Declines universal registry/graph/second archive/solver loop. |
| Contracts | [Typed design](contracts.md) distinguishes grid association, dimensions, units, stage/clock, lineage, optional context, candidate and final accepted state; decoders return invalid/unsupported/unavailable with complete cross-field proof. |
| Evolution | Stage, adapter, runtime, spec, archive and analysis versions are separate; historic accepted v1 interpretation is unchanged, new diagnostic records have a separate typed path under existing recorder ownership. |
| Immutable authority/replay | Freeze resolved spec and source/weight/config material; raw recordings immutable after finalization; comparison results pin exact publications. Inspection, rerun, environment reproduction and restart remain distinct capabilities. |
| IPC/security/lifecycle | Inert metadata discovery; trusted-local process, no sandbox claim; bounded structured requests, closed outcomes, stale-result rejection, worker-owned cancellation/descendant cleanup/reap, no numerical archive access from model. |
| Persistence/resilience | Use SQLite/StepRecorder/RecordingReader, explicit unrecorded tail and uncertain-commit reconciliation. No Debug-text replay parser, zero substitution, active-archive completion inference or recording rollback of accepted physics. |
| Dependencies/licensing | [Alternatives](research.md#11-alternatives-and-licensing-boundary) compare original typed contracts with checkpoint-specific adapters, FMI/BMI, arbitrary graphs, preCICE, ONNX/Wasm and ParaView. No framework, dependency, third-party code, weights or datasets incorporated. Later adoption audits exact licenses/transitives and scientific fit. |
| Verification | Numerical oracles, real worker/GUI, fresh-process persistence and environment-qualified benchmark evidence have separate claims. Equal-input capture and no-fallback diagnostic attempts are prerequisites, not features inferred from current API. |
| Performance/diagnostics | Named timing scopes, support/time coverage, rejected/attempted/selected/fallback state, actual intervals and sample distributions prevent misleading comparisons; no speedup or physical accuracy claim. |
| Documentation/Commit | Only authored Markdown in this plan directory; original report retained with current synthesis in plan/contracts. Exact staged and branch scope, local links, sensitive-file check and whitespace before ordinary commit/push/public PR. |

The [repository source/output policy](../../../REPOSITORY_OUTPUTS.md) excludes
generated fields/logs/plots/archives from commits. No solver execution or training
is needed to validate authored design prose. No future numerical acceptance is
claimed. The independent reviewer reads current source and prose and reports
material findings; their review does not transfer numerical implementation authority.

## Evidence observed in this environment

- Clean original `work` checkout inspected; fetched main and created a fresh docs
  branch without editing implementation. Source baseline matches the reviewed report.
- `PressureRequest`, dispatcher fallback, full-step/session publication, accepted
  recording schema/profile and desktop worker contracts inspected directly.
- Lab branch cloned read-only at the exact report revision. CPU bridge/model/export
  source is source evidence; no package import, inference or training was run.
- Reported integration branch was absent from remote branch discovery; fetching its
  exact reported OID returned `not our ref`. This limits M0, not the generic proposal.
- Primary FMI, BMI, ParaView and PETSc documentation reopened to verify the patterns
  used. Their concepts inform original design; no external framework/spec prose is
  copied into implementation and no numerical theorem transfers to Rheon adapters.
- [Independent review](review.md) and [ledger](../execution-ledger.md) record the
  deciding review findings and actual supporting validation/publication outcomes.

Research stopping condition is met: present source boundaries, adapter restrictions,
source-backed alternatives, concrete first contracts and unresolved integration
facts are identified. More framework survey, model execution or production prototyping
would not resolve the missing target and exceeds this task's authority.
