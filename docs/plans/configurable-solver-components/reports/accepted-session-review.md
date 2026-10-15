# Independent accepted Session core source review

Reviewed on 2026-10-15 by the independent design_review agent (GPT-6.1 Sol).
Workspace: `/workspace/rheon-accepted-session`, uncommitted seven-file slice over
`739c1aef0ddd85c9200f65b9f8155d170737b8b3`.

## Finding and recommendation

No remaining material source or fixture findings in the exact files below.
Recommend proceeding with exact-tree validation and source-admission review for
this bounded accepted Session core slice. This record is an independent source
review, not implementation acceptance, a numerical qualification receipt, model
parity, cloud CI evidence, or complete-plan acceptance.

The previously identified unbounded identity receipt issue is resolved:
`ContextAdapter::new` retains only validated identity while `has_provider` keeps
the actual preferred provider on the existing explicit Unsupported/fallback path.
Invalid identity reaches neither provider support/prediction callbacks nor the
published binding. `prediction_called` explicitly means host method invocation,
without claiming external runtime consumption.

The existing viscosity stage updates candidate velocity before the authoritative
pressure RHS is assembled. Existing boundary/surface refusal and full-liquid
viscosity restrictions remain. The context borrows the persistent accepted
pressure array; no history array or new numerical kernel is introduced. Receipt
assembly precedes volume qualification and final commit. Session fields, receipt,
accepted history, and quota update only after complete successful publication;
failed preparation keeps the prior publication. Changed viscosity and cleared
boundary history cause explicit native fallback before a new composition prefix
can support prediction. Native-only selection does not fabricate a model attempt.

## Fixture assessment

The eight public tests use a clearly declared native-solver candidate, not a
learned model. The physical fixture uses density 2, dynamic viscosity 1/128,
spacing 1/4, a 5-cubed grid and spatially uniform binary acceleration. Pressure
settings are tight (relative 1e-14, absolute 1e-16), and maximum Courant is 1/128;
production bounds are not weakened. Uniform closed-box forcing is a legitimate
pressure-driving composition fixture; it does not claim general flow accuracy.

The tests compare complete native-only combined state bits with existing viscous
transport over multiple intervals. The actual-dt test independently constructs
forced face values, applies the public viscosity stage and builds the authoritative
RHS, proving that the observed viscous RHS differs from its inviscid counterpart.
They check accepted prior bits and hashes, support refusal and bad-candidate
fallback, composition reseeding, every observed cancellation checkpoint, provider
cancellation, post-pressure volume failure, retry quota, stop, exact workspace
caps/mismatches, invalid coefficients/domain, and invalid identity rejection.

## Verification and limits

Reviewer performed source inspection, SHA256 pinning and `git diff --check`
(passed). Reviewer ran no Cargo command, model, numerical experiment, or test.
Execution results reported by other owners are deliberately not independent
receipts in this record. Only this external review document was written.

The additional deterministic unit
`configured_deadline_crossed_at_final_callback_preserves_history_and_quota`
closes the previously noted test-design gap. After one accepted viscous interval
it reaches the actual configured final callback, advances an injected clock to
the deadline inside that callback, and checks Deadline, exact dense3D publication
retention, the previous receipt timing sentinel and every history-stamp field.
A successful retry consumes the remaining quota and the next call is refused.
This review approves the regression's source design; execution remains owned by
the lead and is not claimed as an independent reviewer run.

The API document and three changed plan paragraphs were also reviewed. The API
correctly separates pressure-stage fallback from whole-interval failure at later
velocity/volume gates, distinguishes viscous full-liquid restrictions from the
existing inviscid represented-volume domain, and states identity/IPC/persistence
limits. The plan describes A2 as being qualified, keeps complete acceptance open,
and does not make the absent measured A1 bundle block all A2-A4 implementation.
The concrete configurable runtime bridge, selective package physical-support
checks, measured learned-bundle parity, accepted report persistence, and complete
A2/A3/A4 acceptance are outside this core slice and are not established here.
In particular these fixtures do not establish the rho=1000/mu=0.001 trained-model
contract or actual external IPC execution.

The separately reviewed CI-only commit
`ad3176d0219ae7d88907fc59d8c163d8a08b053d` correctly extends path filters, scoped
Clippy and existing real-checkpoint test invocation to accepted consumers without
changing export or qualification gates. This is source approval only.

## Exact reviewed SHA256

| File | SHA256 |
| --- | --- |
| `src/simulation.rs` | `8dea790daf2a2d0801db3003094f9fffe034c9b87946d520a1eacc266c95f078` |
| `src/liquid_step.rs` | `f99dce6a99f6af8a9aef2436af7f33de58dac3be5998a48e9187fbdf2acc3f93` |
| `src/session.rs` | `604e579bf4a2c780c4941889230654432a2e491059013a94f20300b5ecbe665d` |
| `src/session/configured.rs` | `f8a8b7687de99884ad2b7790e9fb3da9c9674d0bcc1a19a21b1e6f7751ac4187` |
| `src/solver_provider.rs` | `0ab7db08a9016e73d664c730e4ff513d42599a84c680ea061e6604859432a5a5` |
| `src/solver_provider/configured.rs` | `124ca65dd1dec3a8a89043198fb085453a42ca8aff0dc87c67b04b6a986109be` |
| `tests/configured_provider_session_contract.rs` | `ba4a8f26fbf0324d705a80a4b108b1e9d09a293ac2eb29a8278fe01f65dc5a00` |
| `docs/api/configured-pressure-session.md` | `9e939b980ea85abbd5d5dc5835e808d7be33fd9ed6771104e7d01810a9bf3dd2` |
| `docs/plans/configurable-solver-components/plan.md` (three changed paragraphs only) | `442b22a068fa703c6d82a4675a303b75dab0105b81c28d89e0d5fc3c3e0716e8` |
