# Live desktop viewer

Run `cargo run --locked --release --features desktop --bin rheon-desktop`.
The desktop keeps the existing CPU solver and egui/eframe renderer. It displays
perspective voxel concentration surfaces, the rotating emitter, the unit box
and sampled worker positions. Surface threshold changes only the preview.

The default material scene uses water, a 1 m cube, initial vacuum and Earth
gravity. Domain, Fluid material, Emitter, Simulation, CPU workers, Viewport,
Diagnostics and Results have independently collapsible sections. Material controls
run the bounded particle preview; Simulation can select the original carrier
mode for pressure-solver comparisons. Source sizes and positions are in metres.
See [physical units and material model limits](../../../docs/material-fluids.md).

## Navigation

| Input over the 3D view | Action |
| --- | --- |
| Left drag or middle drag | Orbit |
| Shift + left/middle drag | Pan |
| Ctrl + left/middle drag | Zoom |
| Mouse wheel | Zoom |
| Native two-finger scroll | Orbit |
| Shift + two-finger scroll | Pan |
| Pinch or Ctrl + scroll | Zoom |
| 1 / 3 / 7 | Front / right / top |
| 5 | Perspective / orthographic |
| Home or F | Frame scene |

Alt is not required, avoiding Linux window-manager drag conflicts. Enable
**Touchpad scroll / two-finger orbit** if the OS reports touchpad scrolling as
ordinary wheel events. Native pinch/rotate gestures work when supplied by the OS.
Mouse navigation retains Blender's middle-drag and Shift-pan conventions;
left-drag offers the same controls for touchpads without a middle button.

## Ownership and live results

The desktop owns one joinable worker, an immutable run request, and a scoped
Rayon pool. That worker exclusively owns numerical state. It captures step zero
and each successfully committed step before beginning the next. `history.rs`
publishes immutable `Arc<PreviewFrame>` values under a short mutex; painting
does not hold that lock or borrow simulation fields. Cancellation/failure leaves
accepted frames available. Close cancels and waits for the worker; Drop joins
as a backstop. New runs replace the previous timeline after worker startup succeeds.

History defaults to a 96 MiB preview-payload and metadata budget; Simulation
allows selecting up to 4,096 MiB. Material preview requests support up to 512³
eight-bit cells. Carrier previews max-pool concentration over source cells;
material previews splat physical parcels. The sidebar reports actual stored
resolution and planned bytes before starting. Long runs reduce resolution to
fit the selected budget while retaining every accepted step. This is a visualization rather
than a retained full-resolution numerical checkpoint. Simulation budgets,
allocator overhead, thread stacks, renderer meshes and process RSS are separate.

`viewport.rs` owns only camera, display settings and cached surface geometry.
Scrubbing holds the selected accepted step while new steps arrive; Follow live
returns to the newest step and current solver track. Playback uses recorded
simulation timestamps and an adjustable speed. Solver comparisons retain
separate tracks because their adaptive steps may reach different times.

Each frame records wall time around the actual simulation step, excluding
preview capture, rendering and worker startup. The active step also displays
elapsed calculation time. Summary compute time sums accepted step durations.

## Worker boxes

The optional `work-observation` feature enables `CalculationObserver`. Scoped
thread-local bindings connect real kernel visits to one cache-aligned atomic
marker per Rayon worker. Markers include actual voxel coordinate, stage and
active/idle state; they never alter numerical fields. Face visits map to their
adjacent bounded cell. The UI samples these markers at its repaint rate.
Bright orange boxes show active workers; dim boxes retain their last visited
cell while idle. Unused workers have no invented coordinate. Boxes appear only
in Follow live during a run, keeping historical frames free of current markers.

An eight-worker pool supplies eight marker slots, but serial reductions, SGS
dependencies, small grids and scheduling can leave some workers idle. Multiple
workers can also visit the same cell. The display cannot show every individual
visit between screen refreshes. Disable **Orange worker voxel boxes** before
starting when measuring performance without observer overhead. The no-observation
core compiles these probes away.

## Work coverage and batching

Blue voxels visualize appearance concentration in carrier fluid; clear cells
still belong to the pressure domain. Tracer interpolation skips provably zero
regions using exact donor support, expanded by velocity-bounded displacement
and interpolation support. It does not threshold small concentrations or crop
the pressure boundary to the display. The accepted frame records its actual
nonzero tracer-cell count, before preview quantization, and the sidebar reports
that count alongside the full carrier-domain explanation.

Each run also owns `CpuScheduler`, which measures whole joined passes and
learns among bounded batch sizes based on array length and available workers.
It periodically retunes; Rayon's existing scheduler handles work stealing.
SGS separately calibrates 4³ versus 8³ tiles using identical scratch-only
sweeps and schedules only the tiles ready in each dependency wave.
**Adaptive worker batches** shows the last completed plan for each kernel,
including SGS sweep size. Job counts span the whole pass and do not imply
that all SGS jobs can run simultaneously. Cumulative wall times and their
shares cover recorded kernels, including serial reductions; validation,
preview capture and other unrecorded work are excluded from those shares.
Stats remain inspectable after
completion and are replaced when a new run starts. Timing feedback and markers
are run-owned and do not modify numerical state.

See [scheduling ownership and measurements](../../../evidence/scheduling/README.md).

## Cylinder emitter

`Simulation::step_with_emitter` prescribes outward momentum and appearance
concentration at the two end disks. The default cylinder lies horizontally,
rotates about world Y at one revolution per simulated second, and has independent
end pressure heads of 0.12 and 0.08 Pa. The radial profile falls smoothly to zero
at the rim and is stronger on the upper side. Acceleration derives from pressure
head divided by density and nozzle depth; the normal pressure projection still
enforces the carrier's fixed-box divergence gates.

The pose is sampled at each step's midpoint. Rotation follows accepted
simulation time, and the timestep is capped to resolve at least eight intervals
per revolution, in addition to the existing CFL limit. Cancellation or rejection
publishes neither candidate fields nor emitter time. The current cylinder is a
source visualization, not a solid collision boundary. Appearance concentration
is not a conservative liquid-volume field; solid interaction and free-surface
coupling require separate solver work.

## Verification

Desktop tests exercise the actual egui input path for primary-drag orbit/pan/zoom
without Alt and native touchpad scroll/pinch. A gated real worker proves that
accepted frames appear before join and scrubbing survives background work.
History tests check adaptive long-run budgets and ordered publication.
Emitter contracts check vertical-axis rotation, both projected jets, bounded
concentration, invalid-input/cancellation rollback and retry. Observation tests
check actual worker coordinates, idle cleanup, thread-local ownership and
bitwise equality with unobserved simulation. Existing numerical and parallel
contracts remain applicable to both preserved pressure implementations.

Local verification on 2026-10-12: 361 tests passed in the full release desktop
suite, including the material pressure-energy and floor-rebound regressions,
particle thread-count replay and real 256³ preview capture. Focused core-only
pressure-energy and floor-rebound regressions also passed.
Desktop and core-only all-target Clippy passed with warnings denied, formatting
and diff checks passed, and both release binaries built.
Native Linux interaction verified primary-drag orbit, Shift-pan and Ctrl-zoom
without moving the window, plus live accepted geometry, worker markers and
per-step timing in the default 180-step demo.

## Quantitative obstacle projection lab

Select **Obstacle projection lab** above the viewport for a separate accepted
stationary obstacle projection: native pressure slices in Pa, reconstructed
speed and scaled velocity arrows, masked solid cells, and click probes. This
small bounded operator experiment uses actual solver output; it is independent
of the particle scene and live timeline. The lab has its own camera and framing.
See [model, acceptance criteria and remaining CAD milestones](../../../docs/implementation/cad-obstacle-lab.md).

## Quantitative views and interchange

The top tabs separate **Appearance** timeline playback, **Particles** latest raw
material state, the fixed **Obstacle lab**, and a live **Transport session** using
the public API. Settings and Inspector share a scrollable left dock. Particle
speed colors use the full latest-state range; click a displayed point to select
its run-local ID in Inspector. At most 8192 points are drawn, with deterministic
sampling; the inspector and export retain all points. Arrow length is velocity
multiplied by the displayed time scale. No pre-integration particle pressure is
colored onto later accepted positions.

Transport Run/Pause/Advance controls step a bounded 64-cell API session. Fraction,
speed and accepted pressure slices/probes come from that publication. Constructor
pressure is unavailable. The display uses physical origin/spacing and preserves
aspect ratio; **Frame view** and **Orthographic** adjust only the camera. This is a
separate full-box transport case, not an obstacle/free-surface or SPH coupling.

**VTK grid export** saves the current transport snapshot as `.vti`; **VTK particle
export** in Inspector saves latest accepted positions as `.vtp`. Enter a new file
path; existing files are never overwritten. Files default to `.generated/` and
are not repository source. Exports are synchronous and report success only after
flush; particle capture may pause while its coherent latest state is written.
See [VTK contract](../../../docs/api/vtk-interchange.md) for fields and stages.
