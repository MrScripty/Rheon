# GPU grid-fluid research book expansion

Status: Accepted documentation deliverable. Production implementation is excluded.
Current phase: terminal local research handoff; parent integration remains separate.
Next slice: none for this worker's accepted documentation scope.

## Objective, constraints and admission

Expand Puma's existing research book with substantial GPU algorithms grounded in
Rheon's current CPU solver, and compare independently usable CUDA and Vulkan
compute paths. Deliver source inventory, primary-source map, chapters, comparative
architecture and reviewable future validation/implementation stages. User
authorization admits this research work, isolation and independent review; it
does not admit production backend work, merge, deployment or auto-publication.

Fetched main/base: `fd7c0f0aee63bc8710df99f9bb01154e7619b974`.
Branch: `research/gpu-grid-algorithms-20261021`.
Worktree: `/workspace/rheon-gpu-research`.
Owner: GPU research worker; parent owns integration and later draft-PR
coordination. Original `/workspace/Rheon` checkout remains preserved. PR145,
PR146 and Lagrangian FEM are outside scope. No shared numbering, TOC, build or
existing reference edits are authorized by this worker's ownership.

Keep source/field identities, original credit, historical evidence and failed
criteria. Do not run scientific/GPU/hardware workloads or install toolchains,
drivers or modify security settings. Only nonnumerical documentation, link,
format and source-scope checks may execute. Current primary sources must be read;
unavailable source or capabilities stay explicit. No benchmark or proof is
fabricated. GPT-6.1-sol at high reasoning is used for delegated CPU/source and
technical review work as requested.

## Owned artifacts and decisions

The [book source map](../../research-book/implementation/gpu-research-plan.md)
lists the exact new book paths. This plan directory owns plan, execution ledger,
issues and review/standards reports. Only these new GPU paths may change.

The CPU inventory owns claims about existing source. Chapter 37 owns algorithm
comparison, Chapter 38 owns independent backend mapping, Chapter 39 owns future
validation and roadmap. Bibliographies own source provenance. No implementation
contract is silently admitted by these prospective documents.

Composed-design review: applicable to the proposed multi-backend architecture;
documentation changes no runtime composition. Chapter 39 and the following probe
record the candidate design to be challenged in later implementation planning.

| Architecture probe | Research disposition |
| --- | --- |
| Independent what/who/how/when/where/why | Numerical owners select equations/field stages; CUDA and Vulkan independently schedule on their devices; sessions commit complete candidates; recorders/renderers consume accepted versions. |
| Necessary and accidental interleavings | Numerical stage dependencies and publication fences are necessary. Renderer timing, archive serialization, API objects and optional interop must not define the physics. |
| Caller/composition knowledge | Select admitted problem, backend capability/arithmetic policy, resource/cancel/output budget. Do not expose launch/block/barrier details through physical controls. |
| Representative change propagation | New preconditioner affects method/qualification; archive codec affects recording; Vulkan barrier fix affects Vulkan execution; new physical traction affects numerical contracts/all implementing backends/oracles. |
| Stable values versus hidden knowledge | Grid association, units, interval/generation, geometry and acceptance are stable meanings. API handles, descriptor/graph lifetimes and native layouts remain backend-owned. |
| Replaceability and deletion | Each backend can be disabled/removed independently; CPU remains reference. Vulkan must build/run without CUDA; CUDA compute must not require Vulkan. No shared opaque GPU execution object is proposed. |
| Cumulative cost and alternatives | Start with dense Jacobi PCG; defer sparse metadata, graph optimization, multigrid and optional interop until correctness/measurement. Compare pressure-only host provider prototype with resident full step and disclose transfer costs. |
| Simplest coherent artifact | Three complementary research chapters plus source inventory/bibliographies. No generic runtime registry, new archive, shader/kernel code or production abstraction is introduced. |

## Milestones and acceptance

| Milestone / write set | Goal and gate | State |
| --- | --- | --- |
| M0 / new source inventory and plan | Fetch actual main; inspect guidance, book gaps and build scope; preserve unrelated work | Complete |
| M1 / new chapters and bibliographies | Equations/assumptions, pseudocode, comparative independent backends, capabilities, precision, lifecycle and roadmap, grounded in primary sources | Complete |
| M2 / new review report and owned repairs | Independent technical and source review; address deciding findings; nonnumerical local links/format/scope checks | Complete; [review](reports/independent-review.md) has no unresolved deciding findings |
| M3 / ledger and parent handoff | Exact artifacts/branch/commit and evidence limits; coordinate numbering/shared integration and a later draft PR through parent | Handoff prepared; [draft PR text and constraints](reports/draft-pr-handoff.md), no remote publication |

Objective acceptance is documentation-level only: (A) actual CPU source and
capability limits inventoried, (B) substantial principles-first GPU algorithms
and independent mappings, (C) current primary sources with provenance and license
boundaries, (D) explicit validation/benchmark/lifecycle roadmap, and (E)
independent review and nonnumerical checks. Current acceptance: satisfied locally.
GPU speed, hardware support, numerical parity/convergence and book render/build
are prospective, not failed or passed objective claims.

Re-plan when source main changes relevant CPU semantics, another worker claims
numbering/files, source evidence contradicts an algorithm/device claim, or the
scope asks for executable backend/qualification. Stop an affected denied action
and report exact target/reason/outcome. Continue independent authorized work.

## Retained review resource and terminal disposition

The branch/worktree are retained for parent source review and draft-PR
coordination, not integrated. Parent owns final numbering/shared edits and
eventual integration or rejection. No automatic rebase or history rewrite;
refresh relevant main differences read-only and reconcile through a separately
reviewed change. Supported consumer: parent/reviewer only. Retirement condition:
parent records integration/rejection and disposes of the clean isolated resource
under repository lifecycle rules. No merge, push or PR is implied by local
documentation acceptance.

See [ledger](execution-ledger.md), [issues](issues.md) and
[standards report](reports/standards-and-scope.md),
[independent review](reports/independent-review.md) and
[parent draft-PR handoff](reports/draft-pr-handoff.md).
