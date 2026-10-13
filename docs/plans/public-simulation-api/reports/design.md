# Standards and composed-design record

Public standards source: https://github.com/MrScripty/Coding-Standards/tree/188beda1fa477d21d576c233dd8c7f4c4c267d23
Retrieved by authenticated read-only Git, after the parent clarified the external
location. Initial source work preceded this discovery; this record explicitly
corrects the initial assumption that local absence settled applicability.

Read Core and Router and selected Planning, Implementation, Verification,
Development Proportionality, Commit, Documentation, Library, Rust and Rust API;
Architecture, immutable-result closure, Contracts/evolution, code design,
Concurrency, Resilience and Performance apply to the published owner semantics.
No generated binding, FFI, network service, GUI, release packaging or deployment is
introduced. Existing JSONL encoding is reused; no stored format or generator changes.
The application-purpose Standards Engine is not exposed in this environment;
source reading provides policy evidence, not a claim that an engine-approved
application route or automated standards certificate was obtained.

Composed-design review: applicable; all eight Architecture probes follow.

1. The session owns synchronous lifecycle and publication views in Rust. Existing
   liquid physics owns fields/numerical acceptance; exporters own dataset completion;
   hosts own event loops and retained copies. Each can change for its own reason.
2. State/report/frame must publish together. Run/source labels reference caller
   identity; they are not physics inputs or dataset provenance. No incidental coupling
   to a renderer, provider runtime, service or engine is introduced.
3. Callers know constructor budgets, typed controls, accepted dt, borrowed snapshot
   lifetime, terminal stop and retry semantics. The owner hides workspace wiring and
   report/frame association. No undocumented background worker or clock exists.
4. New display fields touch the inspection/renderer owner and documented adapters;
   changing numerical acceptance touches physics; changing lifecycle touches session
   and its public tests. Adding a particle backend requires its own view and contract,
   not unrelated changes to dense3D schema or this transport implementation.
5. Inputs/reports/views are existing stable domain types. Private concrete ownership
   hides workspace ordering. The reusable encoder carries an explicit wire contract;
   no caller depends on session layout, private counters or filesystem locations.
6. Physics tests and session lifecycle tests remain independent. Export writer failure
   does not poison physics; physics rejection cannot publish a snapshot. A future
   provider can fail only under its declared backend acceptance boundary.
7. Deleting the session moves workspace ownership, report association, step quotas,
   deadline handling and lifecycle state into each host. Deleting borrowed snapshots
   loses enforced view lifetime. Deleting capability/identity metadata forces consumers
   to infer units and source. No registry, worker framework, ABI or new wire version
   was added that could be deleted without losing an admitted requirement.
8. Inherent complexity is the existing transactional carrier/liquid pair and staggered
   fields. This module adds one synchronous owner and borrowed view. Independent dataset
   completion remains with Dense3dSequence; no second manifest authority is retained.

Acceptance oracles: native transactional state/report bytes before and after rejected
operations; actual accepted dt vs requested dt; direct existing encoder equality proves
wrapper compatibility only, not independent validation of encoder correctness. Existing
dense3D tests provide its numerical/wire checks. Review found no blocking issue.
