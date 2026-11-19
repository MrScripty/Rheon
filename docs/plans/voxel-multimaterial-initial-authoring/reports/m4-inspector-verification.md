# M4 retained voxel inspection verification

The read-only typed Inspector component is Accepted. Full M4/A5/A6, real worker launch/frozen request/save-load/cold rerun and actual native integration remain pending. This component exposes accurate inspection of already-recorded data; it does not enable execution or introduce a new recorder.

[Separate complete review and targeted rereview](m4-inspector-independent-review.md) resolved R1–R3. An unused proposed recording adapter was removed instead of extending a needless producer API; its partial-float header equality could have overlooked geometry signed zero. A selected-data accessor is test-only. Standalone typed records use generic Liquid/occupied Gas names because the core record API accepts arbitrary captured pairs; coefficients alone do not establish Water/Air identity. Qualified authoring remains Water/Air. A new actual egui mixed-cell/keyboard fixture checks the new view rather than merely requiring nonempty shapes.

Executed with pinned Rust1.92.0 / locked dependencies:

| Check | Result | Ignored local log |
| --- | --- | --- |
| `cargo test --locked --features desktop --bin rheon-desktop inspector::` final reviewed source | 4 passed,0 failed | `.generated/voxel-material/m4-inspector-final-tests.log` |
| Strict desktop all-target Clippy | Passed | `.generated/voxel-material/m4-inspector-final-clippy.log` |
| Formatting/whitespace and source identity | Passed; three source hashes equal separate review | `cargo fmt --all`, `git diff --check`, SHA-256 command |

The four fixtures comprise2 existing particle/carrier Inspector regressions and2 new voxel fixtures. The first actually writes/opens SQLite, selects original data, advances the separate numerical owner and appends through the existing recorder, renders egui, then checks retained independence, observed invalid-selection clearing and explicit later selection. The second records a generic nonreference pair with captured densities3/2kg/m³ and mixed F0.5. Actual AccessKit-labelled `Cell j/k` controls are reached by Tab and edited through Ctrl+A/Text/Enter. The rendered view must contain independently specified phase volumes0.0078125/0.9921875m³, masses0.0234375/1.984375kg, physical center[0.125,0.375,0.375]m and F/(1−F)=0.5. The original liquid_fraction scalar index20 and exact0.5 CSV value must match; unsupported Water/Air labels must be absent.

These are owned-field/derived-display and egui interaction contracts, not a real native window or solver scientific validation. Display sums are finite f64 approximations, not conservation acceptance or independently captured diagnostics. Color is role-based appearance only, with explicit mixed-cell legend; no field, material ID, fraction or domain resolution is edited. Simultaneous decoded/raw publication memory remains a mandatory whole-native budget obligation at worker integration. Generic integrity audit does not replace the saved profile's full pinned recipe/publication audit.

Initial strict lint identified premature unused APIs and ordinary clamp/nested-if style; they were removed/fixed without suppressions. No physics formula, old archive, export contract, dependency, worker or launch gate changed. Main, prior denied operations, uncertain Library delivery and all external publication remain untouched. Next is the real existing-worker integration against these accepted input/record/archive/inspection owners.
