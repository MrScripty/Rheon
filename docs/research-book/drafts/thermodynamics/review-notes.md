# Independent review, corrections and documentary verification

Date: **21 October 2026**. Scope: the new local manuscript in this directory, based on main `b9218038d138d1c73818cf4158757521853a5cd7`. Outcome: **reviewable research drafts with no remaining identified substantive manuscript defect**. This is not publication acceptance, scientific validation, legal clearance or production admission.

## Independent technical pass

A separate read-only reviewer examined Chapters41–47 and independently rederived moving-interface mass/volume, pressure compatibility, capillary/recoil and Marangoni signs, planar energy/Stefan signs, binary Stefan flow, Clapeyron and its approximations, Boussinesq/low-Mach dilation, thermal graph identities and the analytic references. Selected Rheon source and Scapin/Abels primary preprints were checked. The reviewer made no edits or commits and ran no solver tests.

| Finding | Severity | Disposition |
|---|---|---|
| Phase-field energy combined dimensional free-energy density with an unscaled gradient term while epsilon represented a physical length. | medium | Chapter46 now introduces constant energy-density scale \(f_0\), dimensionless potential \(W\), dimensional mobility and matching fourth-order timestep scale. Independent correction re-review passed. |
| Wording implied mass exchange alone automatically transports latent energy. | medium | Chapter46 now requires an evolved conservative total-energy variable and state inversion containing phase offsets; mass conservation alone is explicitly insufficient. Independent re-review passed. |
| Momentum ledger listed actuator work. | low | Chapter47 uses actuator force/impulse in momentum and work in energy. Independent re-review passed. |
| Entropy graph calculation logged dimensional temperature. | low | Chapter42 uses \(\log(T_i/T_{ref})\) with fixed positive reference. Independent re-review passed. |

The independent reviewer found no high-severity defect. Jump signs are tied to liquid-to-vapor normal and vapor-minus-liquid jump. The planar full-energy identity remains distinct from a curved interface with surface energy. The equal-density Stefan surrogate remains explicitly solid–liquid; the spherical diffusion-drop reference retains its quasi-steady, large-density-ratio and thermal assumptions. Conditional proofs are not promoted to full discretization theorems.

## Independent source/license pass

A second reviewer session could not initialize: the collaboration tool returned a thread-store metadata-recovery deadline error. No source-review action executed in that failed session. The existing independent reviewer then performed a separate read-only source/license pass; this is a second pass by the same reviewer, not two different reviewers or legal counsel.

That pass independently matched all **38** repository-audit byte counts, SHA-256 values and Git blobs against the pinned base. It verified Rheon's Apache-2.0 `LICENSE`/Cargo declaration, the pinned CoolProp MIT authority and both inspected file blobs, all five IAPWS cover attribution permissions, the named IAPWS section references, Dodd2025 publication identity/rights notice, N9 institutional metadata and the excluded CC BY-NC-ND study's terms.

It found one low-severity date precision issue: IF97's inspected cover identifies August2007 with revision through2012, rather than a separately established October2012 release date. The bibliography now says **“August2007 release with revisions through2012.”** The reviewer re-read that exact paragraph and closed the finding. Both passes ended with no remaining identified substantive defect.

The reviewer found the citation/reuse distinction and exclusion boundary consistent. This does not establish permission for future source copying, a full dependency graph, imported property data or an adapted paper figure. Three reviewer DOI-resolver opens were inaccessible (Cleary–Monaghan, Guo–Lin and Young–Goldstein–Block); no bypass/retry occurred. Their limited metadata/abstract status and unread bodies remain explicit. The retained primary publisher/institutional records, not a successful DOI resolution claim, support their identities.

## Actual documentary checks

The check outputs and temporary rendering tools are outside Git at `/tmp/rheon-thermal-doccheck/`; they are disposable observations, not scientific evidence or repository inputs. The existing book/Pages pipeline was not run because these drafts are intentionally outside publication and that pipeline invokes other scientific/native prerequisites.

| Claim/check | Observed result | Scope |
|---|---|---|
| Markdown parse with installed Pandoc and the book's `markdown+tex_math_single_backslash` input mode | All authored Markdown parsed | document syntax; no physical accuracy claim |
| KaTeX0.16.22 with `throwOnError:true`, `strict:'error'`, HTML+MathML output | 365 mathematical expressions passed before this review-note addition; final check recorded below | equation syntax only |
| Local relative Markdown links and audit JSON parse | Checked; final result recorded below | local target existence/JSON syntax; not all remote availability |
| Repository source identities | All38 matched, including independent reviewer check | pinned source provenance, not numerical qualification |
| Scope and whitespace | Final staged checks recorded below | only new documents in this directory; protected files unchanged |

Final local results: **11 Markdown documents and2 JSON audits parsed;21 local relative links resolved;368 mathematical expressions passed strict KaTeX0.16.22;38 source pins matched with zero mismatches; no code fences were present.** Pandoc HTML fragments were emitted only to the disposable check directory. No full browser layout/PDF qualification is claimed. The staged-scope check permits only additions under this new draft directory and whitespace checks passed. Protected chapters/builders/indexes/README/production files have no contributed change.

KaTeX was installed using the repository's unchanged `docs/education/package.json` and lockfile, `npm ci --ignore-scripts --no-audit --no-fund`, with task-owned prefix/cache under `/tmp`. No package manifest, dependency version, browser sandbox, workflow or publication tool was changed. No full Pages build, browser/PDF qualification or deployment is implied by successful math parsing.

## Remaining limits and handoff

No production solver, thermal campaign, Lean proof, hardware measurement or performance benchmark ran. No article's accuracy/performance was reclassified as Rheon's. Source access failures and selected-body inspection are retained in the bibliography/audit; unsupported detailed formulas/results from unread bodies are withheld. The proposed validation ladder still needs implementation and exact reference extraction before its acceptance criteria can be selected.

The branch/worktree is retained for parent review under [the review guide's resource contract](REVIEW.md). No push, PR creation, publication, merge, deployment or modification of portal/FEM/GPU work was attempted. Eventual book integration remains a separate authorized step.
