# Native / trained AI pressure comparison

The desktop's **Compare Rheon / trained AI** entry and the dedicated
`rheon-ai-compare` binary run fresh filled-water scenes with a qualified native
reference and a separate raw AI diagnostic trajectory. The ordinary accepted
`Simulation` contract is unchanged. Diagnostic AI frames do not become accepted
simulation publications when pressure checks fail.

```bash
cargo run --offline --locked --features desktop --bin rheon-ai-compare -- \
  --deployment /path/to/deployment.json --project-dir /path/to/observations --auto-run
```

Deployment schema `rheon-pressure-cpu-deployment-v2` binds measured source,
checkpoint, train-only normalization, installed runtime and context policy.
The companion exporter is owned by rheon-surrogate-lab's
`experiments/live-water-comparison-v1`. No artifact hashes or machine-specific
Python paths are compiled into Rheon. The measured Python bridge speaks bounded
newline-delimited JSON. Its closed replies are decoded and correlated in Rust.
CPU float32 weights, zero optimizer updates and no CUDA initialization are checked
at the handshake. GPU training happens upstream, outside this app.

The current checkpoint needs a native prefix matching its training context:
15 qualified steps before the first AI call at step 16. Each prefix frame shows
**Native warm-up · AI not evaluated**. The worker enforces the deployment's minimum,
and the bridge rejects an early request as `unsupported_context`. Cold-start test
records do not establish model failure. With a prefix, failed predictions remain
real accuracy evidence and visible diagnostics. Native prefix frames are never
counted as AI passes. This protocol qualification does not guarantee accuracy.

After handoff, both tracks advance from their own corrected velocities, passive
tracers and previous pressure. They share geometry, time, material and prescribed
forcing. Native advection, viscosity, full pressure operator and velocity correction
are reused. The native reference requires its original gates; raw AI pressures use
the float32 model target and the existing physical checks. Courant, finiteness,
gauge and impermeable-wall/flux violations stop the paired run. A failed pressure
equation check alone retains the AI diagnostic; no silent native fallback occurs.

Water is 1000 kg/m³ with dynamic viscosity 0.001 Pa·s, fixed step 1/60 s, stationary
sealed free-slip walls and no mass source. Seed changes direction and phase of
localized smooth directional/swirl/fluctuating acceleration. This is controlled
forcing, not a claim of resolved turbulence or new solid/free-surface physics.
Integer XYZ cell counts and one cells-per-metre value determine physical size.
Cells remain cubic. Budgets: at most 65,536 cells, 512 steps and 96 MiB of retained
numeric frame payload. Current training coverage is 5–16 cells per axis and 8 or
16 cells per metre; larger shapes are extrapolation.

Both panes share one orbit camera, selected timestamp, Z cut and numeric probe.
The AI pane selects pressure or signed `p_AI − p_reference` in Pa. A zero-centered
blue/white/red legend labels endpoints and zero; probes show exact matched values.
Pressure shares one range across both tracks. The default range uses the selected
frame's complete volume; whole-run and fixed ranges are available. Fixed colors
clamp at endpoints and retain exact probe values. All-zero fields have a defined
white legend. Drag orbits, Shift-drag pans, wheel zooms, and Frame view resets framing.

One worker owns the two numerical tracks, model child and reader threads. UI history
uses immutable `Arc` frames and a short mutex, released before rendering. Cancel
kills the child; worker termination follows child wait and reader joins. Close
cancels active work and waits before closing. A project-directory file lease
prevents concurrent writers. Each run retains scene, model, frames and completion;
atomic `latest.json` / `activity.json` publications support external observation.
This is native interactive inference, not a managed training Run.

Focused tests cover previous solved pressure at handoff, no inference during
warm-up, independent tracks despite bad AI, cancellation, closed/bounded replies,
cubic geometry, signed differences, zero scales and picking. Existing desktop
camera/worker regressions also pass. The current frozen checkpoint still fails
pressure qualification after a valid warm-up; this UI exposes the result.

Source ownership: `desktop/ai_comparison` owns the comparison; `desktop/camera.rs`
owns shared navigation; the ordinary viewport reexports that same camera. Core
physics and provider acceptance are unchanged. Integration work is retained on
`feat/live-ai-pressure-comparison`, based on `origin/main` 43ed26b. Its owner is the
local surrogate-lab integration task; consumers are the active desktop binary and
lab project references. The checkout stays pinned while it serves that UI. Sync
requires stopping the app, reviewing upstream changes and repeating affected
checks. Retire the worktree after accepted integration and migration of the active
binary/project source pointers. No push, merge or original dirty checkout update
was performed by this slice.

## Real checkpoint lifecycle verification

The explicit environment test uses a separate fresh observation directory and the
actual measured CPU checkpoint. It waits for two AI predictions after warm-up,
cancels the resident child, checks joined terminal state and retained frames, then
restarts and completes a new paired run in the same project directory. It does not
send input to an active desktop window.

```bash
RHEON_AI_TEST_DEPLOYMENT=/path/to/deployment.json \
RHEON_AI_TEST_OUTPUT=/path/to/FRESH-lifecycle-evidence \
cargo test --offline --locked --features desktop --bin rheon-ai-compare \
  real_cpu_deployment_cancellation_drains_child_and_allows_restart -- --ignored --nocapture
```

The local measured deployment passed this test: cancellation retained at least
18 frames, terminal state was `cancelled`, the child slot was drained, and restart
completed 16 steps with a separate retained run and another drained child slot.
