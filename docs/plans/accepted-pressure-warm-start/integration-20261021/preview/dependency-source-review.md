# PR59 independent dependency-composition preview review

2026-10-22. Workspace `/workspace/pr59-composition-preview`, HEAD `69754219348d7391c1f5deac685c43b1981382ee`, pending merge of reviewed PR143 `159fa74bd271f8fcc07f3837d66e4738925c820c`. **No blocking Cargo/source-closure defect found in this frozen preview.** This is a preliminary comparison and per-path assessment, not source admission approval, inventory refresh, numerical qualification, final-main acceptance or merge authorization.

## Cargo resolution

The sole Cargo conflict is resolved additively. In memory, removing only the PR59 nondefault `accepted-pressure-warm-start` feature/comment and `accepted_pressure_lab` required-feature example declaration reproduces PR143's **entire Cargo.toml bytes exactly**. Static TOML parsing confirms inherited `[dev-dependencies]` remain precisely `num-rational = "=0.4.2"` and `num-traits = "=0.2.19"`, under their proper table after the example. No dependency moves into a production/build table, no dependency version is changed, and no unrelated manifest content is lost.

Cargo.toml current SHA256: `40f59222652c9d6b9236dd85b2beab83e8e7e9a8fdd8418bf3bd617801724487`. Cargo.lock is byte-identical to PR143, SHA256 `e64beca355b3963812b80d49ac5b17d0b1626f03f5d651b50e4dd25823dce96f`. Exact inherited lock content/registry identities are retained; no package resolution, dependency download or license/artifact retrieval was performed. Prior dependency review belongs to its existing owners. The index still has the expected in-progress Cargo conflict entry, with resolved working content; final commit binding remains required.

## Complete conservative selected closure

Independently reconstructed the unchanged `reviewed_native_paths` selection: every file/symlink under `src` and `.cargo`, root rust-toolchain matches, and all explicit qualification inputs with optional root build.rs presence. Parsed only the constant input list; did not import the module or execute admission, qualification or numerics. Every selected current path is a regular nonsymlink file. Verified all 152 inherited map hashes against PR143 committed blobs and its committed selector closure. No source was dropped by using an incomplete component-branch map.

| Comparison versus PR143 reviewed map | Count |
|---|---:|
| Inherited selected paths | 152 |
| Current selected paths | 153 |
| Added | 1 |
| Changed | 7 |
| Unchanged | 145 |
| Removed | 0 |

Added: `src/pressure_initialization.rs`.
Changed: `Cargo.toml`, `src/lib.rs`, `src/liquid_step.rs`, `src/pressure.rs`, `src/session.rs`, `src/session/projection_tests.rs`, `src/simulation.rs`.

The full machine-readable map, exact before/after hashes and a disposition/owner assessment for **every153 path** are in `/tmp/pr59-final-deps-preview-map.json` (SHA256 `741d85d76807c22eae753a98faa60cb777e78e4b60d9f765c596a5a22bfaff3e`). Canonical sorted compact map SHA256 (no trailing newline): inherited152 `9a10497158827e189fe505f65e1aa203cd8a28fe894c469408977071b055375f`; current153 `8055a53c319ddec6aade4bed35fdc642c7ab07839e4e0bb2866a00a4a03d3688`.

All seven changed paths plus the one addition are owned by PR59's already reviewed warm-start integration; only Cargo acquires the additive inherited development table at this stage. The inherited145 paths match their exact reviewed PR143 identities and retain prior dispositions. This is a complete byte comparison, not a claim to have newly recertified every inherited scientific/runtime path.

## Earlier review and owned deltas

Verified all **24 other files** from the earlier25-file composition review (excluding Cargo.toml) retain their recorded hashes exactly. That includes all reviewed warm production/contract bytes and projection/workflow/recording support. Native projection exclusion, explicit Cold routing, immutable Session intent guard, final prospective receipt sealing, accepted-history publication atomicity, numerical gates and feature exclusion therefore retain the prior source-review conclusion. No new src conflict or source delta was introduced by this dependency merge. The complete24-file verification list is included in the machine map, including integration tests outside the conservative native selector.

Per-path changed/addition ownership:

- `src/pressure_initialization.rs`: PR59 new feature-only process-local accepted history/provenance/key module; exact prior-reviewed source, no numerical seed array and no portable replay claim. SHA256 `dc81cb8c5b9dd1719a8e513e87d655098688b18e32bbea576b72746cadd1a233`.
- `Cargo.toml`: Additive PR59 Cargo boundary: nondefault accepted-pressure-warm-start feature and accepted_pressure_lab required-feature registration; exact inherited PR143 development requirements retained. SHA256 `40f59222652c9d6b9236dd85b2beab83e8e7e9a8fdd8418bf3bd617801724487`.
- `src/lib.rs`: PR59 gated warm-initialization module and public policy/provenance exports; PR142 projection/workflow exports retained. SHA256 `c05e7887e95a97d1db1e5407a9b6d76d4cc94b419b1fb0265d42d4120ab38c3c`.
- `src/liquid_step.rs`: PR59 gated accepted owner history, explicit policy/seed selection and staged commit; native projection excluded from eligibility and its private dispatch forced Cold; PR142 transport/receipt/workflow sealing retained. SHA256 `c3c64645e0af3b11bf279c0ba4fb2740db1b785f046f115d084d136f436d43f5`.
- `src/pressure.rs`: Exact previously reviewed PR59 source: borrowed finite gauge-zero pressure initialization and seeded-only reduced-recurrence replacement; full compatibility/residual/divergence and cold gates retained. SHA256 `fb70a35516764666acbedffb3c8cb2fbdc9c63690cfe87701085a9a5bf4c8827`.
- `src/session.rs`: PR59 API3/capability plus initialized legacy-only entry, actual pre-step clock and acceptance-only metadata clearing; immutable projection guard retained. SHA256 `69a152d5b2b8dfe36888b2dd243b4069fdc2cd17268a201f06027d5c1bd4bb8f`.
- `src/session/projection_tests.rs`: PR59 feature-only cold projection/no-seed/cancellation/refusal/retry integration test, with inherited PR142 tests unchanged. SHA256 `360e7feb3f4955903510c82d0711820ad4aa922ffea075fd7084028d07f57619`.
- `src/simulation.rs`: PR59 feature-only borrowed initial-pressure propagation/native solve branch composed alongside inherited force/workflow/projection inputs; other entrypoints remain cold. SHA256 `509c853d7232d22fecbd14066178befb3174b34e0aedaf4bdc27e0b8263d86fe`.

## Admission and remaining limits

The integration inventory, its complete review record, frozen v1/v2 inventories, selector/checker, qualifier and oracle are byte-identical to PR143; their exact hashes are in the machine map. No admission file, source selector, CI matrix, numerical threshold, oracle or qualifier was changed by this review. The inherited152-path inventory is intentionally stale for the present153-path warm composition and must not be reused as admission. A present stale inventory continues to require refusal; no gate was invoked or bypassed here. This report's machine map is labeled an assessment and explicitly states `admission_approval: false` and `inventory_refreshed: false`; it is not a replacement admissible inventory.

Actual ordered #142/#138/#143 main integration remains pending under the delegated context. Final source/commit binding, lead-owned explicit admission, warm CI coverage, fresh allowed qualification, exact-head checks, fixed prospective controls and lead authorization still remain. Ordinary test results running under the primary worker are separate; no test, build, numerical campaign, timing, cloud/GitHub action, artifact operation or binary edit was executed here. Existing three CV3-A explicit ignores were not selected or changed. Historical **+2.20%** particle adverse evidence and the strict positive/inconclusive performance block remain preserved. No older whole-Git receipt qualifies this new composition.

Scoped Cargo/source whitespace check passed. No denied or uncertain mutation arose. Only the two /tmp assessment deliverables were written; source admission and final qualification are not approved by this review.
