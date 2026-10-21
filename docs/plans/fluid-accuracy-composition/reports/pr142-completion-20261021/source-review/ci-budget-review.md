# Independent outer CI budget patch review

Reviewed read-only against clean source `dbd49d3e41c482a753884abc8aaaff7d1b0d0653`
before the patch was applied. External patch
`/workspace/pr142-rust-ci-budget.patch` SHA256 is
`a6ce599c12f0645fd8fb91b1f3abc18cd5c6b0d78a96a299e3b0edbc3cc97e4f`.
The exact old workflow SHA256 is
`769e89aadb3ed2abbcf2e72a8a722ab0de5165976de0f8f35eca0ad37e63ea76`;
the prospective new workflow SHA256 is
`80f732505778a8f7746ca42c52e919c5d7d2cbdf0fe2432c8695079663c8e300`.

The patch changes only `.github/workflows/rust-rheon.yml`: the outer
`feature-contracts` matrix job timeout increases from 30 to 60 minutes and two
comments explain its scope. Commands, feature matrix, assertions, per-case
limits, scientific bounds, executable job and aggregate gates are unchanged.
Independent in-memory diff comparison and `git apply --check` passed. The
workflow is outside the unchanged conservative 152-path native selector.

The reported historical desktop cancellation at the previous outer job limit
supports allowing more scheduling/compilation/test time without weakening a
test's acceptance criterion. This review did not independently re-fetch that
historical run and does not guarantee that 60 minutes suffices. The newly
audited exact-tree hosted run 39688034222 separately completed all three feature
modes under its original 30-minute setting; that success remains its own
source-bound evidence.

Approve the exact prospective patch within the user's instruction to complete
and normally merge the remaining work. Application was conditioned on the local
clean-source qualification driver and owned children first ceasing. Its native
source closure and prior admission bytes must remain unchanged; original
receipts remain bound to `dbd49d3`, never relabelled as receipts from a later
workflow/documentation commit. Actual final diff, source identity and required
checks on the resulting new head remain separate verification steps.
