# PR59 independent artifact-provenance correction

2026-10-21. Separate correction; the prior report remains unmodified.

## Correction and implications

The primary worker disclosed that earlier GNU objcopy `--dump-section ... INPUT` commands omitted a separate output-object argument and rewrote INPUT. Accordingly, the executables inspected in `/tmp/pr59-execution-image-review.md` must be labeled **objcopy-rewritten diagnostic artifacts**, not untouched Cargo/compiler outputs. This review did not execute that mutation or independently reproduce it. The disclosure changes the provenance and inference scope of the inspected files, regardless of whether any particular machine-code section happened to survive rewriting unchanged.

The prior independent report, SHA256 `0574bb832ad42861fde5537f0f63f2c50c2dd03f1d71ba2d7be9ed2a7264f8ea`, accurately binds and compares its explicitly recorded artifact hashes. Its byte comparisons, section hashes and 21-byte load-payload difference apply to those diagnostic bytes. They do **not** establish the layout, header differences, section exceptions or execution-image identity of unmodified compiler outputs. Any reading of its references to raw artifacts, supplied build lineage, or retained original outputs as proof of untouched Cargo provenance is withdrawn by this correction. No raw-build identity conclusion may be carried forward from that report.

The two previously inspected diagnostic artifacts were:

| Artifact | SHA256 bound by the prior inspection |
|---|---|
| `/workspace/pr59-base-remapped-target/release/examples/cpu_particle_lab` | `9650df93b8c3b360c22e28b91a69a71b42a8a914b7c1565b5ef47e079f46a0db` |
| `/workspace/pr59-candidate-remapped-target/release/examples/cpu_particle_lab` | `3a6fcdeb162d5a5e9f2e902851a532d7f0c16d64b2538b3861b0555c43755c36` |

Preserve these rewritten artifacts, their prior reports and their receipts under explicit diagnostic labels. Apply the same provenance caution to any earlier artifact subjected to the command or whose transformation history is uncertain; do not assume an earlier copy is untouched without a custody record. This correction does not infer anything about unrelated historical campaigns.

The primary worker states **no timing ran**. Thus this incident has not contaminated a measured fixed cohort or changed statistical outcomes. It occurred during pre-timing diagnostics. The original source review and its exact committed-source hashes are unaffected by rewriting executable files. The original historical +2.20% particle observation remains preserved and unresolved. The strict timing stopping rule remains unamended; no whole-file or broader image-based acceptance is established.

## Regeneration and reassessment boundary

Rebuilding the unchanged source with the identical frozen flags is a reasonable recovery. Triggering recompilation by changing only harness mtimes changes no source bytes, but verify the exact source hashes before and after and retain the actual Cargo command, environment, log, artifact path, size and whole-file SHA256 immediately after Cargo succeeds. Do not strip, patch, canonicalize, change build-ID/linker/alignment flags or tune output to recover an expected comparison. Freshly named copies can preserve those exact compiler-produced bytes with a clear custody record.

The new raw outputs require a **fresh independent complete ELF assessment**. Recompute whole-file hashes, program headers, all allocated section metadata/payloads, NOBITS layouts and every differing byte in load payloads. Do not presume the prior build-ID/header exceptions recur. Use read-only `readelf`, `objdump`, direct binary reads and hashes; do not use a dumping command capable of rewriting INPUT. The new assessment must explicitly distinguish raw compiler outputs from preserved diagnostic rewrites and bind the final reviewed source/build recipe. The initial-main diagnostic comparison still cannot replace final ordered-composition qualification.

No earlier report was altered and no rebuild, test, benchmark, binary execution or repository mutation was performed by this correction review. Fresh compiler artifacts and their exact provenance are pending from the primary worker; their outcome is currently unknown. No operation was denied.
