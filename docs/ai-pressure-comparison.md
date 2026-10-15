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

Source ownership: `desktop/ai_comparison` retains the numerical/process/worker/UI owners; shared camera navigation remains in `desktop/camera.rs`. Integration selects original comparison/lifecycle commits onto `implementation/configurable-pressure-m1-20261015` after PR84; unrelated training-viewer/voxel-renderer work is excluded. The public [pressure context and CPU adapter contract](api/pressure-context.md) describes inert support checking, named physical ports, full/correction outputs and the legacy deployment path. No Session acceptance or SQLite schema is changed.

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

The producer-retained measured deployment observations report this test passed: cancellation retained at least
18 frames, terminal state was `cancelled`, the child slot was drained, and restart
completed 16 steps with a separate retained run and another drained child slot.

Those retained observations were hash-audited during M0, not rerun in this environment. The actual deployment and Torch are absent here. New stdlib CPU fixture tests freshly exercise cancellation after predictions, immediate-child drain and restart; they do not qualify the trained checkpoint. Headless egui checks do not establish native-window behavior.

A later isolated Xvfb software-X11 workflow ran the actual native window with a
stdlib analytic CPU package. It rendered inert support and failed diagnostic
pressure, completed 64 steps, cancelled after predictions, restarted to 32-step
completion and closed during a further prediction with owned child termination.
This supplements headless tests; it does not qualify the absent measured trained
bundle or a physical GPU. Evidence subject/digests are recorded in the plan ledger.
