# Carrier recording and cold desktop inspection

Status: Active. Current phase: delivery/exact-head CI.
Next slice: commit/push dependent source, open draft PR and report exact-head CI.
Acceptance: A1–A4 and local A5 satisfied; remote/source-publication gates reported separately.

## Authority, scope and standards

User authorized historical numerical retention, cold desktop opening and carrier
coverage, retaining accepted/rejected publication contracts and source qualification.
Parent owns serial merge/book admission; no PR67 branch/source/receipt changes.
Base: PR69 b1acf2e514b88b528cf43e873035a598a1080ae1. Separate dependent feature branch.
Standards: Coding-Standards 188beda1fa477d21d576c233dd8c7f4c4c267d23 and the routed
owners in ../accepted-step-recording/plan.md; same CPU/Rust/persistence/native UI
responsibilities. No new physics/GPU/remote scope, automatic deletion, checkpoint
or universal-field completeness claim. GPT-6.1 Sol High exclusively.

## Binding decisions and ownership

Recording owner retains native IEEE754 precision and per-field associations, units,
physical publication/evaluation stage and run/config/geometry/provenance. SQLite
schema/atomic accepted-step contract remains unchanged until the storage decision.
No per-cell SQL rows. Instrumentation may write only ignored generated artifacts.
Intended consumer initially selects one whole timestep; no spatial/time-series I/O
performance claim. Simulation remains authoritative and no rejected scratch pressure
can be published as accepted. Native UI may open protected local archives without a
live producer and must clear stale data on missing/corrupt/unsupported selection.
Cold archive display must not borrow current settings, collision mesh or emitter.
Headless UI tests qualify interaction state; a native window requires a display.

## Discovery and composed design probe

Facts: default 32³ carrier, 1024 steps; four f32 face/cell fields ~524 KiB/step,
plus optional native f64 interval pressure ->780 KiB/accepted step. 64³ ->6.05 MiB;
1025 full-pressure records ~6.05 GiB before SQLite overhead, exceeding default 1 GiB.
SQLite whole-field BLOBs versus chunked/compressed scientific arrays must be compared
against actual bounded whole-step access, synchronous latency and total capacity.
Report synthetic probe separately from physics; record host/cache/RSS limits.

Applicable architecture probe: archive owns transactions/integrity/limits; carrier
adapter owns authoritative layout and canonical completeness; simulation owns pressure
success validity; inspector owns exact selected identity and field/cell/face indices;
VTK owns its existing precision/layout metadata. One step identity packages all fields.
Public native adapter borrows canonical slices; no registry, asynchronous queue or
replay engine. Extension adds new producer inventory, not an alternate archive format.
Deleting pressure validity redistributes scratch-state admission to every caller;
deleting archive redistributes atomicity to each consumer. Localized policy boundaries
keep physics unchanged. Generic consumers can inspect absent fields explicitly.

## Milestones, writes and gates

D: instrument/decide, Accepted, operation verify. [Decision](reports/storage-decision.md). Writes: this plan directory,
`examples/recording_storage_probe.rs`. Gate:32³/64³ append/read timing, actual file
size and process peak RSS with qualified host/cache evidence; documented alternatives.
C: accepted carrier capture, Accepted, operation verify; D admitted raw SQLite blobs. Writes:`src/simulation.rs`,`src/lib.rs`,
`src/recording.rs`,`src/recording/carrier.rs`,`src/vtk_export.rs`,
`tests/recording_carrier_contract.rs`,`examples/recorded_carrier.rs`,
`docs/api/{recording,vtk-interchange}.md`,`tools/verify_recorded_carrier_vtk.py`,
`.github/workflows/vtk-interchange.yml`,plan directory.
Gate: exact fields/units/layout/times round trips for both desktop CPU pressure methods;
constructor pressure absent; rejected/cancelled scratch never published; limits,
corruption and selected-step VTK independently parsed/reloaded.
U: live carrier and cold desktop, Accepted, operation verify with C; compose their
acceptance because raw selection/export and per-track worker admission share the adapter. Writes:`src/bin/rheon-desktop.rs`,
`src/bin/desktop/{history,inspector,archive_view}.rs`, docs and plan directory.
Gate:per-track recording before preview, failure/tail disclosure, cold open/ordinal
selection/export/invalid-file clears; selected field and index never latest fallback.
V: fmt/Clippy/core/default/focused native UI, dependent commit/push/PR, exact-head CI.

## Acceptance and coordination

A1 storage decision has measured evidence and explicit bounds/access exclusions.
A2 carrier field precision/stages and rejected-step negatives pass.
A3 live tracks and cold desktop open/select/export use exact indexed records.
A4 invalid/corrupt/missing selected data clears views; no invented absent fields.
A5 appropriate static/focused regression checks pass; current-head CI reported honestly.
Source admission remains parent/publication-owner gate. Coordination comment on PR69
issuecomment-6158642258 lists added/changed admitted inputs; never auto-refresh pins.
No display currently available; native-window smoke blocked, headless egui permitted.

D acceptance satisfied by storage-decision report. C admission supports only native
Simulation.step (optional SmokeSource) and step_with_emitter, Jacobi/SGS PCG fixed
sealed-box methods. Constructor4 fields; accepted 5 including interval projection
pressure. Other boundary/provider/surface/volume/viscosity paths are unsupported by
this adapter. No fraction, masks, pressure scratch/residual vectors or emitter state
are invented. Pressure borrow success stamp is invalidated by every later attempt.

Local acceptance evidence: [verification](reports/verification.md). Source-only
feature delivery is authorized; no merge/publication before owner-qualified admission.
