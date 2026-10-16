# Independent tail-summary source review

Reviewed 2026-10-16, read-only, against /workspace/rheon-configured-comparison-ui HEAD a10667a5713432e21bcc89fae97864bd1fd8d92b. No Cargo, numerical execution, source modifications, admission updates, or merge performed.

No material findings. Source admission approved for exactly these bytes; execution and final whole-source qualification remain Root-owned.

| Path | SHA-256 |
|---|---|
| src/bin/desktop/comparison.rs | ef52e29f4d9c59007c11ff973ed6a43c33ac9e1d583819ede11ebd7aea9ec0a5 |
| tests/saved_comparison_ui_contract.rs | f3bf401a00ccb9ba67de70ff72c6c3a0b7216525073a6f6c8239ec022e0bec1f |
| src/bin/desktop/configured_worker_tests.rs | 5ba01091e267e2ce51e61626807faf773426ec6243d586007c698bc2388c88ff |

comparison.rs:804-805 preserves None accepted generation as unavailable, treats a verified empty archive with accepted generation zero as having no unrecorded accepted advance, and retains the positive greater-than-durable-generation tail result. Empty archives expose no fabricated field selection.

The native public fixture now constructs an actual empty Failed archive using StepRecorder::finish(..., None), then pins and opens it through the existing owner. This is supported by recording.rs:509-534 and 809-824; the rejected earlier Completed/None or retained-row/None fixture variants are not reviewed final evidence. Its assertions require Ready/count0/recordedNone/acceptedNone/tailNone/unavailable, including an explicit negative against absent. Existing known-positive tail assertions remain.

The constructor-cap regression uses the actual Worker and typed storage terminal, then the actual saved comparison page, requiring Ready/count0/recordedNone/acceptedSome0/tailSomefalse. The final Worker change only qualifies SavedComparisonPage/Availability through crate::saved_comparison; it does not weaken assertions. The separate staged PR116 source and registration were excluded from this three-path delta and retain their own exact review.

No test execution is claimed by this reviewer. Adapter/Root scoped tests and Clippy were pending at review. This report does not transfer earlier passing receipts onto changed bytes or grant whole-plan acceptance.

Execution follow-up: Root subsequently reported all 16 scoped cases (saved UI9, constructor-cap1, storage-tail1, configured comparison5) and strict Clippy passed on the final source pins. This is attributed Root/adapter execution evidence, not this reviewer's rerun or whole-tree qualification.
