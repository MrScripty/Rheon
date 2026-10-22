# Independent M5 publication-failure source review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`. Reviewed the complete current diff of `src/bin/desktop/run_sets.rs` and new `run_sets_publish_tests.rs` against the existing publisher and original verification criterion A5 durable failure/recovery. Only this report was written. No build, test, publisher invocation, native workload or numerical work was executed by the reviewer.

**Complete findings disposition: no required source correction identified.** The narrow testability extraction preserves the production publication contract and supplies appropriate fault-boundary fixtures. This is source/fixture disposition only; executed passing checks and final whole-feature A5 reconciliation remain parent-owned and pending at this review boundary.

## Production behavior

`publish_new` forwards its original arguments to the private implementation; only cfg(test) builds add the optional failure argument. The failure enum, optional parameter, branches and child test registration are all cfg(test). Ordinary callers pass None. There is no global runtime failpoint, environment switch, allocation framework, dependency or schema change.

The non-test operation sequence is preserved: reject invalid cap/oversized bytes; resolve/open parent directory; reserve unique same-directory temporary name; `create_new`; `write_all`; file `sync_all`; close file; hard-link to the requested target without replacement; first directory sync; unlink temporary; final directory sync. All guard ordering, existing path ownership assumptions, hard-link/no-replace semantics and error mappings remain unchanged. The wrapper introduces no publication or recovery action. A failed link is still an ordinary Io result; once linked, sync/unlink failures remain PublicationUncertain.

## Fault boundaries and fixture adequacy

The six injected boundaries are placed correctly:

| Injected stage | Operations already completed | Expected authoritative result |
| --- | --- | --- |
| BeforeStage | guards/parent/name selection | Io; no new target or staged file |
| BeforeFlush | actual complete write, before file sync | Io; target absent, staged bytes are not authoritative |
| BeforeLink | file write/sync/close | Io; target absent, staged file is not a published revision |
| FirstDirectorySync | successful hard link | PublicationUncertain; complete target can be inspected |
| RemoveTemporary | first directory sync | PublicationUncertain; complete target can be inspected |
| FinalDirectorySync | temporary unlink | PublicationUncertain; complete target can be inspected |

The flush hook runs after the real `write_all` and returns through the existing staged-error/close path. Post-link hooks return the same uncertainty category as the corresponding real filesystem operations. This does not simulate partial writes, a completed sync later losing data, process death or power loss; those stronger claims are explicitly excluded.

Each stage has its own target/directory and a separately confirmed all-Air4³ old document. A real normal publication provides exact valid all-Water4³ bytes for the new target. All six cases verify old bytes unchanged and strict old-document open/restore. Pre-link cases require absent target; BeforeStage additionally checks no extra staged entry. Post-link cases require exact new bytes and strict typed new-document restore/counts. They perform read-only reconciliation only: no uncertain publication is retried, overwritten, repinned or repaired. Abandoned temporary files are left nonauthoritative, consistent with the original owner. The test does not mistake them for a complete requested revision.

The normal fixture covers zero/too-small byte cap and missing parent refusals, complete strict-open success and existing-target AlreadyExists without replacement. Its subsequent opposite-material publication attempt verifies that the old confirmed bytes stay intact. No incidental cleanup/recovery changes production semantics. These checks target meaningful filesystem/owner invariants rather than merely mirroring the wrapper's return value.

## Acceptance limits and next boundary

After actually executing these fixtures and affected checks, this evidence can close the original missing **manifest publication fault-boundary** portion of A5. It is not by itself all of A5: SQLite accepted-prefix/append uncertainty, immutable frozen bytes, fresh process and actual native save/open/rerun use their existing separately reviewed evidence. The parent must preserve those traceability links during M5 reconciliation. No repeat native campaign or extra numerical cohort is implied.

The unchanged contract remains caller-owned local filesystem paths with supported hard links/directory sync and protection against concurrent replacement. Injection demonstrates branch behavior on that environment, not crash-durability proof, hostile-filesystem race immunity or unsupported filesystem qualification. This review does not authorize ordinary exposure before the separate numerical/objective gates and final source/artifact review.

Final formatted SHA-256 read in this round:

| Owner | SHA-256 |
| --- | --- |
| `src/bin/desktop/run_sets.rs` | `da46d5476760973cb431dc91be4450b844807f862f059b5a72cdc2af96fea6ce` |
| `src/bin/desktop/run_sets_publish_tests.rs` | `a62766b85c7dbd8a91719a239010398158b65e2f0d4cf35df779df4b9b080374` |

No reviewer action was denied, pending or uncertain. Prior denied external operations and uncertain Library writes were untouched. Actual test outcomes and final M5 acceptance remain pending task gates, not claims made by this source review.

## Targeted fresh-process draft fixture review

Reviewed the focused-admission addition and the two new functions in `voxel_material_persistence_tests.rs`: `fresh_process_reopens_complete_and_incomplete_drafts_without_numerical_work` and its explicitly ignored child. **No required correction identified.** The parent writes real4³ complete/incomplete draft envelopes to distinct original paths, records their original bytes, launches the current test executable as a separate process with the exact ignored child name and canonical original paths, and requires both successful status and the child-specific completion marker. The marker prevents an empty exact-filter selection from being mistaken for a pass. After child exit, the parent checks both originals byte-for-byte unchanged and strict-open equality.

The child restores exact IDs (index21 Water or Vacuum, remaining cells Air), identity/revision, counts, original geometry bits, v1 role/color palette and independently literal density/viscosity bits. It checks empty fresh undo/redo/history, no active gesture, and complete validity versus typed one-cell Vacuum at `[1,1,1]` with actionable fill diagnostic. Recapture equality checks restored draft semantics. No App, Worker, executor, numerical constructor or step is called in either new function; draft restoration is an authoring allocation only. The older frozen-input stepping child is separate and is not invoked by this exact filter.

This is suitable evidence design for the original fresh-process **complete and incomplete draft** criterion, conditional on actual execution; it is not additional simulation/replay evidence or all of A5. At this static review boundary the parent reports compilation in progress, so no pass is asserted here. Only this report section was written and no workload was run by the reviewer.

Final reviewed `src/bin/desktop/voxel_material_persistence_tests.rs` SHA-256: `a866e2281c68acb3ef1cdfe3eb8f1b065c5ed59e8e1ffe3946b75a8b84c1c787`.
