# CAD visualization milestone: bounded obstacle projection lab

Goal: quantitative engineering views that faithfully display accepted numerical
fields, with slices, probes, vectors, pressure references, physical geometry and
explicit model limits. Native persistent-goal tooling was not available; this
file retains acceptance and follow-on milestones.

Base: main `2209012bc5f902693d0f695ec1c2d854126af317`.
Research: [PR43](https://github.com/MrScripty/Rheon/pull/43), especially
`cad-visualization-contracts.md` sections 2–3 and the carrier portion of stage 1.
No AGENTS.md, repository .agents/skills or separate coding-standard file was
present on this base or in the selected workspace. Follow existing Rust/egui
ownership, locked dependencies, formatting, Clippy and contract-test conventions.
`docs/REPOSITORY_OUTPUTS.md` keeps generated screenshots and logs out of commits;
this milestone follows that policy.

## Delivered scope and acceptance

Open **Obstacle projection lab** above the native viewport. It computes one
bounded real `StaticObstaclePressure::project` result on a 12×8×6 grid with
0.125 m spacing, in a sealed 1.5×1.0×0.75 m box. The indexed triangle obstacle spans
[0.5,0.25,0.25] to [1.0,0.75,0.5] m. Geometry and pressure assembly share the same
`StaticObstacleGeometry` owner. The prescribed initial x-face pulse is
sin(π y / 1.0) m/s on open interior faces, with all other speeds initially zero.
Density is 1 kg/m³; projection interval is 0.05 s. This is an operator experiment,
not an advancing obstacle-flow, viscous/no-slip, turbulent-wake or CAD-load solver.

The snapshot is published only after existing pressure residual, divergence and
energy acceptance gates pass. A failed solve displays an error, never a fallback
field. Speed is the norm of adjacent-face-averaged corrected velocity, not a
native flux. Pressure is the solver's pressure correction in Pa; one explicit
cell per fluid component has zero gauge pressure. It is not absolute pressure
or particle pressure. Solid cells have unavailable values, not measured zero.

- XY, XZ and YZ native cell slices preserve physical aspect ratio under a common
  camera scale. Each tile is piecewise constant, with no interpolation across
  the obstacle. The view is depth-sorted CPU geometry in the existing renderer. The lab has
  independent camera state, closer default framing, axis labels and orthographic
  control; live camera settings survive switching views.
- Click probes report exact retained binary64 cell position, reconstructed
  velocity and pressure (display rounded); masked cells say unavailable.
- Full-state, valid-cell scalar extrema drive both colors and numeric legends.
  The range does not change when moving a slice. Constant ranges are defined.
- White velocity arrows (explicit overlay drawn after scalar tiles) use a stated displacement scale of 0.1 s, with no
  normalization. Arrow projection shows foreshortening; these are not streamlines.
- Orange obstacle and blue domain edges are explicitly wire overlays. Solid
  cells on the selected slice show the computational mask. Wires are not an
  opaque CAD surface or a hidden-surface rendering claim.
- Camera manipulation changes presentation only. The immutable lab result is
  separate from all live tracks and their appearance-byte history. The label
  explicitly states that independence; live scrub times are never applied to it.
- Fixed small geometry bounds the lab's work. Construction occurs once on first
  selection and is retained; no new thread, renderer dependency or solver API
  is introduced. Larger asynchronous analysis belongs to a later milestone.

Acceptance checks: actual accepted solve, dry-cell exclusion, pressure gauge,
nontrivial corrected fields, anisotropic domain display scale, polygon probes,
constant legends, native UI interactions, screenshot review and regression tests.
Exact commands and results are recorded below after qualification.

## Shared ownership and remaining milestones

This worker owns `desktop/engineering.rs`, viewport integration and this record.
The only `rheon-desktop.rs` change declares the module. No history, API, library
exports, numerical solver, particle or settings files are changed. The parent
can integrate that two-line declaration with the quantitative inspection work.
The desktop README adds a discovery link to this milestone.

1. **Shared accepted snapshot adapter:** API owner defines solver/run/revision,
   geometry, accepted time/stage, units, associations and validity; inspection
   owner supplies actual retained fields and budget accounting. No implicit
   conversion of appearance voxels into quantitative data is permissible.
2. **Live particle points and inspection:** coordinate with inspection owner for
   accepted positions/velocities/materials and temperature-model qualification.
   Pressure requires synchronized diagnostic stage and separate liquid/gas
   reference semantics. Do not use report maxima as spatial fields.
3. **VTK exchange:** ownership is unconfirmed; no competing exporter is added.
   Coordinate with API/export owner for .vti/.vtp metadata and independent-reader
   asymmetric round-trip checks. This milestone remains open.
4. **General grid slice input:** replace the lab-local adapter with shared
   immutable field contracts after review. Cut-cell reconstruction needs its own
   rule; the current demonstration uses exactly representable aligned solid
   boundaries only and rejects partial-cell volumes. An initial decimal-spacing
   fixture was rejected by the existing numerical operator; no solver tolerance
   or acceptance gate was loosened to admit it.
5. **Further CAD/physics:** imported CAD, native flux sections, trajectories,
   pressure loads, viscous wall stress, particle reconstruction and GPU rendering
   are not implemented here. Preserve PR43's staged prerequisites and acceptance
   fixtures before exposing those quantities.

## Qualification record (2026-10-13)

- `cargo test --locked --features desktop --bin rheon-desktop -- --test-threads=1`:
  **25 passed**, isolated run, 9.61 s. Includes four engineering numerical/UI
  tests and the separate-camera interaction test. The new keyboard test first
  establishes egui hit regions before delivering hover-dependent Home input.
- `cargo test --locked --features desktop --test obstacle_flow_contract`: **16 passed**, including independent pressure/energy, component gauge, and anisotropic geometry contracts.
- `cargo clippy --locked --features desktop --all-targets -- -D warnings`:
  passed on the final code, as did `cargo fmt --all --check` and `git diff --check`.
- The broad `cargo test --locked --features desktop` debug run was interrupted
  during the pre-existing long-running extruded-third contract after earlier
  suites passed. **This is not a full numerical-suite pass.** Concurrent desktop
  runs hit the existing ten-second worker-lifecycle timeout; the isolated serial
  run above passed. No timing threshold was modified.
- Independent review found and fixed arrow overpainting by nearer scalar tiles.
  An egui shape-order test now requires every vector glyph after opaque tiles.
  Review also led to closer initial framing, a full-cell-volume guard, and a
  nonvacuous single-gauge test. The reviewer independently executed the four
  engineering tests and inspected speed/pressure screenshots.

Native Linux qualification used Xorg's dummy display and software OpenGL.
Inspected actual desktop screenshots at 1250×820 and 900×650, including field
selection, fluid-cell probe, masked obstacle, slice geometry, expanded numerical
inspector, and independent timeline scrubbing. Hardware GPUs, physical touchpads,
other operating systems, OS clipboard delivery, and accelerated-rendering
performance remain untested. No throughput or universal engineering accuracy
claim is made. Generated screenshots and logs are retained outside the repository.

### PR47 integration qualification

A detached temporary worktree at inspector commit `1923d5b` received this CAD
patch without modifying inspector source. Plain patch application reported one
conflict at the initial `rheon-desktop.rs` module block. Retaining both the
`engineering` and `inspector` declarations resolves it; inspector capture hooks,
window, diagnostics, conservation totals and CSV remain intact. This is a tested
local integration, not a merge to main or a dependency silently folded into this PR.

Forced rebuilding the combined worktree and running
`cargo test --manifest-path /workspace/rheon-cad-integration/Cargo.toml --locked --features desktop --bin rheon-desktop -- --test-threads=1`
passed **26 tests** in 8.80 s, explicitly including the inspector trace/CSV test.
`cargo build --manifest-path /workspace/rheon-cad-integration/Cargo.toml --locked --features desktop --bin rheon-desktop`
also passed. A shared Cargo target cache was forced to rebuild to ensure this
receipt includes the inspector rather than a standalone cached test executable.

In the actual combined app a 1,024-step material run completed with 87 particles.
The expanded inspector showed latest generation 1024 and 256 retained selected
samples, distinct accepted and diagnostic evaluation times, physical fields and
raw conservation totals. Scrubbing the appearance timeline to step 240 preserved
that latest inspector state and the separately labelled obstacle projection.
The floating inspector initially overlaps viewport controls; moving/collapsing
it restores access. At 900×650 its expanded content and copy button remain visible
when positioned at the top left. A shared dock layout is a future UX refinement.

Final independent screenshot review confirmed no blocking data/rendering issue.
The floating window overlaps roughly 66 pixels of the CAD control region in the
combined captures and clips part of the narrow footer; these are not overlap-free
captures. Move or collapse the inspector to read covered labels.
