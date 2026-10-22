# PR59 prospective final-native driver review

2026-10-22. **Static protocol review only** of `/workspace/pr59-publication-tools/final_native_driver.py`, SHA256 `dd708bf8082ab8502213938525e719cda5d8589c8710ad2d22aee9f1894dd68c`. AST parsing/in-memory syntax compilation passed without importing or executing the driver. No output directory, journal, build, test, numerical run, timing, pins, Git refs, external task/artifact or GitHub action was created/run/changed by this review.

**Disposition: command scope is correct, but improve the custody/journaling details below before execution and obtain the still-pending final exact-source/admission/protocol approval. This is not permission to run now.** Actual PR143 merged-main O and the final clean reviewed/admitted R1 remain absent from the proposed invocation. Verified actual PR138 N supplied by the parent is not a substitute for O. All prior792/692/etc source receipts remain preliminary and are not relabeled by this driver review.

## Fixed command sequence and fixture scope

The seven outer commands are: admission; source-only CiBudgetContract normal/optimized; unchanged native qualifier; fresh real packet; packet tests normal/optimized. No `--ignored`, `--allow-dirty`, `--lean`, sealed-campaign/performance runner, archive publisher, inventory writer or Git ref mutation is selected.

Both CiBudget commands use cwd `/workspace/Rheon/tools` and the explicit unittest target `test_column_interface_verifier.CiBudgetContract`. In the inherited reviewed PR143 source, that class derives directly from unittest.TestCase, has four source-only methods and no setUpClass. `ColumnVerifierTest` is a separate class whose setUpClass constructs `fixture("column_interface")`; targeting CiBudgetContract does not load that class into the suite or invoke its fixture. Module imports of generated_fixtures/verify_column_interface have guarded entrypoints and do not call fixture during import. The selected commands therefore do not inadvertently select numerical ColumnVerifier fixtures. This must be rechecked against final R1, particularly if CI guards change for warm coverage.

The unchanged native qualifier's no-lean path is exactly18 inner commands: six setup/contracts/build/lint/gate commands, five fixed native/oracle pairs, and two oracle-test modes. Real packet capture uses its fresh native qualification receipt and exporter, with no historical record injection. Packet unittest discovery has26 methods in the reviewed source; the driver sets RHEON_ALIGNED_PACKET to the newly produced packet for both modes, enabling the real-packet class. Target path selection matches the existing qualifier's CARGO_TARGET_DIR/default debug-example convention. These are scoped ordinary native/packet checks explicitly intended for a later authorized run, not a claim that no numerical work exists inside qualification.

The supporting source files inspected are byte-identical to published reviewed159; they describe the intended inherited command behavior, not an already admitted/final checkout at the hard-coded `/workspace/Rheon` target. Final R1 must supply and bind its actual hashes.

## Findings to resolve before execution

1. **Medium — interrupted command record is incomplete (driver lines69–85).** A nonzero child return is properly recorded and stops the sequence without retry. But TimeoutExpired, KeyboardInterrupt, spawn/open failure or other exceptions before line74 leave the active item `status: running`, `returncode: null`, without ended time or log hash. The outer journal becomes failed-or-interrupted, so it does not falsely pass, but individual command custody remains incomplete. Add per-command exception/finalization: preserve the actual exception/status, end time and hash of any retained partial log, retain unknown return code as unknown, and stop without continuing/retrying. Do not infer that all descendants completed after a timeout/interruption unless observed.

2. **Medium — journal rewrites are non-atomic (line61).** write_text truncates the only journal on every save. An abrupt interruption during save can corrupt the prior valid record, precisely when interruption evidence matters. Use a sibling temporary plus atomic replacement, with flush/fsync if crash durability is claimed; retain a valid running/failed record. This should be a bounded journal-integrity change only, not an execution retry policy.

3. **Medium — final exporter custody is not reconciled/retained (line80).** The existing real-packet builder checks exporter hash against native qualification and rechecks after capture. The outer driver later records the exporter hash without comparing it to native/packet receipts; a changed/missing executable after those stages is not explicitly reconciled, and no raw executable copy is retained outside the shared target. Before success, require the actual exporter SHA256 to equal the qualified native and packet receipt identities, then retain a raw byte-identical copy with exact path/hash/size and verify copy identity. Keep source/receipt equality checks; do not rebuild, strip, objcopy, patch or rerun to repair a mismatch.

4. **Execution prerequisite — actual-base approval is external, not enforced by the blacklist (lines26–33).** Full lowercase SHA, clean exact HEAD and ancestor checks are useful. Excluding three old bases does not prove the supplied base is the actual PR143 merge; an unlisted ancestor, reviewed synthetic159 or synthetic088 can otherwise satisfy ancestry. After O lands, freeze/check the exact independently approved O/R1 identities (or bind an independently sealed approval record with those identities). Do not treat the blacklist or `preliminary: false` journal field as evidence that this prerequisite was met. This is currently a preparation-only driver; no execution should occur until final review explicitly binds the invocation and new source/admission/protocol state.

## Additional bounded improvements

- Record all names actually unset: the five provenance/GitHub variables are removed and **none is spoofed to a candidate SHA**, which is correct; RHEON_ALIGNED_PACKET and RHEON_ALIGNED_NATIVE_DUMP are also removed but absent from `unset_environment_names`. Record the intentional packet-only override per command, plus nonsecret target/toolchain/build-flag settings or a bound environment receipt, to make the effective build context reviewable.
- Validate final native/packet receipt status, clean/source head/tree, expected18 inner-command inventory and exporter bindings, and retain normal/optimized packet26 counts with no unexpected skips. Current exact source commands support those expectations, but exit0 alone is a weaker summary than the protocol promises. Do not silently accept missing coverage or extend/retry failed work.
- Align the output preflight with the qualifier's rejection of **any Git-owned destination**, before creating journal/output. Current driver excludes its own repo only; a fresh path in another worktree can be written before native qualifier later refuses. Keep outputs fresh and external with no prior-file overwrite.
- The all-tracked byte snapshot and final HEAD/status/path/hash comparisons correctly reject changed source at the end. For complete custody, record file type/mode where relevant and the driver hash itself, and finalize the last source check after executable custody operations. This does not replace the final independently approved conservative selected map and committed admission binding.

## Correct retained safeguards

The driver uses fresh exclusive log files and an external fresh top-level output, saves command intent before launch, records actual nonzero returns, stops on the first failure, catches BaseException for outer failure/interruption, verifies HEAD/tracked set/status and all tracked-file hashes on normal completion, and does not import historical receipts. It does not refresh pins or write Git refs. Child environment removes GITHUB_SHA/REF/HEAD_REF and RHEON_SOURCE_REVISION/build-material overrides rather than fabricating approved provenance. The packet override is newly generated evidence only. These safeguards should remain unchanged while addressing the findings.

## Exact supporting source hashes

- `tools/test_column_interface_verifier.py`: `46a9d538ef4680e983099a3dfdcb6eb98b7ca87a5f8c8ec80e02d64508142960`.
- `tools/generated_fixtures.py`: `09ba81c54209e5896b9d840cd42192716e5008e0a1004533359b3c34cf4ef791`.
- `tools/verify_column_interface.py`: `b4e822ec7977c9a3145b4a299532e09453a1c400163360e35a09306a6860e13c`.
- `tools/qualify_aligned_strain.py`: `5b907d7d65f688b41b5c56a2c2f3f2f4cc4a4b7958a4bb3af1a7df5ac448e5ba`.
- `docs/education/aligned_strain_packet.py`: `361ec0530ad64bbb0f698463a0553ab1c3b4964af3e9cbce547a327456cb8b3a`.
- `docs/education/test_aligned_strain_packet.py`: `d53b4a44943782288868bfd2bcab56ca2f24220990a62a749e63a9148550a5f9`.
- `docs/education/check_native_admission.py`: `fa315c21355a5eca90f0692f01b4d3a2c1c936cbd6a8e0f8e192d06beac847b2`.

No final native, real-packet, ignored/sealed or performance execution occurred here. Final actual ordered main, explicit source/admission review, authorized warm CI and exact-head checks, strict prospective performance controls and lead normal-merge authorization remain separate. Historical +2.20% adverse evidence and all prior failures/uncertainty remain visible. No denied action or uncertain mutation arose; the blockers are driver improvements and pending final prerequisites, not an attempted run.
