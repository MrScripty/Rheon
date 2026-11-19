# PR59 preliminary test-ELF custody review

2026-10-22. Read-only local custody inspection. No binary execution/mutation, test/build/rerun, timing, cloud/GitHub call, log/artifact retrieval or repository mutation was performed. Existing local files were read; only this report was written.

**Custody check passed for the supplied retained set: 91 ELF files, 198390984 bytes.** Manifest SHA256 verified: `9dfc42337e66ab12afc9af334c0d42b072a5e177997d3620ec8a89b74f4913f8`.

## Exact coverage and byte checks

Parsed every Cargo `Running ... (PATH)` line in the retained confirmation log, SHA256 `8a8dee9a657de9ac327c85aedf03e385cc06d46197385f1b3b8318975f02bafa`. There are exactly **91 unique paths**. Manifest entries match all91 original paths in order, with no omission, duplicate or added entry. Retained filenames are unique basenames; each resolves to a regular nonsymlink file inside `/workspace/pr59-preliminary-test-elfs`. Directory membership is exactly manifest.json plus those91 retained files, with no unlisted artifact.

For every retained file, read complete bytes and independently verified manifest byte size, SHA256 and ELF magic. All91 are ELF64 little-endian x86-64. Aggregate size is exactly **198390984 bytes**. Manifest remains unchanged after inspection. No objcopy, strip, patch, chmod or executable invocation was used.

Current original files: **91 present and byte-identical to their retained copies**, **0 missing**, **0 mismatched**. Equality was checked by full bytes in addition to size/hash for every present original. This verifies custody against files present now; no mismatch was concealed or repaired.

The confirmation contains92 top-level result groups because its additional doc-test target reports **3 compile-fail documentation tests**, with no executed test ELF listed for those cases. Those are correctly excluded from the91 test-ELF set. The set covers the logged Cargo test targets; it is not a claim to enumerate every compiler, helper, CLI subprocess, dynamic library or external executable that tests might invoke.

## Provenance and interpretation

Manifest explicitly identifies preliminary resolved `6975421 +159fa74` working code, full tree `7514ff90d5efc823ad5d5dbf7c9dab687184f838`, reviewed connector commit `c33f02906532a77de456a99738e7e134ea1fbf1c` and source-map canonical SHA256 `8055a53c319ddec6aade4bed35fdc642c7ab07839e4e0bb2866a00a4a03d3688`. The locally fetched connector object independently has that tree, and the current153 selected source bytes still match the reviewed source map. This supports the stated source-equivalence boundary. It does **not** assert tests executed on the later connector-created commit.

The manifest and prior matched confirmation journal record child return0, with792 passed /0 failed /13 inherited ignores across92 top-level groups. The manifest explicitly preserves the first invocation's outer-exit1 uncertainty, says no tests were rerun to seal these files, and says no performance cohort ran. These are retained receipt/custody statements; this reviewer did not execute the tests or independently observe historical process execution. Current post-run binary copies plus the log's path names do not cryptographically prove the same bytes were loaded at the earlier instant of execution, absent a contemporaneous executable digest receipt. No new executable-at-run-time guarantee is inferred from this sealing step.

Raw files are retained outside Git for later archival by the existing task owner. This review does not create/upload an archive or transfer qualification to another source/commit. Earlier objcopy-rewritten diagnostic particle artifacts are a separate set and remain separately labeled; none was substituted into this test-ELF manifest by the path/hash checks above.

## Remaining scope

No branch/ref publication was performed or remotely checked here; the normal support-branch fast-forward to c33 was supplied as completed context. Original branches and distinct local/connector identities remain governed by their prior records. Final actual ordered #142/#138/#143 source admission, allowed native/packet qualification, warm CI, fixed prospective controls and lead authorization remain pending. Historical **+2.20%** adverse particle evidence and strict positive/inconclusive stopping rules remain unchanged. No preliminary custody success is final scientific/performance/main admission.

All requested local custody checks completed. No denied action, uncertain write or blocker arose. Manifest binding supplies the exact per-file original path, retained filename, hash and size for all91 verified files without duplicating that immutable table here.
