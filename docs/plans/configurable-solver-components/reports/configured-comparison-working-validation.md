# Configured comparison composed validation

Worktree: `/workspace/rheon-configured-comparison-ui`.
Frozen base HEAD: `254dd4913a7b89e376996feccca6f3427834e2db`, plus root-owned composed working delta.
All 11 current paths and their exact hashes are recorded in
`/tmp/rheon-configured-comparison-ui-source-pins.json` (SHA-256
`138cc9a759b7dee78da63b83960c9df1a2b7313eef1829c574de2217957b99e4`).
This receipt does not qualify a later integration commit automatically.

Owned new test file: `src/bin/desktop/configured_comparison_tests.rs`, SHA-256
`e6716e8c707f30cc20304b16e3201f8fb1fd838650bd08a43a95ab4d68cbbf46`.
The sole author edit during execution corrected the egui fixture's extra-render
timestamp to use its existing explicit monotonically increasing clock. No
assertions or production/helper source were changed by the test author.

Each Cargo command sourced `/workspace/rheon-setup/env.sh`, then explicitly used
`CARGO_TARGET_DIR=/workspace/.rheon-tools/target-configured-comparison-ui`,
`CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=1`, `--locked --offline`,
`--no-default-features --features desktop`.

1. `cargo test ... --bin rheon-desktop configured_comparison_tests`:
   **5 passed, 0 failed, 0 ignored**, 26.97s execution after rebuilding the test
   binary. The source/first-failure and focused execution details are in
   `/tmp/rheon-configured-comparison-ui-focused-receipt.md`.
2. `cargo test ... --test comparison_archive_contract --test comparison_field_view_contract --test saved_comparison_ui_contract --test saved_source_contract --test configured_archive_binding_contract --test configured_recording_comparison_contract`:
   **63 passed, 0 failed, 0 ignored**. Counts respectively 12, 9, 9, 6, 17, 10.
   Raw stdout/stderr: `/tmp/rheon-configured-comparison-ui-regressions.log`.
3. `cargo clippy ... --bins --tests -- -D warnings`: first run found one
   production `needless_borrow` in `comparison_field_view.rs:271`; root removed
   the unnecessary borrow. The field-view hash after that owner fix is
   `ec8ea0799522d5a7fadac47b06b42f26717f935bbb9a1677fc5052f096c1cc82`.
   Exact command rerun **passed**, 47.39s. No behavioral rerun was required for
   this borrow-only correction. Raw successful output:
   `/tmp/rheon-configured-comparison-ui-clippy.log`; first failure retained at
   `/tmp/rheon-configured-comparison-ui-clippy-first.log`.

The five new tests use actual existing Worker/CPU/Session/SQLite owners,
independently known hydrostatic pressure, actual headless egui events, linked
field probes and Inspector clipboard CSV. A separate real producer process
exits before a fresh reader process opens saved artifacts with an empty model
PATH and removed package files. Stored unverified metrics are never accepted
without exact-source recomputation. These contracts establish no learned-model
accuracy, physical qualification beyond the named owners, performance, hardware,
interactive-window, source-admission or later-commit qualification claim.

Source admission and final committed integration qualification remain root-owned
and pending. No commits, pushes, merges, inventory changes or shared-target Cargo
were performed by the test author. The isolated UI target is idle and released.
