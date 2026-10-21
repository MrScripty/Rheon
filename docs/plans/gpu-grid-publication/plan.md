# GPU research publication continuation

Status: Active documentation publication preparation.
Current phase: normal exact-head publication CI and artifact-review preparation.
Next slice: push the reviewed publication head normally and create its draft PR
for exact-head CI and actual artifact review.

## Authority, source and ownership

The user's “Get that published” authorizes normal publication preparation and
checks for the GPU research. Production GPU implementation, GPU/hardware
validation, benchmark/speedup claims and physics changes remain excluded.
The parent owns merge order and final lead merge/deployment decision; this worker
must report PR/head/review/CI/PDF/browser evidence first and does not merge.

The preserved research branch is `research/gpu-grid-algorithms-20261021` at
`295e9be5736a92425b4281047576d959bbc1d5b2`. Publication branch
`documentation/gpu-research-publication-20261021`, worktree
`/workspace/rheon-gpu-publication`, starts at main
`d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3`, tree
`8e630e9f870ea3f59ac7962c557089d50bc6cfaf`. A normal merge preserves the research
commit in ancestry. Main includes FEM PR147, portal PR148 and product PR145/146.
No shared-index conflict occurred at this merge preparation. No rebase, force
push, deletion or rewriting of another branch is authorized.

Write set: GPU Chapters 37–39; GPU implementation/source/license notes and
bibliographies; original research plan history as inherited unchanged;
this new publication plan/reports; and one additive GPU paragraph in the book
README. No production, proof, Cargo, workflow, admission manifest or builder
edit is planned. FEM 30–36, portal 40 and thermal-owned paths stay unchanged.
Any later shared README collision is reported to the parent's serial integration
owner rather than overwritten.

## Publication decisions and contract

Puma remains the credited author. Replace nine old pseudocode/diagram blocks
with original explanatory prose and explicit mathematical recurrences, retaining
all original equations and numerical/termination conditions. Publish a public
source/license/access audit; use exact inspected BSD implementation comparison
pins, distinguish them from papers/proprietary API documentation used as factual
provenance, and incorporate no upstream code/dependency/figure or substantial
quoted expression. Future reuse requires its own complete component audit.

The original CPU pin remains `fd7c0f0`; a dated source refresh at publication main
documents new authored-field and force-recording custody without relabeling old
reviews or numerical receipts. Pressure/operator/geometry are unchanged in the
delta. Published GPU capability, device support and speed remain prospective.

Composed-design review: applicable to the candidate independent-backend design,
whose complete ownership/change/dependency/replacement probe remains in the
preserved research plan and Chapter 39. This publication adds no runtime owners
or architecture. Its produced artifact is prose/equation HTML, reading PDF and
portable Markdown/source bundle, all from the existing book pipeline. No generic
registry, renderer, archive or native GPU adapter is introduced.

Concurrent-plan-integration profile: not applicable to these separate single-writer
plans with a designated parent integration owner, rather than competing proposals
to the same mutable plan revision. Shared publication source still requires
current-main reconciliation and explicit conflict reporting before integration.

## Milestones and acceptance claims

| Milestone | Owned output and deciding gate | State |
| --- | --- | --- |
| P0 | Isolate from current main; preserve research/FEM/portal/product history; source/output/standards/build-scope review | Complete |
| P1 | Prose-only GPU chapters, current CPU refresh, source/license/access audit and minimal discovery paragraph; independent source/math review | Complete; no deciding source findings remain |
| P2 | Exact immutable head pushed normally; draft PR; existing matching native/scientific/book/browser CI | Active preparation |
| P3 | Actual generated PDF visual inspection, GPU HTML/Markdown/source links, math, no code blocks, PDF/source freshness and browser evidence | Pending |
| P4 | Parent handoff with exact PR/head/review/CI/artifact evidence and remaining blockers; no worker merge | Pending |

Acceptance is publication-specific: source/math/license scope, original equation
retention, no pseudocode, correct current-source limits, exact-head normal CI,
actual PDF and browser/link/source fidelity. Status: pending. Source parse is
not a substitute for rendered output; green CI is not a GPU numerical result.
All generated plots, fields, logs, site, PDF, screenshots and archives remain
outside Git under repository policy. Remote jobs' own caps are 15/90/35 minutes
for book lessons/full edition/compressible preview; unfiltered dense3D also runs
with no explicit job timeout. Do not weaken or bypass their gates.

The older Library PDF/Markdown and uncertain upload outcome remain untouched.
Repository publication is not a Library retry. If this executor cannot retrieve
artifacts or reach the publication site under effective access, report the exact
target/action/reason/outcome for the parent's existing fresh verifier; do not
retry old denied reads, alter sandbox/security or substitute stale output.

Re-plan for changed pressure/force/recording semantics, an incompatible/unread
source claim, lost original math, output clipping, source/receipt mismatch,
shared-index overlap, failed CI or denied required verification. A failed check
remains failed/pending until the named evidence resolves it.

## Retained resources and rollback

The publication branch/worktree are retained for the draft PR and parent review.
After remote publication they are shared and are not rewritten. Parent owns
normal merge or rejection and eventual clean-resource retirement; no cleanup is
performed here. The original research ref and uncertain Library artifacts are
independently preserved. If published documentation needs correction, parent
uses a normal corrective/revert PR with new artifact identity and unchanged
numerical gates, not mutation of retained outputs or history.

See [ledger](execution-ledger.md), [issues](issues.md),
[CPU refresh](reports/cpu-refresh.md) and
[independent publication review](reports/independent-publication-review.md).
