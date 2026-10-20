# Independent mathematics and source review

Review date: 20 October 2026. Reviewer: separate read-only agent
`/root/independent_math_review`, explicitly requested by the task. This was an
independent source/mathematics review, not an external human peer review.

Disposition: **accept substantive corrections, Chapter 29 and Appendix G for
documentation review**. No outstanding substantive mathematical error was found.

The reviewer independently checked projection signs and SI units, weighted
adjoints, integrated residual/divergence scaling, mixed-material particle
pressure as the derivative of the chosen compression energy, fluid/body traction
and torque signs, moving cut-volume balance, sealed-component compatibility and
the frozen constrained work identity. It compared main source/history and the
specified active PR records rather than treating proposed experiments as results.

The review identified and the author corrected:

- Stale PR45/47 capability status and the claim that bounded obstacle pressure
  did not consume the retained static geometry.
- Unspecified solid/fluid normal in the pressure load formula.
- Missing distinction between geometric volume change and closed incompressible
  compatibility.
- BVH traversal incorrectly described as the owner of convex containment;
  actual containment uses a separate halfspace test.
- Overly broad light-body stability wording and a presupposition that an
  unretained earlier particle implementation necessarily existed.
- Chorin read-depth reporting and CGAL links. The final bibliography uses the
  actual accessed `/latest/` manuals, observed as release 6.2.1, with date stated.

The first-pass accepted substantive source hashes are:

| Source | SHA256 |
| --- | --- |
| `docs/research-book/chapters/29-liquid-solid-interaction.md` | `5aa90a83974894fa62ad510449be1351df137f1d8523923762b7358c5249922a` |
| `docs/research-book/appendices/G-paper-audit-and-corrections.md` | `81941f8b6b61197ef9f5c98fccb1eedd12e1ee517b675f309ae92d0a115ee59c` |

These historical hashes match book commit `4a78007571932ffbf46ab2ef3e34cb5bd08526dd`.
The final CGAL link substitution changes retrieval provenance, not mathematics.

Citation limits are explicit: Chorin's scanned full text was not freshly read;
its metadata and independently derived projection are distinguished. The density
maps and thin-shell comparisons use verified author abstracts, with no detailed
equation attributed to unread full text. Primary S7/S11/S13 and the CCD,
embedded-boundary and semi-analytical-wall sources support their stated uses.
Published performance/accuracy is never adopted as a Rheon measurement.

The reviewer changed no file, ran no solver, mutated no evidence and performed
no publication. This review is not native execution, PDE convergence, physical
validation, a proof of all book claims or a fresh Lean kernel qualification.

## Supplemental lead reconciliation

The same independent reviewer accepted the supplemental diff in book commit
`750352e3f56abc156d81799d32f94096e8884689`. It independently checked
`dU=-sum(p_i dV_i)`, the fixed-wall constant-pressure partition requirement,
the distinction between the calibrated-offset gamma diagnostic and Mayrhofer's
method, and the full-mass staggered-Q1/P0/absolute-traction-pressure limitations.
It read Larionov's primary supplement and confirmed that the disk's nonzero
interface pressure balances viscous normal stress under zero total traction;
the annulus and Rheon's prospective polygon are different boundary problems.
It accepted Chapter 25's historical PR45 status and corrected fragment link.
No substantive correction remains requested.

Final accepted SHA256 identities:

| Source | SHA256 |
| --- | --- |
| Chapter 29 | `85bab89355e7bb8ac66f05f6610578fb00ab68dcf0dbf2747da31f5e5f3895c2` |
| Appendix G | `18eb0ddcc69c32a463355f72c2fbcc20b3c9e43e4e49181eb1dc1347e7bf0c4c` |
| Bibliography C | `a178b20ec92dd49aa427e6af7d7cc39ba4556c3a42e2add8b8d690e7531931db` |
| Chapter 25 | `0e53828029a721166193b27f45ce8bbe572c8ab407360d1571a3aa0ef48a3778` |

The review was read-only and adds no numerical execution or publication claim.
