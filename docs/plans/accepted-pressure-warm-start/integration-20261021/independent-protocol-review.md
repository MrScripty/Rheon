# PR59 supplementary independent protocol review

Date: 2026-10-21. Read-only analysis; no build, test, benchmark, repository mutation, or cloud operation performed. The only written deliverable is this report.

## Recommendation

**Yes: explicitly normalize source-root strings before the final fixed timing cohort.** The proposed identical literal RUSTFLAGS in both independent builds are a reasonable control of a build-location confounder:

```text
--remap-path-prefix=/workspace/pr59-base=/rheon --remap-path-prefix=/workspace/Rheon=/rheon
```

Record and freeze the exact flag string, toolchain, target/profile/features, source commits, lockfile, build commands and environment before timing. Keep the initial unremapped binaries, hashes and diagnosis; do not replace their record. Do not change code, alignment, linker settings, flags or sampling after timing outcomes to obtain a preferable comparison. A supported final baseline worktree can reuse `/workspace/pr59-base` after verification of the final predecessor main, with a separate target root and preserved provenance for the earlier baseline. Neither source path prefix overlaps the other, so this particular pair does not introduce ambiguous replacement precedence.

**Whole-executable byte identity, if obtained, is strong construction evidence for the exact particle harness and build configuration.** Independently built byte-identical executable files contain the same code, data, headers and relative layout. Under the same loader/dependencies, inputs and execution environment, there is no candidate-specific generated executable change to cause a runtime regression. Equal bytes do not imply identical timing observations: scheduling, ASLR, physical placement/cache state, process startup and host load can differ. They also do not prove every default consumer, target, toolchain, feature combination, or unremapped build is identical. The supported claim must name the exact executable/configuration and artifact hashes.

I support an explicit **pre-timing identity branch** in the protocol for this exact artifact. It supplies a different and stronger artifact-level fact than noisy statistical equivalence. It must not be silently substituted for the existing stopping rule, and it must not be advertised as a general default-path performance guarantee. The protocol owner and serial integration lead should record agreement before timing; final admission still follows the specified dependency order, native source admission, exact-source review and required CI. If they do not adopt that amendment, the existing rule remains controlling even if the binaries later prove identical. If exact identity is not obtained, retain the current positive/inconclusive blocker with no reduced thresholds or extra sampling.

## Independently inspected evidence

The current protocol (SHA256 `7c509870878006887cc91a5808fa553f9a7632d114c578202e9ab4808d60a3b0`) says that default performance still needs an actual controlled comparison and that a positive median or inconclusive comparison cannot claim default non-regression. Its fixed cohort and historical evidence retention remain appropriate. This report recommends a clarification/amendment and does not assert it has been approved or applied.

Parsed `/tmp/pr59-initial-codegen.json` (SHA256 `27e52e99a5e5e0c885bc8636781f1dfdb4c9c23afd77450c787da9ac6a7d8643`): all 736 listed changed extents have equal base/candidate sizes, every listed address delta is -16 bytes, and the added/removed symbol lists are empty. This verifies the report's contents; symbol extents alone cannot prove equal instructions or complete executable equivalence.

Read the actual binary files without executing them:

| Material | Bytes | SHA256 |
|---|---:|---|
| `/workspace/pr59-base-target/release/examples/cpu_particle_lab` | 925992 | `9dcf19de99ec0d9754c1625e83b9557ec9292a31e7629e1308431fa51fdb5a84` |
| `/workspace/pr59-candidate-target/release/examples/cpu_particle_lab` | 925944 | `9da8c2797be2f112f8c8ff144e0098cb118140fab71eeb2b05ea8888a5f4de09` |

The base contains `/workspace/pr59-base/src/parallel.rs` and no `/workspace/Rheon/src/parallel.rs`; the candidate has the reverse. Their total lengths differ by 48 bytes. This supports a source-location confounder hypothesis, but does not establish that these strings explain every byte/layout difference, nor any historical timing difference. The current binaries are not identical. No remapped executable has been supplied or validated in this review. The primary worker reports that no new timing has run; I did not inspect timing logs to establish that absence.

## Suggested advance protocol text

The following wording would make the branch explicit while preserving the statistical gate when it is needed:

> Both final source roots shall be compiled using the same literal RUSTFLAGS: `--remap-path-prefix=/workspace/pr59-base=/rheon --remap-path-prefix=/workspace/Rheon=/rheon`. This controls embedded source locations before timing and is frozen for the cohort. Preserve the initial unremapped artifacts and diagnosis. Independently build final pre-PR59 main and the final candidate into separate target roots, record exact commands/source/binary hashes, then compare complete executable bytes without stripping, patching, canonicalizing or otherwise rewriting the outputs. Symbol sizes, disassembly similarity, equal `.text` alone or matching selected sections do not satisfy identity.
>
> If complete executable bytes match exactly, record construction identity only for that named default-plus-parallel particle harness, toolchain, target and build recipe. Still run and retain the predeclared 12-pair cohort per case, every stdout/stderr/outcome and exact accepted horizon/fingerprint/managed-byte check. Report median, range, adverse count and paired uncertainty without using those observations to claim statistical equivalence or speedup. Positive or inconclusive timing does not negate the independently established artifact identity; investigate harness/environment discrepancies without extending the cohort or changing the frozen build. This identity branch requires recorded owner/lead acceptance before timing and does not retroactively resolve the historical +2.20% control.
>
> If the final executables differ, the original stopping rule applies: a positive median or inconclusive comparison cannot claim default non-regression; default harm remains a blocker requiring a supported repair or explicit owner tradeoff. Do not relax numerical gates, invent an equivalence margin, tune layout after results, or extend the cohort.

A final identity receipt must bind the final ordered composition, rather than initial main `15026a9`. Use both full-file SHA256 and an ordinary complete byte comparison; retain each build's original artifact and build log. Loader/dependency and runtime command/environment records should accompany the receipt. The identity claim must be refreshed if source, features, toolchain, flags, target or linked dependencies change.

## Remaining limits

No final performance or merge admission is given here. Initial unremapped codegen evidence cannot establish path strings as the complete cause; a controlled rebuild and full comparison are needed. The original 1000-particle +1.83% screening / **+2.20% confirmation** remains visible, adverse and bound to its original binaries. Feature-enabled mixed particle applications remain outside the historical warm-start performance admission. No denied, pending or uncertain external action arose during this review.
