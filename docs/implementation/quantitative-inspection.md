# Quantitative inspection checkpoint

Base: remote main `2209012`. Goal: preserve binary64 particle data and inspect
committed density, pressure and force evaluations without confusing voxel history
with recoverable state. No native persistent-goal facility is available in the
execution environment; this plan and the task plan retain the goal.

## Milestones and acceptance

1. Publish only committed diagnostics. Before a first accepted step they are
   unavailable. Rejected/cancelled steps preserve values. Charge retained storage
   to the particle simulation budget. Match reported maximum pressure and the
   actual pre-integration acceleration; verify equal/opposite pair force.
2. Add a separate numerical inspector, preserving visual history and renderer.
   Select particles by append-only run-local index. Show round-trip binary64
   quantities with units and stages. Keep one latest owned state and a 256-sample
   selected-particle trace; clear trace on selection/new run. Export CSV via
   clipboard, with blank unavailable fields and explicit evaluation time.
3. Run Rust tests, formatting, warning-denied Clippy and independent review.
   Push a dedicated feature branch and open a draft PR. No merge or deployment.

## Contracts and integration ownership

`ParticleFluidSimulation::particle_diagnostics()` returns an optional evaluation
time and borrowed `ParticleDiagnostic` slice, ordered exactly like `particles()`.
The stage is **post-emission, pre-integration**. Density is kg/m³, pressure Pa,
acceleration m/s² (including gravity and all implemented pair forces). Force is
mass times that acceleration; wall velocity clipping/drag occurs later and is
excluded. Accepted positions/velocities have the accepted time, not evaluation
time. Diagnostics describe the numerical preview model already implemented; they
are not validated real-fluid measurements. Initial state has no evaluated data.

IDs are indices within a particular simulation instance, stable because particles
are appended and never deleted or reordered. Recreating a simulation starts a new
namespace. Generation identifies a committed step, not an object lifetime.
The API owner can wrap this accessor in the shared snapshot facade; this change
does not introduce a competing snapshot API or modify the CAD renderer.

Shared integration edits: one `src/lib.rs` re-export, one `LiveHistory` inspector
store, initial/accepted-step worker capture calls, and an inspector window in
`rheon-desktop.rs`. Rendering and viewport modules are unchanged.

## Run the checkpoint

```
cargo run --locked --release --features desktop --bin rheon-desktop
```

Start a material run; expand **Numerical inspector**. Select an ID and expand
position/velocity or density/pressure/force readouts. The window always identifies
latest numerical state independently of the visual scrubber. Copy numerical
history CSV exports only the selected-particle samples actually captured, not a
reconstructed visual history. Selecting another ID clears the trace and seeds its current sample; new samples
arrive on accepted worker captures. A finished run can inspect all latest IDs but
cannot retrospectively recover their earlier states.

Raw mass, momentum and kinetic-energy totals are available under conservation
diagnostics. They are not closed-system residuals: emission, gravity, viscosity,
cohesion and wall response must be accounted for before asserting conservation.
No changes to the physics model or field solver are made here. Carrier field
inspection remains the responsibility of the common API/field snapshot work.

## Storage and boundaries

Committed diagnostics add 40 bytes per configured maximum particle (binary64
fields) within the solver's existing allocation budget. The desktop retains one
latest full numerical copy (particle plus optional diagnostic arrays) outside the
voxel history budget, plus at most 256 selected samples. Vec capacities/allocator
rounding, GUI resources and clipboard strings are outside the solver budget.
This is bounded by the configured maximum of 262,144 particles, not a numerical
archive per visual frame. Copying latest particle state is O(N) per capture;
no GPU, real-time throughput or whole-process memory cap is claimed.

## Review and validation evidence

Independent review identified an empty-export defect when selecting a new
particle after completion. Selection now seeds the trace with the latest sample;
the owned-history regression covers it. Review also requested emission alignment,
wall-stage, invalid-attempt and initial-empty-field coverage, now included.

Executed checks:

- `cargo test --release --locked --no-default-features`: all 326 tests passed,
  including the complete material/conservation contracts and new inspection test.

- `cargo test --locked --no-default-features --lib`: 14 passed, including
  commit consistency, cancellation, late arithmetic failure after scratch changes,
  invalid dt, pair force balance and wall-stage exclusion.
- `cargo test --locked --no-default-features --test particle_inspection_contract`:
  emission/ID alignment passed.
- `cargo test --locked --features desktop --bin rheon-desktop inspector::`:
  256-sample bound, owned lifetime, binary64 round trip, initial blank columns
  and completed-run selection/export passed.
- `cargo clippy --locked --features desktop --all-targets -- -D warnings` and
  `cargo fmt --all --check`: passed.
- Full desktop debug suite: 20 passed, one existing wall-clock lifecycle test
  timed out under concurrent CPU load. The same test also timed out on untouched
  `2209012` in a detached worktree. An isolated feature-branch run passed.
  This host does not qualify that test's ten-second wall-clock assumption.
- Broad debug numerical contracts were stopped in favor of optimized execution;
  they are not claimed as a completed debug-suite pass.

Native window presentation, OS clipboard delivery, GPU/driver behavior and
interactive performance on user hardware have not been tested. Headless egui
and worker tests do not establish those boundaries. No merge/deploy performed.
