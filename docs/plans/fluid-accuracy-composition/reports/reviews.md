# Independent review and dispositions

All initial findings below were collected before material revisions. The three reviewers were disjoint from research/visual authors and worked read-only against the proposal and actual main. These are independent design reviews, not independent native execution or experimental validation. Primary author owns revisions and qualified kernel runs. Reviewed pre-revision plan SHA256 `931c6bf9f43acb3d248d622148cd495865d03ec2ff2111ec17b9cb041799a653`.

| Reviewer / finding | Severity | Required correction | Disposition |
|---|---|---|---|
| math_review M1 | Medium | Separate normal flux/velocity and divergence units | Addressed by design revision; implementation gate remains open |
| math_review M2 | Medium | Define weakly compressible EOS energy and native defect/work ledger | Addressed by design revision; implementation gate remains open |
| math_review M3 | Medium | Normalize zero-traction residual by nonzero declared stress; separate physical and algebraic measures | Addressed by design revision; implementation gate remains open |
| architecture_review A1 | Medium | Assign failed/cancelled attempts optional-publication terminal custody and cold-open tests | Addressed by design revision; implementation gate remains open |
| architecture_review A2 | Medium | Require infallible local adoption/disposable provider preparation; define commit/abort failure and uncertain-state recovery | Addressed by design revision; implementation gate remains open |
| architecture_review A3 | Low | Prototype must include existing RCP3/RCT1 observations/terminal custody | Addressed by design revision; implementation gate remains open |
| standards_review S1 | Medium | Separate completed P0 delivery lifecycle from unadmitted production; canonical next-slice admission | Addressed by design revision; implementation gate remains open |
| standards_review S2 | Medium | Stable qualified acceptance claim register, evidence/environment/mode/owner/status | Addressed by design revision; implementation gate remains open |
| standards_review S3 | Medium | Exact bounded G1 existing/new write set and replan boundary | Addressed by design revision; implementation gate remains open |

Math review found no high-severity equation/sign/unit formulation error beyond the acceptance-unit issue; exponential equilibrium, capillary sign and coupled defect/work identity were coherent. It did not run an independent Lean kernel. Architecture review confirmed D1 reuse and native-family boundaries; standards review confirmed the complete eight-part architecture probe and HTML/native distinction. Qualification/reviews were still being authored and excluded from initial standards review. Rechecks and final dispositions follow below.

## Rechecks after revision

- math_review confirmed M2/M3; M1's final flux wording is now an absolute face leakage flux bound of `A_face × (1e-8 m/s)`, dimensionally m³/s. No new mathematical blocker; no second kernel run claimed.
- architecture_review confirmed A1/A2 addressed at planning level. Added accepted-versus-attempted generation distinction and pre-adoption fault placement. A3 text correction is included; it does not add graph custody to current code.
- standards_review confirmed S3, then requested precise Deferred→Planned→Active admission and pending/blocked/satisfied claim statuses. Those corrections separate lifecycle from acceptance without a ritual path-recitation requirement.
- All production resolution claims remain pending. Reviews qualify this proposal, not native implementations or experiments.

standards_review final recheck: S1–S3 resolved, no material standards blocker. Rechecked pre-final plan SHA256 `3cbeccf50939f71544740162d985fb63f190ec9a19eabd997de8f5d14a3db472`; verification `8f61deaee458c491eef69c091524aaeab032f17b31796d6c4c207db5d40ddcb9`. Subsequent changes only finalize actual qualification/status and correct Markdown flux notation.
