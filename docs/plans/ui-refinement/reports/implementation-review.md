# Independent first-slice source and evidence review

Reviewed 2026-10-20 by independent `ui_source_review`. Base is
`fd7c0f0aee63bc8710df99f9bb01154e7619b974`; subject is M1's local production
diff and `setup_ui_tests.rs`, against the canonical plan and prior plan review.
This report is not native visual acceptance, numerical qualification or a
standards-engine conformance certificate.

## Final disposition

IR1–IR4 were repaired and independently re-read. M1's bounded source change is
suitable to retain for native validation; no remaining material source issue
was identified in this scope. Objective visual/interaction
acceptance remains blocked by the missing native display, and later milestones
remain unaccepted. This disposition authorizes no publication or integration.

## Findings

| ID | Severity | Initial evidence and consequence | Disposition |
| --- | --- | --- | --- |
| IR1 | Medium | `sidebar` initially derived the displayed Setup label from the active immutable Request, while `controls` selected the disabled form from retained `self.setup_workflow`. Saved reruns could therefore show the opposite form and stale values beneath the actual profile label. | Resolved: shared `Request::setup_workflow` feeds the selector and `running_setup_ui`; active work paints only a frozen-Request summary, not the next-run form. The pure summary test reads physical samples, domain, horizon and maximum dt from an inert Request; no Worker is constructed. Actual running/native coverage remains required. |
| IR2 | Medium | Full Status details initially contained a plain label inside a capped ScrollArea. Pinned egui 0.33.3 has no keyboard scrolling there, and ordinary selectable labels are not tab targets. Keyboard users lacked an in-app route to its clipped tail. | Resolved: immutable `&str`-backed multiline TextEdit preserves full feedback with normal focus/cursor/selection behavior inside the cap. The egui test proves focused Ctrl+End, Select All and exact Copy while attempted editing leaves authoritative status unchanged. Native Tab traversal/focus visibility remain required. |
| IR3 | Low | Shortened configured profile captions initially omitted the old explicit closed-domain word “sealed”; “no inlet” alone does not establish no outlet. | Resolved: both setup and frozen-request summary retain “Sealed full liquid · starts at rest · no inlet.” |
| IR4 | Medium | The unchanged existing `docked_inspector_cannot_overlap_viewport_at_supported_window_sizes` test initially failed on its first egui frame: the nested bottom panel started with a default height shorter than its content and expanded past the dock/window bottom before its cached height settled. Warmed-up new tests did not establish this first-frame property. | Resolved: a normal top-down action/status strip precedes the ScrollArea. Persistence is required; bottom placement is not. The unchanged existing first-frame dock test now passes at both supported sizes, as do all five new tests. No guessed height, empty reservation, extra sizing/input pass or new UI state is retained. |

## Source observations

- Action-strip painting returns intent; App dispatches only after current form edits.
  Configured freeze still checks the current inspection/path/provenance and
  support before saving or launching. No stale action-strip result becomes authority
  for package validity.
- Accepted Session shared controls select carrier sampling, pressure method and
  tolerance independently of retained preview `material_mode`. Switching setup
  changes presentation state only; it does not assign numerical settings or
  launch a worker.
- Existing start/start_configured, immutable intent, Request, worker, polling,
  cancellation, recording and source pin bodies are unchanged. Unit-bearing
  inputs and the configured profile's limitations remain present.
- Primary action, progress, Cancel and status are outside the scrolling setup
  content and remain in the Inspector tab composition. Long status has bounded
  geometry and retains the original string; no parsing invents severity or
  completion.
- Production writes are limited to the two admitted UI composition/form files;
  focused tests are separate. Inspector, D1, numerical sources, root README,
  local agent work and the separately validated Inspector branch are untouched.

## Evidence reviewed and limits

The final `/tmp/rheon-ui-focused-tests.log` showed five `ui_setup_` tests pass:
pointer setup selection/value preservation, carrier controls for Accepted
Session, bounded status/top-strip composition and Inspector composition plus focused
full-text navigation/copy, disabled uninspected adapter action, and inert frozen
Request summary. These are actual egui input/paint assertions, not screenshots or
a native process. The test source never dispatches solver/model launch intent.
The affected strict Clippy log reports success; formatting log has no findings.
`git diff --check` passed during this review. After the IR4 repair, the unchanged
existing dock, section-collapse and archive-transition tests each passed in
`/tmp/rheon-ui-existing-{dock,sidebar,archive-transition}.log`. Their real egui
assertions cover first-frame dock bounds, section state and stale Inspector
suppression respectively. They do not launch numerical workers. The lead owns
the final static-gate rerun and consolidated verification record.

The final test-only strengthening was independently read: wheel input targets
the actual setup-form title region, the title must scroll out of view, and the
Simulate rectangle must remain fixed. This distinguishes real form scrolling
from a no-op wheel event. Test naming now reflects the top strip. Production
source hashes are unchanged; no further test execution was needed for this read.

The summary fixture proves its projection, not a complete active-worker/rerun
workflow. Directly requesting text focus in an egui test proves focused editing
semantics, not real Tab traversal or native focus visibility. Native perception,
scaling, ongoing-worker Cancel, usability and 40% fewer visible words remain
unproved. No GUI, solver, model, evaluator, corpus or scientific experiment was
run by this reviewer. Only this report was authored.

Final reviewed source SHA256 after the IR4 placement repair:

- `src/bin/rheon-desktop.rs`: `3004e9bdf1b53c5dd079bca14dae9aaedcd85047f3d833a2934045fc06466882`
- `src/bin/desktop/configured_controls.rs`: `f4581090a79d9d9eed6c5fc3e6e53d0cd7b6b6bf26aa924a18aa4a038feb8ae4`
- `src/bin/desktop/setup_ui_tests.rs`: `ba81bde816aea562b40d11a2290b3b7099ef8c3b184010ec74b61f1717314dfc`
