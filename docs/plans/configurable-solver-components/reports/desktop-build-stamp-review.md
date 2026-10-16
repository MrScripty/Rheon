# Native compiled build stamp independent source review

Reviewed five current Rust files in `/workspace/rheon-qualified-desktop-build`, base `f5bd8b5cc468415df2ab5c3347d8046f813602ef`. Read-only review; no Cargo or native execution. `git diff --check` passed. The separate wrapper/tool/tests/docs were unfinished and are excluded from this assessment.

## Assessment

No material source finding. The bounded central declaration formatter and its registrations/call sites are suitable for source admission and the lead's fresh validation. This is source approval of declared compiled identity propagation, not proof that a caller's declarations identify actual build inputs or an artifact. Full A8 qualification still requires the build-wrapper/material and actual compiled-consumer evidence.

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/build_identity.rs` | `a94fea1f039a49f8d0eb79b019b3fbd0f2c9da1762ba854d10cba1e5fd87a25b` |
| `src/bin/desktop/identity.rs` | `df451a986476b9df252a9a4dbc1e33218bf5ab3cb268b83fa78b47dfffdae184` |
| `src/bin/desktop/ai_comparison/diagnostic_runs.rs` | `80559d9d69d8164c24b8dd45f71c51b972fc8af9349c9e56b3e36b14b827b41c` |
| `src/bin/rheon-ai-compare.rs` | `e102cccdc1a86afbdcf5a942ec6b1125655b9d1fcc7f4eb479935940633082bc` |
| `src/bin/rheon-desktop.rs` | `b484c75de20df8cb36010e31cd9cbab879e4f348050b2d551f2bcf553cb25d3f` |

## Source basis and limits

The central module accepts source declarations only as exactly 40 lowercase hexadecimal bytes and material declarations only as exactly 64 lowercase hexadecimal bytes. Missing/malformed declarations become explicit unspecified values, separately for each field. These are compiled `option_env!` declarations, not runtime environment values, Git queries, dirty-tree deductions, an automatic build script, signatures, reproducibility guarantees, or full compiler-read closure evidence. A well-shaped fabricated digest remains a caller declaration. The module says this explicitly.

Both native binaries register the same module, allowing the diagnostic submodule to resolve the same crate-level owner. Desktop identities use the same formatter for their existing backend role; their process-local atomic run-id behavior is unchanged. The formatter retains owner, package version, source and tracked-material declaration in one bounded string for the current fixed internal owner/backend values. Manually assigned public `SessionIdentity` values and their validation/API are untouched.

The diagnostic executor's capture and existing execution guard obtain the same new complete stamp. `validate_execution` still requires exact equality before execution; historical `validate`, decoding and header verification continue to accept and compare the archive's own historical declaration. The new formatting can deliberately make an older saved diagnostic stamp unequal even when its source part was unspecified. This refuses attribution to a different current stamp while preserving cold inspection; it does not prove that two matching unspecified builds have equal source. No schema migration or cold rewrite was introduced.

The native saved-carrier producer now supplies this declaration in both its fresh Session identity and build-provenance role rather than emitting a hard-coded unspecified provenance sentence. Existing native intent/archive association and digests continue to bind the declared strings. This change alone does not add a universal native frozen-rerun execution-identity guard; the explicit diagnostic guard and future configured execution guard retain their existing separate semantics.

The lead removed the small hex-helper mirror test before final pinning. Final reviewed source contains no such unit. That removal is reasonable: meaningful deciding validation should exercise actual compiled environment propagation, ordinary/malformed declaration behavior, both native consumers, historical cold readability and frozen execution mismatch refusal. I did not run those checks and do not infer their result from this review. The wrapper's handling of Git/material closure, ignored/untracked inputs, controlled environment, target isolation and receipt/artifact binding still needs its separate review and evidence.
