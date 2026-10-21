# Independent M1 fixture and gap-handoff review

Reviewed 2026-10-20 against production checkpoint
`426fd25d22975e1259f5543c77dc771a6ac646ff`. Scope: the new test-only synthetic
active-state case and `m1-native-and-gap-handoff.md`. Only this report was
authored; no numerical, model, native or scientific workload was executed by
the reviewer.

## Fixture disposition

No workload path or remaining material test-source issue identified. The test
constructs one waiting owned thread, manually initialized progress/cancel atoms,
an inert Request and empty bounded LiveHistory. `with_budget` reserves empty
frame slots, `begin_step` only records timing, and `CpuScheduler::new` initializes
accounting state. The path paints App's real sidebar and dispatches the actual
Cancel operation with `child=None`. It never calls `Worker::start`, `run_owned`,
physics advancement, model startup, frame publication, archive open or recording.

The five-second channel timeout bounds the synthetic thread's lifetime even
if an assertion prevents explicit release. On success the test releases and
joins it. The thread intentionally ignores the cancel flag; asserting it stays
alive after Cancel prevents confusing signal routing with worker retirement.
The synthetic outcome is consumed by the test, not polled into a fabricated App
completion status. No production flag, state machine or numerical default changes.

At both supported logical sizes, the test checks manually seeded 1/4 progress,
frozen request values despite conflicting next-run preferences, screen-contained
Cancel/progress text and Cancel's stable position through Inspector switching.
It then proves real flag routing and unchanged progress/preferences. These are
egui input/projection claims, not measured physical progress, real saved-rerun
execution, cancellation latency/cleanup, native scaling or focus evidence.

The final lead log records nine `ui_setup_` tests passing and final static gates
passing. Reviewed test SHA256:
`1df47b37d2a830617ef2c6a2389a512f57c490a400b745b5927379501419129a`.
No production file differs from the prerequisite in this fixture slice.

## Handoff evidence boundary

The handoff attributes native idle success and 42 retained JPEGs to
the parent's validator relay, binds production checkpoint/tree and explicitly
states that this task did not view/re-hash those artifacts. Forbidden delivery
is not treated as a confirmed Library save. The uncertain capture completion is
not retried or mislabeled as capture failure. Idle native results do not close
active-state, long-status, focus/accessibility, user timing or word-count claims.
Quality-85 export was requested; missing capture metadata is explicitly not
represented as independent confirmation of the image quality setting.

The possible native UI-only harness remains a proposal: test-only entry point,
clear fixture label, synthetic values, no production CLI flag and no launch in
this turn. The separate genuine-worker proposal explicitly calls out numerical
execution/new recording, existing owner limits versus RSS, fresh paths, no model,
no budget increases, no retry and separate execution/resource authority. A frozen
saved-rerun needs its own authorized immutable fixture and bounds.

The requested timeout clarification is resolved and independently re-read:
the proposed 10-second gate-arrival and 10-second retirement waits bound
observations, not the full process. The handoff now explains the potentially
blocking destructor join, cancellation's limits, the need for separately
authorized supervision/teardown when a hard process limit is required, and the
absence of process-kill authority. This does not affect the current five-second
synthetic handle. No remaining material source or handoff-boundary finding in
this scoped fixture checkpoint; whole M1 acceptance remains incomplete.
