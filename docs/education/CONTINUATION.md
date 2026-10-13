# Book continuation milestone

Goal: continue primary-source research, teachable interactive 3D examples and
scoped Lean contracts without changing parallel workers' solver implementations.

Base: PR43 `66cc5678910cc5a9203f7f0b60f85e5bc3e9e905`;
branch: `research/book-pr43-continuation`. Main inspected at `2209012`.
No AGENTS.md or repository .agents/skills exists at those refs. No persistent-goal
API was exposed in this session. This file preserves the bounded plan.

1. Preserve PR43 and inspect current native contracts: complete.
2. Chapter 9 swept-volume geometry, exact-real ledger/bounds, primary research:
   implemented; mathematical qualification is separate in native-sequence.json.
3. Independent equations, all-chapter HTML build, desktop/mobile controls,
   screenshot inspection, pinned Lean and axiom audit: passed. The longer
   native/full-book pipeline remains a separate required check before merge.

Changes are confined to book education, proof sources and their validation.
No native solver edits, PR42 images, merge or Pages deployment is authorized.
The draft PR is stacked on PR43 so its diff contains this milestone only.

## Verification boundaries

- All 20,402 discrete control states compare with an independent Fraction oracle.
- Local Lean bootstrap verified the official elan archive checksum, then the
  pinned toolchain fetch failed: CONNECT tunnel failed, response 403. No bypass.
- Local sandboxed Chromium aborts because the existing SUID sandbox helper is
  not configured correctly. No sandbox disabling or host-security changes.
- Hosted workflows retain sandboxed Chromium, full-book checks, proof source
  policy, pinned lake build and transitive axiom audit. Fresh results determine
  qualification; prior PR43 receipts are historical only.
- The earlier PR43 artifact binary 403 is not treated as current visual evidence.
- Further solver, scheduler, CAD and AI changes require exact implementation
  handoffs. PR45/47/48 coordination below preserves their unmerged scope.

## Qualified milestone

At head `29522d4`, [Lean CI](https://github.com/MrScripty/Rheon/actions/runs/38380599816)
passed pinned build, the 252-declaration audit and rejection tests. The
[proof receipt](../research-book/evidence/volume-flux-proof-qualification.json)
binds exact proof bytes; no proof source changed afterward.

The focused `volume-lesson` job in [book CI](https://github.com/MrScripty/Rheon/actions/runs/38380599905)
passed sandboxed desktop/390px/320px interactions. New screenshots and their
source-bound receipt were retrieved through the supported GitHub connector,
then desktop and 320px captures were visually inspected. This does not resolve
or claim a review of the older PR43 artifact.

The local complete HTML build contains 31 chapter/appendix sections and 645
rendered math expressions. Its 4,956 local link/asset references resolve except
for the unrendered PDF. Local actual-browser rejection probes could not run
without a qualified PDF/browser edition; hosted full-book checks remain required.
No screenshot establishes physical correctness or numerical convergence.

## Implementation coordination update

PR47 `1923d5b`, PR48 `f06a127` and PR45 `bbc6089` were read at exact heads,
including implementation and contract tests. Chapters 11/17/22 and COVERAGE.md
now distinguish stage-labelled diagnostics, transport-only sessions and the
stationary Stokes experiment. Ten PR45 exact controls passed locally in both
Python modes. No draft implementation was merged and no theorem inventory or
Cargo.lock binding changed. The three drafts are not described as a combined API.
