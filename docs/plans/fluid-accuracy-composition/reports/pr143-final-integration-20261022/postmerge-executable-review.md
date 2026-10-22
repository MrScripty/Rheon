# Independent actual-P executable and supporting publication audit

Reviewer `/root/pr143_review`, 2026-10-22. Read-only audit; no test, workflow,
native example or scientific campaign was rerun.

Supporting commit `59f37799bc1348a7f33e1ce90546033efe9fc9fb` has sole parent
`623ed3cd08757cc4c8295277832f0eac4285522d` and full tree
`f4cbd0c11086aa3b76d0b16f9fcb333a99464e59`. Local committed objects and the
authorized app's remote evidence-branch ref agree. The three previously reviewed
post-merge payload hashes are unchanged; the fourth independent report review is
the exact authored2,880-byte file at SHA256
`55e12acfa73f8ce11984c5bb05c8303d7fce597178e2279fa9397d2fe29f4747`.
The prior seven supporting payloads remain retained through normal ancestry.
This supporting publication changes no main/candidate source.

The retained own actual-P executable log
`/workspace/pr143-postmerge-executable-20a0e1e.log` is76,981 bytes at SHA256
`6be8f96a9bc1b0c3bcbfbc9bb8f7c6bb854de955bbab143b3614617ebef5d71d`.
It explicitly fetches actualP
`20a0e1e20538b859193c08eaf25a3b8cd26135a5` into origin/main, checks out main,
and prints P from `git log -1 --format=%H`. This push checkout uses a branch;
absence of a detached `HEAD is now at` message does not obscure its actual
printed source identity. No synthetic pre-merge checkout is substituted.

Raw log observations:

- Shared bootstrap contracts pass7 normally and7 optimized.
- The existing executable harness runs220 cases and reports `OK (skipped=27)`:
 193 passes and27 conditional skips, no failures/errors.
- The real CLI command uses size16,12 steps and source-off-at8. Its following
  actual assertions pass for completed manifest,333824 managed simulation bytes,
  all12 step-divergence values and last divergence at most1e-5, PNG signature and
  16-by-16 dimensions. The log prints the actual smoke PASS message after the
  command. Whole-process memory-cap and real-time claims are explicitly false.
- No error marker appears; the job proceeds through evidence upload and cleanup.
  Upload metadata is not a downloaded-content, ELF-identity or visual acceptance
  claim. This reviewer downloaded no artifact and inspected no executing ELF
  for this hosted job.

The owner attributes this completed successful own job119592284582 to actual-P
run39707942188. An optional independent direct app GET of
`/repos/MrScripty/Rheon/actions/jobs/119592284582` was rejected by the fetch
wrapper with HTTP400/INVALID_ARGUMENT: the endpoint was not an allowed public
repository/search URL. That optional metadata lookup was stopped, without retry
or alternate-route retrieval. It is a tool endpoint limit, not a runtime/test
failure. The direct raw-log source and assertions above are the deciding audited
evidence; no additional independently fetched job metadata is claimed.

This audit passes its named source/executable scope. Other actual-P Rust feature
jobs, aggregate completion and the lead's disposition6330773060 for filtered
book/VTK/final-main publication remain separate pending gates. EarlierN/159
results retain their execution subjects and are not transferred to P. No CV3
scientific, hardware/native-window, performance or whole-application production
acceptance follows.
