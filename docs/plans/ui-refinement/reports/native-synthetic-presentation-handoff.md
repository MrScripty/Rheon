# Workerless native retained/timeline handoff

Use the separate published harness checkpoint after425a986, exact Git commit/tree
and three source seals from evidence/synthetic-presentation-20261020/manifest.json.
No existing DB reconstruction, worker, solver, model, recorder, run/Cancel or rerun.
This verifies presentation using two tracks×two ordered literal2³preview frames,
supplied timestamps0/0.25s and unmeasured calculation duration; never call scientific
or archive/cancellation evidence. Persistent synthetic annotation stays visible.

Build/verify in the existing authorized native environment:

```sh
cargo test --locked --offline --features desktop --bin rheon-desktop -- --list
cargo test --locked --offline --features desktop --bin rheon-desktop native_presentation_harness::synthetic_retained_history_timeline_and_disabled_modes_are_workerless -- --exact --test-threads=1
cargo rustc --locked --offline --features desktop --bin rheon-desktop -- --cfg test
```

The third command does NOT use --test: it creates the standalone cfg(test) main-thread
presentation entry. Normal cargo test uses the compiler-generated unit-test runner;
the native function is not #[test], so enumeration must show only the one pure probe
in this module. Normal cfg(not(test)) main keeps its original production body.
No Cargo/lock/dependency change or production CLI/env switch was added.

Immediately copy the emitted target/debug/rheon-desktop into a uniquely named sealed
path outside the normal target filename and record its SHA256/size/config/source.
Then run the ordinary locked offline desktop cargo build to restore the normal
production target before other use. Execute ONLY the uniquely sealed harness path
under the validator's existing native process supervision. Resizing is native;
default1250×820, minimum600×430. A Close button is always outside the disabled canvas.
The monotonic300s self-close is cooperative, not a hard process limit: external
supervision remains required. No native launch occurred in this source environment.

Local compiled final-guard artifact (not a cross-environment reproducibility claim):
`/workspace/scratch/rheon-ui-presentation/rheon-ui-synthetic-presentation-testcfg-guarded-425a986-increment.bin`
235185520bytes; SHA256
040715719f4e34e04951c8ad428024cbdbd219d7b65ac95d64d54e3e6e8d5cbd.
Never executed. Earlier pre-guard artifact is preserved but is NOT the handoff target.
Standalone build emits258unused-code/import warnings from cfg(test) helpers outside
a test runner; no warning suppression or automatic fix. Ordinary production restore,
strict production Clippy and check pass cleanly. Do not call the native cfg build
strict-lint clean. Native builds should seal their own binary, not assume identical
hash across paths/toolchain environments.

At1250×820/900×650 and100/150% scale, capture actual content rect/scale and Scene state.
Default follow_live=false,track/index0; dark8×7 style,280px sidebar reservation except
open narrow Scene. Check Play/Pause, current frame/timestamp/track, slider endpoints,
speed text editing/tab/focus/bounds, track dropdown/retained selection. Switch Draft:
playback pauses, timeline hides and retained index is preserved. Return Retained:
track/index and literal frame remain coherent. Scene hide/reopen and long captions
should keep compact controls. The wrapper refetches selected frame after controls as
production does. The central actual Appearance renderer is disabled and hard-guarded
before any numerical lab dispatch: this harness cannot validate viewport gestures.
The pure probe executes disabled tab clicks, not native canvas/key input usability.

Capture actual quality85JPEGs labelled exact source, size/scale and SYNTHETIC state.
Do not represent them as a simulated run or use them for normal viewport/force-handle
validation; use the production425a986+ checkpoint for those separate idle gestures.
Save confirmed Library IDs only when actual uploads succeed; no local IDs are claimed.
