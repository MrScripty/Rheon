# Standards route and first-slice design

Authoritative source inspected: MrScripty/Coding-Standards
`0fc92a24279f3a4574c80af0a2f31ea347a21597`, current main on 2026-10-20.
Rheon's recent fluid-accuracy plan uses the same pin; older plans use `188beda1`.
No application MCP or executable engine certification is claimed. The standards
authoring skill is not applicable to ordinary application edits. No standards
source/configuration is changed.

Read Core then Router. Applicable workflows: Planning with accepted audit as
discovery input, Implementation, Verification/GUI, Development Proportionality,
Commit (isolation/history ownership), Documentation (plan and handoff). Rust
profile and Rust API profile apply to native egui code and the coordinated
internal form/action method split; no externally supported Rust API changes. Accessibility, Diagnostics, Architecture
and Code Design apply to interaction/hierarchy, status projection and composition.
Existing Contracts/Concurrency invariants are preserved at call sites; no public
API, IPC, persistence, native build adapter or dependency changes are admitted.
Frontend web profile does not apply to native egui. Concurrent Plan Integration
does not apply: one lead serially owns the plan; independent review reports do
not authorize or concurrently mutate its decisions.

| Obligation | Concrete design/evidence |
| --- | --- |
| Planning admission/lifecycle | Canonical plan path, start/continue binding, one next slice, explicit native blocker, milestone write sets and rollback |
| Proportionality | Accepted audit settles direction. Implement bounded reversible presentation after one independent plan review; avoid speculative framework rewrite or further broad UX research |
| Ownership/composition | App selects form and routes existing launch; ConfiguredControls retains readiness; Worker retains all actual work/cancellation; no duplicated worker state |
| Contracts | Setup switching does not mutate physics, clear archives, publish intent or launch; all consumed shared inputs remain reachable; validation/immutability unchanged |
| Diagnostics | Full existing error/status remains available; no text parsing to infer success/severity; integrity and completion retain distinct owners |
| Accessibility | Named mode/action controls with native keyboard activation and preserved units; moving focus/hiding active inputs requires a deliberate transition; actual native focus remains evidence need |
| Verification | Real egui input for selection/form/action projection plus source/static checks; native perception/scaling/keyboard/cancel coverage separately blocked |
| Rust quality | Pinned 1.92, locked dependency resolution, affected formatting/strict Clippy and focused UI tests; no execution of numerical test suite |
| Repository integrity | Isolated main-only branch, no Inspector/README/agent changes, generated evidence outside Git, no remote mutations |

The first native observation will decide only whether the chosen fixed action placement
and form height work at supported sizes. Missing display does not prevent the
independently useful workflow separation; it prevents accepting the visual claim.
