# Native admission failure and integration contract

Book lane, 2026-10-13. Coordinate with runtime integration thread
`01a1374a-bbd5-71b6-991b-4e0112e10032` before changing main. No main merge
is performed by this diagnostic change.

## Observed failure

[PR59 book run 38440499705](https://github.com/MrScripty/Rheon/actions/runs/38440499705),
job 115445519656, fails while constructing the educational aligned-strain packet:
`aligned_strain_oracle.Refusal: changed qualified native source Cargo.lock`.
The decoded supported job-log tool confirmed the exception. A separate CLI
archive-log download returned HTTP403; that denial was not bypassed.

Main `2209012` still contains the historical admission rule pinned to native
source `f75cd66`, with one narrowly permitted dense3D export extension. That
rule compares all selected native/build inputs, including Cargo.lock. It cannot
admit later material/API/provider sources simply because those sources pass
their own tests. A new numerical receipt does not override an older source pin.
The refusal protects the meaning of the packet; it is not evidence that the
runtime numerical oracle disagreed, nor that the runtime PR is ready to merge.

The workflow design currently discovers this integration mismatch only after
expensive compilation/qualification. The new read-only preflight calls the exact
same source gate immediately after checkout and provides a specific remediation.
It writes no hashes, changes no acceptance condition, and does not replace the
later independent qualification or publication checks.

## Existing reviewed book extension

PR43 introduced `aligned-strain-native-sources.json`, an explicit complete
admission inventory over the integrated material/CPU source. Its base ancestry
is main `2209012`, but its bytes describe the reviewed book source, not every
possible descendant. Fresh receipts must bind both its digest and the actual
committed source blobs. A present stale inventory fails without fallback.

The book-to-main native differences at PR43 are three files: desktop diagnostic
labels distinguish accepted speed from the earlier pressure used for forces;
`scheduling.rs` adds policy/failure/nonoverlap tests; `sgs.rs` adds joined-wave
predecessor tests. They must be preserved when integrating runtime changes.
No source admission from PR43 automatically authorizes later API, CAD, provider,
collision or performance changes.

PR60 full run [38458741142](https://github.com/MrScripty/Rheon/actions/runs/38458741142)
and PR61 full run [38464301184](https://github.com/MrScripty/Rheon/actions/runs/38464301184)
passed on their own committed sources, including fresh native/rational and
browser/publication checks. Those passes cannot be transferred to another tree.

## Coordinated repair, still required

1. Agree the exact runtime integration commit and approved PR set. Hold PR59's
   unresolved performance acceptance. Do not merge it through the book chain.
2. Merge preserved book work into an isolated integration branch; review every
   changed/added/removed native/build/qualifier input relative to the admitted
   book source, including dependencies and the three book-owned changes above.
3. Preserve the old admission record as historical evidence. Explicitly create
   the reviewed current admission record only after source review. Its scope is
   permission to attempt fresh qualification, never evidence that it passed.
4. Commit the integration and admission review together. Run locked native tests,
   the independent rational qualifier, adversarial packet tests, source/receipt
   binding checks, Lean audit, browser controls, book/PDF/links and publication
   probes on that clean committed source. Any failure blocks publication.
5. Coordinate main merge order, then let main's workflow rebuild and publish the
   exact integrated tree. Verify the deployed source receipt and actual live
   pages. Do not use the separately pinned PR36 deployment workflow for new work.

This keeps a deliberately conservative all-source inventory. Narrowing the
inventory to an assumed transitive numerical subset would need its own dependency
analysis and tests; no such weakening is part of this repair. A book-only
source build may remain available for review, but missing qualified packets must
stay unavailable and must not be presented as a successful publishable edition.
