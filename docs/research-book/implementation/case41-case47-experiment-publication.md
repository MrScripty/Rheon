# Separate bounded case41 and case47 experiments: publication identities

Branch: `research/case41-fd-case47-pure-e2`. Tested combined evidence result: `c20aa84e2de27a6537d4685c102c640d8f87dba4`; tree `a45d25d85d634fd18a14f95f14e3fe2ff9f798af`. These histories are forward commits; no frozen source/evidence, production code, threshold or Lean claim is rewritten.

## Case41: stopped before the seven equations

Experiment source `f2d2183346fcab7f6a825b831e3f15c194ed4c96`, tree `da4ec51815425d758dbb24578f776c2c8d1ca962`. Failed preflight result `e0b5343c865c9389d4b59ed7bfd60c272d53a4c2`, tree `961270caa6545a25adf64c82bdfb6dbe6eb40bcb`. The three scalar/layout/affine tests passed. The memory reader failed to handle a jointly absent standalone equation-frame category, consistent with inlining. Per instruction, no corrected preflight, candidate equation, Newton correction or owner advance ran. [Result and remaining blocker](../../../evidence/case41-six-fd-columns-v1/RESULTS.md), [linked-route diagnosis](../../../evidence/case41-six-fd-columns-v1/DIAGNOSIS.md).

Compiled preparation input remains source `c238111d8c8d159c157e489294a25998aaf36516`, tree `c87e9c658cad27df708a34aa63bccf9868095598`; ELF SHA-256 `48f58e62577eef569e942d3e8e7139701613091076a736c015bee5256691ba72`. Blocked receipt SHA-256 `9c086b1a1564880991a84984129ac5a42c21ad5d7763e388ac420f7203eee59a`.

## Case47: original pure-E2 128 steps and eleven cancellations passed

Compiled v2 source `e37b0bb34ace62f16dc931488146d6a396b27be0`, tree `5e3fef4bdb87087acbeb7bdf56303fa9669b2b2a`. Passed preflight/execution source `529707429fb092beb4df9a278d978bf72e8612cf`, tree `5badc6cb9d3a792d37a5d2a391e0d66126f0c49a`. ELF SHA-256 `6e6ea0505c81cbf8624b177c6c782ff8cb29cc0830c0524e79088e8307a10989`. Result receipt SHA-256 `18341d6835f9af2dd8bedad04b15a8fd2848401744617453473618d80dfdec6a`.

Exactly one original main trajectory reached 128 steps, followed by eleven serial cancellation-preservation jobs with matching 25-step pure-E2 prefixes. No finite step retried. Independent physical replay and eight corruption controls passed; additional memory bound 67,088/67,584 bytes. [Detailed results and limits](../../../evidence/case47-pure-e2-128-v2/RESULTS.md), [inspected actual trajectory render](../../../evidence/case47-pure-e2-128-v2/trajectory.svg). The wrong-family v1 constructor assertion and both read-only serialization failures remain preserved with forward repairs.

## Preservation and remaining scope

[Normal/optimized post-Git binding](../../../evidence/case47-pure-e2-128-v2/post-git-normal.json) is byte-identical, SHA-256 `73d4995a5d62cb72d8d67cd382d29baf6b9bff0dc4fcf5583b1b5067aa19a4ab`. It verifies all 8,386 tested-result tree files, all three experiment receipts, every one of the 8,134 inherited bytes/files, the same ELF, author/committer identities and seventeen clean frozen worktrees. Its inventory SHA-256 is `638f60fe630e0c7804445f4962d70252f51e6a58569ecd0738e7bfebd3091f28`.

Case41's full scaled Jacobian remains unavailable. Case47 qualifies only this original-family trajectory and first-visit cancellation preservation; retry behavior, endpoint reference error/temporal order, the full refinement roster, all eight original geometry-band failures, arbitrary meshes/materials, smooth-root/IEEE proofs and production adoption remain separate. Both historical runtime refusals stay intact. Parent owns reviews, PRs and merges; no such actions are requested by these packets.
