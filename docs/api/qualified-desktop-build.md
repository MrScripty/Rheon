# Controlled desktop build material

`tools/build_qualified_desktop.py` builds `rheon-desktop` and
`rheon-ai-compare` from an exact, clean, committed Rheon HEAD. It follows the
tracked-only export pattern in `tools/verify_compressible.py::capture`; it does
not compile the developer checkout or reuse a previous target directory.

This records **tracked source material and declared trusted build tools**. It
is not a signature, authentication, hermetic build, complete compiler-read
inventory, reproducibility guarantee, numerical qualification, model pass, or
hardware qualification. The word “qualified” names this bounded build recipe.

## Invocation

Python 3.11 or later, Git, and the already provisioned trusted native Rust
1.92.0 toolchain are required. Run after committing the intended source:

```sh
python3 tools/build_qualified_desktop.py \
  --source-repo /absolute/path/to/Rheon \
  --source-commit <full-40-lowercase-hex-HEAD> \
  --output /absolute/path/outside-any-checkout/new-desktop-build \
  --cargo-home /absolute/trusted/cargo-home \
  --rustup-home /absolute/trusted/rustup-home \
  --toolchain-path /absolute/trusted/cargo-bin:/usr/bin:/bin \
  --offline
```

`--output` must not exist; its parent must exist outside a Git checkout.
Existing output is never replaced or resumed. `--offline` is optional and is
part of the recorded recipe. Without it Cargo may obtain locked dependencies
through the declared Cargo home. `--timeout-seconds` defaults to 1800 and is
bounded to 1–86400 seconds; the operational timeout is recorded in the sidecar.
This wrapper supports Rheon’s SHA-1 Git repository and fixed toolchain, not
every Git object format or cross-compilation setup.

## Selected inputs and refusals

The source repository must be its checkout root, at the supplied full commit,
with no tracked modifications or nonignored untracked files. Ignored files are
permitted in the checkout but never copied. Every recursive Git tree entry must
be a regular `100644` or `100755` blob. Symlinks, submodules, unsafe relative
paths, control characters and Git metadata paths are refused.

The wrapper reads exact blobs using `git ls-tree` and `git cat-file`, rather
than extracting an archive. Consequently archive path traversal is absent and
`.gitattributes` `export-ignore`/`export-subst` cannot remove or rewrite material.
The export is limited to 20,000 files, 64 MiB per file and 1 GiB total. The
versioned material manifest is limited to 16 MiB. A new source directory has no
`.git`, ignored files or copied developer target directory.

Tracked `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` are mandatory. Cargo
configuration in the source export, its ancestors or the declared Cargo home is
refused, including nested tracked `.cargo/config` and `.cargo/config.toml` files.
An ancestor `Cargo.toml` outside the export is also refused, preventing implicit
discovery of an external Cargo workspace.
Declared Cargo `path`, package build/workspace paths and workspace member paths
must stay within, and exist in, the tracked export. This deliberately restrictive
profile also treats metadata keys named `path` as path declarations. Unsupported
configuration needs an explicitly extended recipe; it cannot silently acquire
the captured source identity.

The caller explicitly selects absolute Cargo home, Rustup home and PATH
directories outside Git checkouts. These are trusted developer inputs, separate
from the tracked-source claim. Their declared paths, selected Cargo/rustc
launcher hashes and actual verbose version outputs enter the material manifest.
A Rustup launcher hash is a launcher hash, not a hash inventory of the complete
installed toolchain, SDK, system libraries or dependency cache. Cargo home and
Rustup home remain external state and can be updated by their owning tools.

All other inherited environment variables are discarded. In particular,
inherited Rust flags, encoded flags, compiler/linker wrappers, Cargo profile or
target overrides, target directory, loader injection and preexisting Rheon stamps
are not build inputs. The controlled environment selects Rust 1.92.0, desktop
features, release profile, disabled default features, `--locked`, one Cargo job
and no incremental compilation. HOME, TMPDIR and CARGO_TARGET_DIR point to fresh
directories in the output. Neither binary is executed by the wrapper.

A missing relative `include!` input that existed only as an ignored/untracked
checkout file is absent from the export and must fail compilation. This is not
filesystem isolation: committed code, including `build.rs`, procedural macros or
an absolute include, can deliberately read external files. Such external reads
are an **unsupported provenance dimension**, not silently counted as tracked
material. Locked dependencies and trusted tools can also read external state.
The manifest therefore calls its file inventory `tracked_source_material`, not
“all compiler inputs.” Use a separately owned isolation mechanism if complete
read closure becomes a requirement.

## Material identity and retained results

Before Cargo build, `build-material.json` records schema/version/kind, source
commit and tree, every tracked file’s Git blob/mode/length/SHA-256, the wrapper
SHA-256, fixed command/features/profile/flags, controlled environment recipe and
declared trusted tool versions. Sorted-key UTF-8 JSON with two-space indentation
and a final newline defines its exact hashed bytes.

The build receives `RHEON_SOURCE_REVISION=<full HEAD>` and
`RHEON_BUILD_MATERIAL_SHA256=<SHA-256 of those exact manifest bytes>`. The
manifest contains a literal derivation marker for its digest, avoiding a
self-referential hash. Temporary output/source/target/home paths and the source
checkout location do not enter the material identity. Stable placeholders define
fresh directories; their actual locations and timeout enter the sidecar. The
declared toolchain-home and PATH locations intentionally remain material inputs.
Moving only the fresh output directory does not change material identity; this
does not promise equal binaries from arbitrary repeated builds.

After a successful Cargo build, the wrapper checks that HEAD stayed clean and
pinned, the tracked export retained its exact files/modes/bytes without extra
files, and the material manifest, wrapper and selected launcher bytes did not
change. It accepts exactly one root-package Cargo binary artifact for each
required name, inside the fresh target, with no symlink path. It does not infer
success from a matching filename in an older directory.

Only then does it create `build-sidecar.json`, containing:

- schema/kind and `completed` status;
- source commit/tree and the prebuild material file’s SHA-256;
- the canonical recipe SHA-256;
- actual binary paths, lengths and SHA-256 values;
- actual controlled environment, source/export/target locations and timeout;
- the same explicit provenance limits.

The CLI prints the sidecar’s SHA-256 for external retention. No binary digest is
embedded into that binary. The sidecar and source/material stamps are declarations
and custody evidence; retain an external sidecar pin when comparing artifacts.
They provide no authentication of a person or scientific result.

Cargo stdout/stderr are retained as `cargo.jsonl`/`cargo.log`. Cargo metadata is
limited to 64 MiB before artifact decoding. Failure after output creation retains
`build-failed.json` and any partial material/logs, and creates no completed
sidecar. A partial output is not a qualified build and is never reused.

## Contract verification

```sh
python3 tools/test_build_qualified_desktop.py -v
python3 -O tools/test_build_qualified_desktop.py -v
```

These tests create temporary Git repositories and explicitly fake Cargo/rustc
programs. They verify exact exports, environment stripping, material/stamp/hash
binding, output-location independence, strict refusals and honest failures.
Their fixture binary bytes are not native executable evidence. Only running the
wrapper with the declared real toolchain can establish an actual native build;
GUI smoke, numerical qualification and measured-model evidence remain separate.
