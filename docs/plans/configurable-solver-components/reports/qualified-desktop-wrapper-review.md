# Qualified desktop wrapper independent review

Read-only review of the three completed wrapper paths in `/workspace/rheon-qualified-desktop-build`. The five Rust stamp files remain covered separately by `/tmp/rheon-desktop-build-stamp-independent-review.md` (SHA-256 `5c2de326a19d42b8e7ce477fd9597b7a84d67b2016ecfaa2d46b2d63a6afef8a`). No Cargo, real compiler, desktop executable, model, GUI or numerical qualification was invoked in this review.

## Assessment and exact bytes

No material finding. Source approval is appropriate for this bounded tracked-source custody wrapper and its contracts. It is not actual native-build evidence or full compiler-read closure evidence. The lead must commit the intended clean source and run the wrapper with real trusted tools to produce fresh artifact receipts.

| Path | SHA-256 |
| --- | --- |
| `tools/build_qualified_desktop.py` | `392381ab8810ac08bd801b7a3e25ea2432f55debec6c72248b01073b96e582a1` |
| `tools/test_build_qualified_desktop.py` | `857a0dc7f459a73bd8156a8164b529ac1399db41bd8e934a73367fb9a44bee7c` |
| `docs/api/qualified-desktop-build.md` | `b157f2acb0d9c0279658cb80e93f1f7b547960c47e385cae42558f2b0169f9aa` |

All three hashes matched the requested pins before and after independent test execution. `git diff --check` passed.

## Independently executed contract evidence

| Command | Result |
| --- | --- |
| `python3 -B tools/test_build_qualified_desktop.py -v` | 8 passed, exit 0, 6.398 s |
| `python3 -B -O tools/test_build_qualified_desktop.py -v` | 8 passed, exit 0, 6.107 s |

These were real executions of the tool's contract suite in temporary Git repositories. The Cargo/rustc programs are explicitly fake fixture programs, including explicit non-native version text and fixture binary bytes. No result above is a native compile or native smoke claim. The `-B` switch prevented Python bytecode writes in the reviewed repository.

## Review basis

The wrapper checks a full lowercase SHA-1 commit, checkout-root identity, exact clean HEAD, and nonignored untracked absence. It exports exact recursive Git blobs with regular Git modes and bounded count/per-file/total material, rather than using archive attributes or copying ignored developer inputs. Unsafe paths, symlinks, gitlinks/submodules and Git metadata paths are refused. A postbuild check compares the exported set, bytes and Git executable-mode semantics and rechecks original clean HEAD.

The recipe requires tracked Cargo/lock/toolchain inputs, a fixed 1.92.0 toolchain declaration, fresh output/home/temp/target directories outside Git, and explicitly selected trusted toolchain directories. It refuses Cargo configuration from export/ancestor/Cargo-home locations, external workspace discovery and explicit Cargo path inputs outside the export. Inherited build variables, wrappers, loader injection, targets, profile overrides and source/material stamp values are discarded. The recorded recipe fixes release, desktop feature selection, disabled defaults, locked dependencies, one job, no incremental compilation and optional recorded offline mode.

The material hash covers the exact canonical manifest bytes, including source commit/tree, exact tracked inventory, controlling wrapper hash, fixed recipe and trusted launcher paths/hashes/verbose version declarations. Placeholder paths and a derivation marker avoid fresh-output dependence and self-reference. The actual invocation and timeout are retained separately. After successful subprocess exit and material/source/controller/tool checks, only root-package Cargo artifact messages for both exact binary names are accepted; their paths must remain regular, non-symlink files inside the fresh target. The completed sidecar records actual binary sizes/hashes and the prebuild material and recipe hashes. Retaining an external sidecar digest remains necessary for later custody comparison.

The suite meaningfully exercises exact blobs despite export-ignore/subst attributes, executable-mode preservation, ignored-input exclusion, environment clearing, exact stamp injection, output-location-independent material, missing ignored include failure, dirty/wrong/untracked source refusal, fresh-output boundaries, unsafe paths/symlinks/gitlinks/source limits, Cargo configuration and external paths, and failure/mutation/malformed/missing/symlink artifacts without completed receipts. Explicit unittest/require checks remain operative under optimization.

## Preserved limits

The declared Cargo/Rustup homes and PATH are trusted external state. Launcher hashes do not identify an entire installed toolchain, SDK, linker, dependency cache or system library closure. Committed build code, procedural macros, dependencies or absolute includes can read external files; the wrapper is not a sandbox and does not attest those reads as tracked source material. Postbuild equality is a custody check, not authentication against malicious concurrent writers. Git/tool metadata and Cargo message limits are checked before interpretation, not a total process/RSS or log-storage budget. Timeout controls the immediate subprocess invocation rather than proving descendant termination.

These limits are compatible with the stated bounded purpose and explicitly disclosed in the module/docs. This review does not extend them to hermeticity, reproducible binaries, scientific/model qualification, a trusted signer, or automatic Git stamping in ordinary Cargo builds.
