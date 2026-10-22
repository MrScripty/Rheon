# PR59 final-native driver V3 static follow-up and hash correction

2026-10-22. **Static review only. No driver or selected child command executed.** AST parsing and in-memory syntax compilation passed without import/execution. No approved binding, final invocation, output directory, test/build/native/packet/timing run, binary edit, admission/ref change or external operation was created by this reviewer. The parent froze V3 during this final binding check.

**Conclusion: no blocking static defect found in frozen V3 for its intended fixed seven-command route. The main V1/V2 issues are addressed. This is preparation-only approval of the driver logic, not execution authorization or final source/admission/protocol acceptance. Actual merged PR143 O and independently approved R1/binding/invocation remain mandatory and absent. PR143 being reported READY does not satisfy actual merge.**

## Explicit correction to the first report's version/hash attribution

The initial report `/tmp/pr59-final-native-driver-review.md` remains unchanged, SHA256 `cb7080b00b5ef28c88adeb5769262406de619a033a7d429f5ca213c09011b4f1`. Its findings described the initial V1 text, but its final source-hash read overlapped the primary worker's V2 edit and recorded the **V2 hash**. That attribution was incorrect. It must not be used as evidence that V1 findings were a complete assessment of V2. The version history is corrected explicitly here without rewriting the first report:

| Version | Preserved/current file | Correct SHA256 |
|---|---|---|
| V1, original reviewed text | final_native_driver-v1.py | `76151dc973a7537ab5a95dfd0e42935d80f6855c04f75c48986325e8195f246f` |
| V2, first repair | final_native_driver-v2.py | `dd708bf8082ab8502213938525e719cda5d8589c8710ad2d22aee9f1894dd68c` |
| V3, frozen current reviewed bytes | final_native_driver.py | `62585bdb57f14a796cad0a1b2b830aa0d22650e448b5ea1e6fc28067532d3449` |

Read and compared the preserved versions separately. Final V3 bytes and all version hashes were rechecked after writing this report to prevent another overlapping-edit binding. No version file was changed by this reviewer.

## Repairs verified in V3

1. **Positive exact approval binding.** Required `--approved-binding` is read once as bytes, hashed from that buffer, parsed from the same buffer and archived unchanged in the fresh output. It must match full candidate/base heads, candidate/base trees, actual143-verified true and independent-final-source-review-passed true. Exact clean HEAD and ancestry are also required. This replaces the denylist as the primary positive prerequisite. The binding is still a trusted independently reviewed factual record; booleans must not be fabricated for synthetic159/088 or earlier N. No such record is supplied or approved here.
2. **Failure/interruption custody.** Each attempted command's intent is journaled before launch. Nonzero child return is recorded and stops without retry. Exceptions/timeouts/interruptions set ended time, explicit interrupted-or-uncertain status, exception and partial-log SHA where available, preserving returncode null rather than inventing success. Outer status becomes failed-or-interrupted and exits nonzero. Unknown descendant-process outcomes are not treated as completion.
3. **Atomic journal saves.** Writes use a temporary, flush/fsync and atomic replacement, preserving the prior valid journal across ordinary interrupted saves. This is not an unconditional hardware/directory-crash durability claim.
4. **Native/packet/exporter binding.** Exporter checks require native passed/clean/exact head/tree, exact18 command names in order and zero exits, plus the live exporter hash. Existing packet receipt must be passed/clean, bind current and native head/tree and exporter, and match the exact native receipt SHA. Checks run after native qualification, after fresh packet capture and after retention; an identity mismatch fails rather than triggering a rebuild/retry.
5. **Bounded raw custody.** Retains exporter and exactly four unique native Rust test ELFs from actual rust-contracts output. Prefix checks map once each to aligned_strain_contract, obstacle_flow_contract, static_obstacle_contract and the aligned_strain example. Copies use exclusive destinations, require ELF/non-symlink original, and compare original/copy hashes before/after; no objcopy/strip/patch occurs. All five successful custody records enter the journal. This is post-run custody, not proof of the exact historical loaded bytes without contemporaneous hashes.
6. **Final unchanged-source verification.** Exact HEAD/status/tracked set and all captured tracked-file hashes are checked before custody and again after all copies, before successful final status. Exporter/receipt identity is rechecked afterward. Existing all-source/receipt checks are not replaced by a partial source list.
7. **Environment/output controls.** All seven removed provenance/packet/native environment names are recorded, without spoofing GITHUB_SHA or source/build-material identity. Each packet test item records its sole fresh-packet environment addition. Fresh output is rejected inside any Git worktree before creation, aligning with the qualifier guard.

## Command/cwd and numerical-scope check

Seven outer commands remain exactly admission; CiBudget normal/optimized; unchanged native qualifier; fresh real packet; packet tests normal/optimized. Both CiBudget calls target `test_column_interface_verifier.CiBudgetContract` from tools cwd. As verified in the inherited published source, it is a separate four-method source-only TestCase with no ColumnVerifierTest fixture setup; imports have guarded fixture entrypoints. No numerical ColumnVerifier fixture is selected.

The qualifier is called without --lean/--allow-dirty and retains its exact18 ordinary commands. It includes the existing fixed native/rational checks; saying the driver is ordinary does not mean those checks contain no numerical work. No ignored/sealed/performance runner is selected. The packet pattern has26 reviewed tests and receives the fresh packet environment for both modes. The driver does not parse Python test counts itself; final result review must independently confirm26 normal/optimized completions with no unexpected skips, plus the exact four native target receipts, rather than equating any exit0 with arbitrary coverage.

Supporting child-source hashes and scope remain those recorded in the first report, verified against reviewed159 during that review. The final actual R1 may include authorized CI/admission changes and must be independently rebound/reviewed before execution; this review does not preapprove those unknown bytes. In particular, recheck CiBudget remains source-only after warm-CI integration.

## Final prerequisites and residual limits

Before use, independently verify actual143 O (including its ordered ancestry), the exact clean committed/admitted R1 head/tree/source map, unchanged qualification/oracle/gate contracts, the positive approval record and the exact invocation/output/environment/toolchain context. Retain this V3 driver SHA with the approved invocation; the journal does not self-authenticate the Python file or record every inherited build flag/toolchain setting. Those belong in the final reviewed execution receipt. Preserve final partial logs/copies on any failure; do not silently continue, relabel or rerun.

No existing792/692/etc preliminary receipt is relabeled final. First outer exit1 uncertainty and all original logs/versions remain preserved. Historical +2.20% adverse particle evidence, strict positive/inconclusive control block, permitted qualification scope, warm CI, final actual-main source admission and lead normal merge remain separate gates. No denied action or uncertain mutation arose. The only remaining execution blockers are the absent actual-O/final-R1/admission/binding/invocation prerequisites and their independent final review, not a static defect found in this frozen driver.
