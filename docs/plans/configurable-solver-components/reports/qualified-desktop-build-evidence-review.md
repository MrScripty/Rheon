# Qualified desktop actual-build evidence independent review

Read-only evidence review for clean source commit `dfde6dcb889e561ad0e3e4d7fd7f00bbffde21bb`, tree `dcda9b837cfe5d629af595f04734b356617e1581`. No Cargo, compiler or full build was run by this reviewer. I independently audited retained files/hashes and replayed the four existing small identity-consumer executables; I did not execute either release GUI binary, a model, or numerical qualification.

## Assessment

No material evidence-binding finding. The retained artifacts support an actual controlled release build of both native binaries from this exact tracked source/material identity, plus real native execution of the actual shared stamp/desktop identity sources in four separate compiled-declaration cases. This is sufficient evidence for that bounded A8 build-custody slice. It does not establish GUI/frozen-rerun user workflows, complete compiler-read closure, hermeticity, reproducibility, authenticated provenance, model/numerical/hardware qualification, or complete-plan acceptance. Hosted gates and the still-running packet work remain separate.

## Retained exact artifact identity

| Artifact | SHA-256 |
| --- | --- |
| `/tmp/rheon-qualified-desktop-controlled/build-sidecar.json` | `867394d410b8e24f4af20de9c80c7b6b8ab5efa55bc0944421667f68c74adb4f` |
| `/tmp/rheon-qualified-desktop-controlled/build-material.json` | `9e99d78b18f4edd7ef38b02619b63a893d8583d86caeb6e3b38a7b93ffe1cd7d` |
| `build/release/rheon-desktop` (21,878,624 bytes) | `2d5c0df4a372633bf30f105d8e9d8c4342ac330d85b5f29494eb29a56a53d8a9` |
| `build/release/rheon-ai-compare` (19,403,304 bytes) | `257af94d14d465cbe35fb590fa375ce17f0509a6e8d44293ca20f57dfab49afe` |
| `cargo.jsonl` | `0fd492dde4a712bc853628feb2b64de29dcc962e01548e806bb407de7c1ef2ff` |
| `cargo.log` | `11a832f8986de690e19c01ccd9bb0457df343819bdc512437f52cb4790e7650f` |
| `/tmp/rheon-qualified-desktop-consumers/receipt.json` | `300e82910bd90e554d75e6574d2feaea1ae68567f155841dca04b31eeb2ef994` |
| `/tmp/rheon-qualified-desktop-consumers/consumer.rs` | `198fa1c35edb16e2d0555a7676b11719b8e41d16532c812dbf4dabf2a4b43f87` |
| `/tmp/rheon-check-qualified-desktop-consumers.py` | `4bb8d0b1a13569c99ff2e90c999a7b25025a564bd1fd78c7fc0fb061ac039b1c` |

The release executable paths are relative to `/tmp/rheon-qualified-desktop-controlled`. The sidecar canonical recipe digest independently recomputed as `e06f94c9c244346c40bb4f5a943379e7d6c672eaac2b670fdc5955c33c7d31df`.

## Independent custody checks

I compared the complete material inventory against exact `git ls-tree` entries for the commit: 1,213 files and 14,061,263 tracked bytes. The exported file set exactly matched; each file's bytes/length/SHA-256 and executable Git mode matched the material, and an independently computed Git blob SHA-1 matched the committed blob. No extra exported files were admitted. Commit/tree, sidecar/material association, wrapper hash and current selected launcher hashes all agreed. Both material/sidecar hash pins matched those supplied by the lead.

The recorded controlled invocation explicitly passes the exact source commit and exact material digest, selects Rust 1.92.0, locked release desktop builds with defaults disabled, a new target/home/temp, one Cargo job and no incremental compilation. The material records actual verbose Cargo/rustc versions and the selected rustup launcher hash; the common launcher hash remains a launcher identity, not a complete installed toolchain inventory.

The retained Cargo JSON ends with successful build-finished and contains exactly the two corresponding root-package release binary artifacts, both `fresh:false`, at the same paths subsequently pinned. The build log records an actual optimized release completion. I independently rehashed both binaries, checked exact recorded lengths and regular non-symlink files, and confirmed both contain the exact source commit and material digest declarations. I did not infer this build from filenames or prior target artifacts.

## Four real compiled consumer cases

The external harness compiles the actual shared `build_identity.rs` and actual desktop `identity.rs`, not a duplicated formatter. I verified those referenced checkout sources remain byte-identical to the exact exported committed files. Its runtime assertions check complete formatted identity, process-local incrementing run ids and equal source stamp across identities. The retained compiler logs are empty, the four resulting files are actual ELF executables, and their receipt reports successful runs.

I independently executed those four already-built consumer binaries with the expected exact arguments. All exited 0, had empty stderr, and produced exactly the independently derived and previously retained output:

| Case | Observed source | Observed tracked material |
| --- | --- | --- |
| Ordinary (both declarations absent) | `build revision unspecified` | `unspecified` |
| Malformed declarations | `build revision unspecified` | `unspecified` |
| Valid source, malformed material | `dfde6dcb889e561ad0e3e4d7fd7f00bbffde21bb` | `unspecified` |
| Qualified declarations | `dfde6dcb889e561ad0e3e4d7fd7f00bbffde21bb` | `9e99d78b18f4edd7ef38b02619b63a893d8583d86caeb6e3b38a7b93ffe1cd7d` |

Every output used `rheon-desktop-configured-session/0.1.0; source=...; tracked-material=...` through the real desktop identity constructor. Consumer binary pins:

| Case | SHA-256 |
| --- | --- |
| Ordinary | `ce9cc3b62eb33df40e3585ca6a3bc4955e47f2a4815b503d7be31cacd0b3bf6d` |
| Malformed | `b4a99189df0516c3daf580b530473587b08fa87a86398babf0d667254c502f3d` |
| Valid source/malformed material | `ede769848c708526becf1174c86aaf191939dbaa834cb8644230a771c12b6f33` |
| Qualified declarations | `8e699c68066341664953b7aeb5237d2e0b5c9ea04c16dae75b2a63fb1a47d8b1` |

This replay verifies compiled consumer behavior; I did not independently recompile the harness. It uses an existing Rheon library for the public identity type and does not execute the actual release applications' window/recording actions. Embedded declarations in both release artifacts and the prior source review establish the propagation connection without granting GUI or frozen-execution workflow evidence.

## Separate qualification receipt and limits

Also read `/tmp/rheon-qualified-desktop-qualified/qualification.json`, SHA-256 `3fdc7f3a0655bd1cfd935a82c8e6aa28bdb7f901a93b0b8703bc4a80e810a924`. Its exact source/tree and all 1,213 recorded source hashes agree with the controlled material inventory. It records all 18 commands exiting 0 and status passed; I independently rehashed all 28 retained artifact entries successfully. This is verification of the retained qualification receipt, not a fresh rerun, and it explicitly records `lean_checked:false`. Packet completion was not required for this bounded evidence audit and is not asserted here.

All evidence remains local custody declarations requiring retained external hash pins. Trusted Cargo/Rustup homes, launcher-selected installed tools, dependencies, linkers/system libraries and external reads by committed code are not fully inventoried or isolated. Postbuild checks are not authentication against malicious concurrent writers. No source identity is equated with reproducibility or scientific acceptance, and these receipts must not transfer to later composed worker/source revisions without fresh qualification.
