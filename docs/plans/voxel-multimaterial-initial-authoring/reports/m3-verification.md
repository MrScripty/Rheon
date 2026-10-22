# M3 authoring candidate evidence and native environment blocker

This candidate follows committed M2 `d0e92b7343599703978b660d3c057cf8602cc98f` in the isolated implementation worktree. M3 is not accepted: the required real native idle-edit workflow has not run. M4 saved input/native execution and M5 objective acceptance remain pending. No save/load, complete immutable run recipe, new recording variant or two-material retained-result consumer is present.

## Executed evidence

- Core `cargo test --locked --no-default-features --test initial_contents_contract`: eight tests pass, no failures, filters or ignores. Same eight pass with default features. They independently enumerate the material operation table, axis-aligned swept support, exact frozen fractions, owned initial bytes and positive gas mass; test no-op/cancel/redo semantics, typed Vacuum refusal, capacity failures, whole-command eviction and direction-scale/near-distance invariance.
- Desktop `cargo test --locked --features desktop --bin rheon-desktop initial_contents_ui::tests`: ten selected in-process egui fixtures pass. Pointer press/move/release are real egui input events, including coalescing, cancellation, invalid terminal input, Scene identity/legacy launch refusal, text-undo priority and hidden-page interruption. A separate actual-controls fixture renders the panel, checks accessibility names/units for Radius and both Centers, reaches the fields by Tab, keyboard-edits their values, then Tabs to Apply dab and Space-activates it. Exact IDs/revision match the pointer operation. The earlier direct numeric-dab-owner fixture remains a component check. In-process Tab/activation evidence does not qualify real native layout, scaling or graphics.
- Existing Scene regressions: eight pass. Existing force-handle regressions: six pass. Viewport regressions: nine pass, including the new integrated transition from an active force handle to Initial contents. The previous force owner is cancelled and drains its release before any brush/camera handoff, including release→new press→held move; its original force state and camera remain exact and a subsequent fresh contents press succeeds. Numeric view keys respect a focused text editor, with a positive viewport-owned shortcut case. These are in-process input/presentation fixtures, not native windows or new simulation cohorts.
- Strict `cargo clippy --locked --features desktop --all-targets -- -D warnings` and `cargo fmt --all -- --check` pass on the reported candidate. `cargo build --locked --features desktop --bin rheon-desktop` successfully builds the actual native executable.

Initial source checks found one closure result needing an explicit error type; corrected before the first desktop test run. Initial six UI fixtures passed. Desktop review collected three further required corrections: release/new-press batch ownership, numeric view shortcut ownership and named actual-control keyboard coverage. Serial fixes retain cancellation ownership while the final primary button remains down, guard all numeric view keys and label/qualify the numeric controls. The new keyboard fixture initially queried Label-role accessibility text through Node.label; inspection of egui's actual source showed Label.value is its contract, and correcting that lookup passed all ten tests. Original failed log is preserved. A final lint found a test default-field assignment; the equivalent struct initializer corrected it. No original material/history/gesture assertion was weakened. Added targeted cases/source corrections are recorded in independent review rather than treating the first six as whole-M3 acceptance. Core review R1 (history metadata selected before eviction) and R2 (ray parameter/near units) were corrected serially; focused regressions pass and targeted independent core rereview resolves both. Required UI findings and final corrections are owned by [m3-ui-independent-review.md](m3-ui-independent-review.md).

## Native startup: unavailable required environment

Environment inspection: Linux native x86_64, workspace-write sandbox, no `DISPLAY`, `WAYLAND_DISPLAY` or `WAYLAND_SOCKET`; neither Xvfb nor weston executable is available. No operator desktop session was inherited. No graphics/shared-memory configuration was inferred or runtime sandbox flag weakened.

Actual action: `target/debug/rheon-desktop` from the isolated worktree. It exits 1 before a native window or event loop can be created:

> WinitEventLoop ... neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.

This is an unavailable native display environment, not a passed startup or workflow. At that attempted startup, native executable SHA-256 was `f9c3f087b039463b1cd39f55c15f3100504c585531f1694cbc79407401c8faf3`. Later source-level corrections require rebuilding and a qualified native run when a display becomes available. No screenshot was captured or labelled actual UI; no native brush gesture, layout scale, simulation, persistence or performance acceptance is claimed.

A local-only display-server provisioning investigation ran:

`apt-get -o Dir::State::lists=/workspace/.voxel-apt/lists -o Dir::Cache=/workspace/.voxel-apt/cache -o Debug::NoLocking=true update`

All three configured Debian snapshot InRelease requests returned proxy HTTP 403. Targets: `https://snapshot.debian.org/archive/debian/20260828T000000Z/dists/trixie/InRelease`, the matching `trixie-updates/InRelease`, and `https://snapshot.debian.org/archive/debian-security/20260828T000000Z/dists/trixie-security/InRelease`. Outcome: metadata retrieval denied by proxy; no package installed and no display server obtained. Apt also reported a permission-denied read of `/etc/apt/apt.conf.d/80-applied-apt-retries` and its existing cache-clean hook could not remove `/var/cache/apt/archives/partial/*.deb`. Those denied targets were not retried or escalated; no alternate mirror/upload or protected-file change was attempted. Apt's zero exit despite warnings is not treated as successful provisioning.

The required M3 native idle-edit observation therefore remains blocked. Owner: parent/environment operator supplies a supported X11/Wayland graphics environment; root then rebuilds the exact candidate, records graphics/session/lifecycle facts and performs the bounded actual idle brush/undo/cancel/keyboard checks. The ordered M4 boundary is not crossed by renaming headless egui fixtures native validation. No new simulation/benchmark cohort is required to resolve this environment gap.

## Other unchanged blockers and scope

The prior external PR59 coordination comment was rejected by automatic approval review and remains unretried. Shared-source integration requires sole-lead coordination; main remains frozen. No push/PR/merge/deployment or uncertain Library write occurred. M1/M2 provide bounded numerical component evidence, not native feature acceptance. All A1–A6 remain pending for the composed feature.

Raw logs remain in ignored `.generated/voxel-material/`; the local provisioning log remains `/workspace/.voxel-display/apt-update.log`. Final deciding log identities:

| File | SHA-256 |
| --- | --- |
| `m3-corrected-contracts.log` | `566bc006a12b1a0e7a74b6b6ad5db2d7800bdfb27f036bfaaa2d580956755075` |
| `m3-default-contracts.log` | `0ae8377d4cf1448f842c448fdf8b90bec88509595204b2702a298e846a0081a8` |
| `m3-ui-keyboard-corrected2.log` | `b5a9461148bfc9f44c5c1d862dc74466a04f6e6fb0e189b21416d133e788ca60` |
| `m3-viewport-round-corrected.log` | `8ee0f62be667533dd0396649c973ee3cd8ddf3b7cc7827da00b1317a1ef91bd9` |
| `m3-scene-regressions.log` | `7be9b7e1d4a245d16c39ef21e78ea630aa60bec5db5227dd6d1df1ce452aa779` |
| `m3-force-handle-regressions.log` | `cdeedf161b78269f4ebbfbdb3f092543cb314b77db5c47b39e8e77bee02379f5` |
| `m3-core-clippy-final.log` | `392015e665a8c1367e333af656cdd600283683d3150a74f0ae8fb580abce5bf7` |
| `m3-desktop-clippy-final-corrected.log` | `a84629d590aed18c8876eee8b573f9eba178f6f554d033e8d7c8a183d06a0c62` |
| `m3-native-startup.log` | `099549005d4edfc6eff15616db07925e7980a569ada327e3aa8bc90f3b789220` |

Provisioning log SHA-256: `6a1a3a8104c3adeeb3b13c94a85cd09c47e8972d57e32516e821df35f478e468`. Independent reviewed source identities are in the separate reports. Final native build log SHA-256: `b2faf60a995f9379599b2f1b7423ce04e8a115477bf810d1deb8a13941a17130`; final built executable SHA-256: `731b13b7bed5657968c38a692490892ed4345a42442053a933382be56d722b14`. This final binary was built, not successfully launched. No generated binary, native image, package metadata or log is committed.

## Native idle-edit continuation procedure

After a supported graphics environment is supplied, record exact source/executable, display/backend/graphics and isolated state. Use one8³,1m closed draft, no worker or numerical steps: new Vacuum document, explicit Air fill, round Replace Water dab on Z layer4 at radius0.18m, interior Layer/Volume edits, exact Undo/Redo, Remove→Vacuum refusal and explicit Air restoration, interrupted/cancelled pointer tails and the labelled numeric coordinates/Apply path. Compare committed cell IDs/counts to the headless predicate and verify canonical force/contents selection plus text-local undo. Inspect supported1250×820 and900×650 views at actual available100%/150% scale; do not infer unavailable scales. No Simulate, recording, benchmark or physics workload is needed for this M3 gate. If it passes, record the actual evidence and admit the single next M4 slice; if it fails, fix/review the owning source before continuing. Do not edit a nonempty lattice resolution in place.

The final equivalent viewport fixture initializer was also executed separately through `cargo test --locked --features desktop --bin rheon-desktop viewport::tests::selecting_contents_cancels_previous_force_gesture_before_brush_or_camera`: one selected pass,218 intentionally filtered, no failure/ignore. Log `m3-final-handoff.log` SHA-256 `19a5b6455acc0e68fb8e332703be7d814cf8642a6d99cc2b7021300a0017f67a`. Final research/planning worktrees were checked clean at their original commits;28 local Markdown targets and fenced blocks were checked.
