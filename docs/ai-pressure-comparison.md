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

Current package schema `rheon-pressure-adapter-v1` binds measured interpreter,
entrypoint and inventoried model assets. The lab owns packaging and exact
preprocessing/reconstruction; Rheon owns bounded process I/O and lifecycle.
No artifact hashes or machine-specific Python paths are compiled into Rheon.
The bridge uses bounded, closed, correlated newline-delimited JSON. The handshake
requires CPU execution and zero optimizer updates. GPU training happens upstream.
Older deployment descriptors are offline conversion inputs, rejected by normal
execution.

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
Cells remain cubic. Density selection is 1–1000 cells/m, independently of the
comparison owner's 5–128 cells/axis, 65536 total cells, 512 steps and 96 MiB retained
numeric-frame payload. These model-comparison count limits are narrower than
the general native application's 10000-axis setting. CLI `--counts X,Y,Z`,
`--cells-per-metre N`, `--steps N`, `--warmup-steps N` and `--preflight` expose
the same Scene contract; preflight allocates no numerical fields and executes
no model. Current training coverage remains 5–16 cells/axis and 8 or 16 cells/m.
Selecting a new density does not qualify a model: dynamic adapters must declare
the actual spacing/counts/material/dt/context; unsupported contexts are rejected
inertly before a child starts. The measured
A1 package remains unavailable. See [density contracts](api/domain-density.md).

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

Source ownership: `src/diagnostic` retains numerical advancement, scene and
history; desktop aliases that same owner. `desktop/ai_comparison` owns package
validation, model process, worker and UI. Shared camera navigation remains in
`desktop/camera.rs`. The public [pressure context and CPU adapter
contract](api/pressure-context.md) describes named physical ports and
full/correction outputs. Accepted Session publication and SQLite schema are
unchanged.

## Real checkpoint lifecycle verification

The explicit environment test uses a separate fresh observation directory and the
actual measured CPU checkpoint. It waits for two AI predictions after warm-up,
cancels the resident child, checks joined terminal state and retained frames, then
restarts and completes a new paired run in the same project directory. It does not
send input to an active desktop window.

```bash
RHEON_AI_TEST_PACKAGE=/path/to/adapter.json \
RHEON_AI_TEST_OUTPUT=/path/to/FRESH-lifecycle-evidence \
cargo test --offline --locked --features desktop --bin rheon-ai-compare \
  trained_current_package_cancellation_drains_child_and_allows_restart -- --ignored --nocapture
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

## Current package and public diagnostic boundary

Current execution accepts only `rheon-pressure-adapter-v1` packages. Convert
older deployments offline; the ordinary process owner does not decode them.
The measured package inventories unchanged weights, normalization, entrypoint and
interpreter. Its entrypoint receives `--deployment` with the current manifest
path. The CPU owner checks the correlated `adapter_ready` handshake and each
`candidate` response. A full-pressure package owns its complete reconstruction;
a pressure-correction package receives exactly one prior addition in the host.

The `diagnostic` Cargo feature exposes `rheon::diagnostic::DiagnosticSession`,
`Scene`, typed `Input`/`PredictionFailure`, frames and advance outcomes without
eframe. Desktop aliases these same numerical owners. The lab calls
`with_policy(scene, IndependentStopOnReject, run_id)` and `advance_typed`;
`Input.context` supplies validated physical geometry, RHS and actual solved
history. Every typed result retains its raw attempt and same-request native
reference. Rejection stops before history advancement and performs zero native
fallback/refinement. Configured accepted Sessions retain their separate visible
native-fallback contract.

The fixed-step clock uses `step_index * DT` throughout force sampling, frames
and typed request endpoints. `PressureContext::with_interval_end` retains that
explicit endpoint, allowing at most one ULP of ordinary binary rounding from
`start + DT`; dt stays unchanged and prior-end/next-start alignment stays exact.
The diagnostic archive format 3 persists and hashes the explicit endpoint.
Accepted Sessions with adaptive dt retain their existing clock authority. A 32-step focused comparison checks exact
pressure, velocity and tracer values against the original numerical policy,
including 15 actual native steps before the first prediction at step 16. This
fixture does not establish trained-model parity or the 74-scene acceptance gate.

The real current-package lifecycle selector is
`trained_current_package_cancellation_drains_child_and_allows_restart`, enabled
with `--ignored --nocapture`, `RHEON_AI_TEST_PACKAGE` naming the measured current
manifest and `RHEON_AI_TEST_OUTPUT` naming a fresh evidence directory. It copies
measured package bytes into that owned evidence directory so cold inspection can
hide its package without changing a shared package. It checks
consumed predictions, cancellation/reaping, a fresh run identity, and exactly one
AI prediction after 15 native steps in the 16-step restart. A third typed
16-step run retains the known raw rejection in schema 3, then hides the owned
package copy and verifies the saved attempt through current cold readers.
Model accuracy remains
an independent observed result.
