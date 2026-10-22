# Independent PR143 actual merge and source-sanity review

Reviewer `/root/pr143_review`, 2026-10-22. Disposition: PASS for actual normal
merge identity, retained source/ancestry and the named source-only post-merge
checks. Hosted post-merge CI remains a separate pending gate at this review.
No numerical qualification, native/scientific campaign or test was rerun by
this reviewer.

## Actual merge and retained branches

Fresh authorized GitHub-app PR143 metadata reports closed/merged at
`20a0e1e20538b859193c08eaf25a3b8cd26135a5` (P), with reviewed PR head
`159fa74bd271f8fcc07f3837d66e4738925c820c`. The fresh branch-main API and
fetched `origin/main` both equal P. Actual GitHub commit metadata and the fetched
local object agree on exact ordered parents:

`[7befee45270280d07693b91f4afa934ec1780b66,
159fa74bd271f8fcc07f3837d66e4738925c820c]`.

The actual entire merge tree is
`d66c930d091f8e84cf445f0920922d65afed8f8e`, exactly the reviewed and qualified159
tree. This is the expected normal merge after actual ordered PR142 then PR138;
it does not rely on a claim that candidate159 previously descended from N.

Independent app ref reads confirm original
`implementation/q2-static-mms-pilot-20261020` and
`integration/pr143-current-main` remain at159. Supporting
`integration/pr143-final-evidence` remains at
`623ed3cd08757cc4c8295277832f0eac4285522d`. Seven required ancestors are
independently confirmed by `git merge-base --is-ancestor` on actual P: N,159,
M=b70dbae, reviewed142/3cd735f, reviewed138/da1708d, original143/c7f76ec and
prior main15026a9. No branch deletion, force push or unrelated main action was
performed by this review.

## Actual-P source-only sanity

The owner ran one recorded source-only attempt from clean detached
`/workspace/Rheon-pr143-postmerge` at actual P. I inspected its journal and all
four raw logs directly. Journal SHA256 is
`d3917d80c2f3d98bbb84fcbe1939f2c596352b34d84e8c39382641e7536e888c`.
Each command records this exact cwd and exit zero. Every recorded log hash and
the complete four-file artifact roster match.

- The unchanged admission checker reports152 paths at actual P. Its log SHA256
  is `60d73cb42f7dbdd1784da0bf64ed1d71742f20eac0dc2bfa424c3a6a8a3fd7a4`.
- The four pure `CiBudgetContract` cases pass normally and under `-O`, zero skips.
  Both logs have SHA256
  `8b1e10158d32c5c32e8b421ca010cadb069fa0183e175a960d0020ce21215aad`.
  Their byte equality alone does not prove different invocations; the owner
  recorded separate actual argv/start/cwd/exit records for normal and optimized.
- `cargo fmt --check` exits zero and its retained log is empty.

I independently hashed all2,052 current tracked files: the complete map equals
the freshly qualified159 journal's exact map and has compact sorted JSON SHA256
`6aebe9dd485c142965bcfda8575f77245e605c4fe768bdbbefa217704efada7c`.
The actual P tree is the same complete committed source. Thus the independently
reviewed152 closure/v3/admission, private preparation, fixed scientific criteria,
three CV3 ignores and54/16/25 historical seals retain their exact bytes and
previously stated Cargo.toml variance. This does not relabel159 execution as
an execution of P. The detached P worktree remains clean after these checks.

## Remaining boundary

Actual graph/tree/branch preservation and source sanity have no outstanding
finding. This review does not claim the separate actual-P hosted runtime/book
jobs have completed. Their required outcomes must be monitored and reported
under actual P; any later failure keeps its own identity. The original sealed
campaigns, failed/cancelled CI outcomes and prior receipts remain preserved.
PR143 has integrated preparation only: no nine-case scientific, physical/PDE,
moving-boundary/GCL, generic stability, native-window/GPU, performance or
whole-application production acceptance is established.
