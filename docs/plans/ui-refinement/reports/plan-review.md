# Independent plan and design review

Reviewed 2026-10-20 by the independent `ui_plan_review` agent. Production base:
`fd7c0f0aee63bc8710df99f9bb01154e7619b974`. Subject: the initial UI refinement
plan and standards/design report, before production implementation. Initial
plan SHA256: `bf0f3f2807386f1a22162aaeead908a8ee77509b95ef9742b40fe3cf1d43ed02`.
Initial standards/design report SHA256:
`86dd954abfdaa0249de457e7786ee40c207cf9ad0dfd5887c9aaad726937026e`.

## Disposition

The direction, main-only dependency strategy and bounded first production slice
are suitable. Start M1 after incorporating the conditions below; no further
optional user approval or broader design research is needed. This is a source
and plan review, not production acceptance or standards-engine certification.
Native visual, focus and ongoing-worker interaction claims remain unsatisfied.

The lead accepted PR1 and PR2 before implementation: collect launch/cancel
intent, dispatch after form painting/readiness invalidation, and preserve full
status behind keyboard-reachable bounded details with a concise summary. The
lead also committed to effective-workflow carrier controls without mutating
retained material mode and to structural plan alignment. These are accepted
design dispositions; their implementation/test outcomes require later review.

## Findings and implementation conditions

| ID | Severity | Finding | Required disposition / evidence |
| --- | --- | --- | --- |
| PR1 | Medium | Moving the action area ahead of scrolling form painting can make execution observe the previous form values when edits and activation occur in one input frame. The configured package's readiness/invalidation remains owned by its form. | Paint the footer to collect intent, process current form edits, then execute the selected launch with existing current readiness/validation. Do not start work from footer painting before the form is processed. Add an inert UI/input regression or adequate construction/source proof; never launch a solver/model to test this ordering. |
| PR2 | Medium | An unbounded full status/error label in a fixed bottom area can grow until setup, status detail or Cancel is clipped, particularly for long local paths and validation errors. Merely moving the existing label does not establish persistent usable controls. | Bound status presentation geometry while retaining a visible meaningful cause and explicit keyboard-reachable full text/details. Preserve the complete original status and do not parse text to invent severity/completion. Exercise long/multiline feedback at supported sizes with egui layout assertions; retain native verification as required evidence. |
| PR3 | Low | The initial draft expresses the required plan facts in prose/table equivalents but does not yet match the available source `validate_plan` structural projection (exact headings/fields, separate Status/Evidence columns and named design probes). | Lead has identified this and will align the canonical artifact and invoke the pinned source check. A structural pass supports only plan shape; it is not application qualification or evidence that the design is correct. |

The selected-workflow implementation must also enforce a specific existing
source constraint: `start_configured` uses carrier counts and pressure controls
regardless of retained preview `material_mode`. Accepted Session controls must
therefore render carrier semantics even when preview material mode remains true.
Do not change `material_mode` merely to reuse the old form. Verify shared domain,
grid/count-or-density, steps, maximum dt, pressure method/tolerance, display,
recording and CPU inputs remain reachable, retain values and feed their current
owners. This elaborates the plan's existing binding decision rather than adding
a new numerical feature.

## Source and standards observations

- Inspected actual App sidebar, controls, `start_configured` and configured
  launch-input construction, plus `ConfiguredControls::show`, package readiness
  and path/provenance invalidation. The original simultaneous forms and launch
  actions substantiate M1's hierarchy change.
- `App` is the correct owner for setup selection and routing. Splitting form
  painting from launch intent retains configured package authority and the
  existing worker/immutable-intent lifecycle. No registry, alternative worker
  state, new schema or compatibility layer is needed.
- Exact M1 production write boundaries exclude Inspector, D1 comparison,
  numerical kernels, root README and agent files. Main-only work avoids
  speculatively stacking the separately validated Inspector slice. Reconcile
  its final revision before M4; do not edit its test-only Clippy repair here.
- The plan names one next slice, separates objective acceptance from local
  supporting gates, provides explicit rollback and keeps later write sets
  disjoint with serial re-admission for composition-root changes.
- The 40% word target is appropriately measured over matched contemporary
  states, not asserted from source counts or used to discard units/warnings.
- Standards source `0fc92a24279f3a4574c80af0a2f31ea347a21597` Planning,
  Implementation, Verification/GUI, Accessibility and Architecture support
  bounded reversible implementation while the native evidence remains blocked.
  The authoring-only skill is not an ordinary application coding requirement;
  no unavailable MCP is represented as passed.
- Focus requirements need real native evidence. Egui pointer/paint tests prove
  only their exact input/layout claims and cannot establish keyboard/native
  perception, scaling, usability or screen-reader conformance by themselves.

## Evidence and limits

This review ran no solver, model, evaluator, corpus or GUI. No current native
display exists in this environment. It does not approve future Inspector
integration, publication, merge, numerical acceptance or branch cleanup. Only
this review report was authored; production and canonical plan were not edited.
