# CAD live inspection and VTK interchange

Status: implementation and local review complete; final core/hosted checks pending. Goal: fix overlapping/narrow native inspection UI, expose
real live numerical fields without mixing evaluation stages, and export reusable
VTK data that independently round trips. Continue under the user's explicit
implementation, push and draft-PR authorization. No main merge or deployment.
Canonical plan: `docs/plans/cad-live-vtk/plan.md`.

Dependencies: PR50 `3f49a15`, PR47 `1923d5b`, PR48 `f06a127`, plus combined PR53 `ec9d960`, composed on
`codex/cad-live-vtk`. Shared owners retain their contracts: session owns transport
snapshots; inspector/particle solver owns particle diagnostics and their stages.
The CAD owner owns presentation, encoding, export UI and validation. No alternate
public/general snapshot schema or physics contract is introduced.

Standards: MrScripty/Coding-Standards `188beda1fa477d21d576c233dd8c7f4c4c267d23`.
Read Core/Router, implementation, planning, verification, Rust, accessibility and
schema ownership. Native egui uses no browser or web frontend; no browser evidence
or standards-engine certificate is claimed. Existing source/output hygiene,
locked dependencies, denied unsafe, formatting and Clippy remain applicable.

## Acceptance and sequence

1. Dock inspector in a scrollable sidebar tab. At 900×650 and 1250×820 controls,
   fields and viewport do not overlap; footer/status text wraps or uses bounded
   short labels. Verify actual egui input and native screenshots.
2. Live particle view consumes PR47 accepted positions/velocities, supports speed
   coloring and picking, and identifies latest-state independence from scrubber.
   Pre-integration pressure remains inspector/export only with its own timestamp.
3. Transport session view consumes PR48 `SessionSnapshot`, advances actual solver
   state through bounded controls, exposes slices/probes and exports that exact
   publication. Constructor pressure is unavailable; accepted projection pressure
   has interval/gauge semantics. Existing carrier-emitter mode is not silently
   replaced because the session facade lacks emitter controls.
4. VTK .vti and .vtp writers consume existing source types, preserve coordinates,
   associations, units, native precision, identities, stages and validity. No
   synthetic field fallback; writer/invalid-input failures remain failures.
   UI exports avoid overwriting files and report success only after completion.
5. Independent VTK reader checks asymmetric geometry, x-fast order, raw fields,
   reconstructed vectors, Unicode metadata, stages and absent diagnostics.
   Focused regression suites, serial desktop suite, Clippy, native screenshots,
   independent review and hosted checks gate the draft stacked PR.

Next slice: finish release core suite, push draft stack, verify hosted checks.
See reports/verification.md for completed acceptance evidence.

Write sets: root owns desktop modules, app composition, src/lib.rs wiring and this
plan. Export delegate owns src/vtk_export.rs, tests/vtk_export_contract.rs,
examples/vtk_export_fixture.rs, tools/test_vtk_roundtrip.py and focused workflow.
No source changes to src/session.rs or numerical solvers are planned.
