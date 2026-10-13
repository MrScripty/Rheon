# Emitter controls, version 2

`RotatingCylinderEmitter` is shared by the fixed-box carrier, particle material
simulation and native desktop request. `EMITTER_CONTROLS_VERSION` is 2;
`MaterialSessionCapabilities` exposes that version and API version 2. Existing
`..Default::default()` consumers keep pressure-head mode with stationary geometry
and zero perturbation. Exhaustive struct literals need the new fields. No dense3D
wire format, network service or independently validated turbulence solver is added.

## Velocity and geometry

- `emitted_velocity: None`: preserve pressure-head mode. Heads are **Pa**; nominal
  relative particle speed is `sqrt(2 * head / density)` at each end.
- `Some([vx, vy, vz])`: positive-end relative peak velocity, **m/s**, at simulated
  time zero. Its norm is speed and its direction is the cylinder axis. The negative
  end is opposed with equal speed. `[0; 3]` disables both jets, even with motion or
  perturbation configured. Pressure heads are ignored in this mode but remain valid
  nonnegative settings for switching back.
- `rotations_per_second`: rotates that initial axis about world Y using the existing
  convention (+X turns toward +Z). `axis_at` and `basis_at` expose the current frame.
- `motion_velocity`: separate world velocity, **m/s**. `center_at(t)` returns
  `center + t * motion_velocity`, referenced to absolute simulated time zero.
  New particles inherit this velocity; `peak_velocity_at` includes it. Editing
  motion mid-run therefore changes the current location. To keep that location,
  rebase `center = current_location - current_time * new_motion_velocity`.
- Centre, radius, half-length and nozzle depth are **metres**. Geometry is a source,
  not a collider. Conservative whole-rotation X/Z bounds and tilted Y bounds must
  stay inside the box throughout the accepted interval. Motion beyond that supported
  domain returns `InvalidEmitter`; it does not wrap, clip or move accepted state.

Both end disks retain their nonuniform profile. Particle velocity at birth is
`motion + sign * axis * speed * sqrt(profile) + perturbation * sqrt(profile)`.
Nominal volume remains `pi * radius^2 * speed * 0.43 * dt` per end; retained volume
credits make capacity and acceptance transactional. Noise never determines mass.

The carrier's vector mode relaxes faces toward that inlet target with blend
`1 - exp(-dt * speed / nozzle_depth)` before the existing pressure projection.
The pressure mode retains its original acceleration when new controls are zero;
with motion/noise it adds a profile-weighted reservoir forcing. These are prescribed
source models: the vector is neither a post-projection velocity guarantee nor a
new open-boundary, mass-conserving carrier inlet. The carrier transports appearance
concentration; the particle path emits material mass.

## Seeded perturbations

`EmitterTurbulence` holds a `u64` seed, `strength` in **m/s**, `spatial_scale` in
**metres**, and `time_scale` in **seconds**. Strength bounds the perturbation vector
norm before profile attenuation. Zero strength adds exactly zero. The smooth 4D
value-noise lattice is attached to `(axis, disk-up, disk-across)` coordinates and
absolute simulation time; adjacent time planes are smoothly interpolated.
`sample(local_position, time)` lets consumers inspect the authored field.

The seed and coordinates completely determine a sample on the same numerical
platform. Calls and rejected attempts consume no random state. Solver sources
sample at step midpoint. A changed timestep partition can change the trajectory;
this is not a claim of trajectory invariance or cross-platform bitwise replay.
The field is an authored inlet disturbance, **not validated turbulent physics**,
a turbulence closure, measured spectrum, eddy viscosity or energy-conserving model.

Finite velocity components and strength are bounded to 1e6 m/s; strength is
nonnegative. Scales must be positive normal floating-point values. Sampling beyond
±2^40 lattice coordinates is refused to avoid unresolved coordinate aliasing.
Nonzero noise caps the accepted timestep at one quarter of its time scale. Motion
also caps travel to one quarter of carrier minimum cell spacing or particle support
length. Existing flow, acoustic, viscosity, capillary and rotation limits remain.
These are numerical bounds, not real-time guarantees or physical calibration.

## Recording and desktop

`MaterialStepControls.emitter` records the whole emitter, caller source ID and
source version. `MaterialSession::snapshot().accepted_controls()` retains only the
last accepted controls; cancellation or invalid input cannot replace them. Store
these values with the advertised emitter version, source revision, material,
initial state and accepted timestep sequence for replay. Desktop frozen run
requests retain the same SI values. This does not introduce a persistent emitter
archive or add controls to dense3D data.

The native panel exposes velocity XYZ and direction/speed, independent motion,
and seed/strength/scales. The viewport shows jet and motion directions with fixed
illustrative arrow lengths, not velocity magnitude; disabled jets have no active
jet arrow. Direction editing preserves the last nonzero direction at zero speed.
The headless [example](../examples/emitter_controls.rs) prints accepted controls
and particle output without a renderer.

## Evidence and boundaries

See the [execution ledger](plans/emitter-controls/execution-ledger.md) for exact
commands and results. The pinned-base comparison covers 12 carrier updates and
4,000 water and air updates; its byte equality is evidence for those cases, not a
proof across all inputs. Automated native checks use egui without a window.
Interactive input, screenshots, GPU/display behavior and hardware timing are
untested. No collision algorithm or CPU worker scheduling changed.
