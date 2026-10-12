# Floor rebound and preview resolution

The user reported default water in a 1 m vacuum domain jumping to the ceiling
after the first floor layer filled. The reproduction uses the real material
step, default Earth gravity, 16 particle samples per side, 1 rev/s rotation,
1,000/800 Pa end heads and continuous emission. `material_bounce_lab` is retained
as a reproduction lab, not a performance benchmark:

```sh
cargo run --locked --release --example material_bounce_lab -- 12000 100 water
cargo test --locked --release --test material_contract a_filling_water_floor
cargo test --locked --release --lib pressure_acceleration_matches
```

The old run reached 0.950421 m at step 6,318 (6.304143 simulated seconds), with
peak upward velocity 4.962247 m/s and pressure 57,757 Pa. Turning the source off
at 2 s avoided the failure; continued accumulation was required. The same
continuous scenario with corrected pressure completed 12,000 steps, retaining
a maximum height of 0.582657 m over 12.334643 simulated seconds. There is no
artificial height or speed cap.

Density used poly6, while pressure used spiky gradients and a coefficient that
did not differentiate the reference-volume compression energy. In the isolated
eight-parcel compression fixture, the old X acceleration was -2,627.5616 m/s²;
an independent finite difference of compression energy required -369.7513 m/s².
The correction uses poly6 gradients and the reference-density factors from
`rho_i = rho0_i sum((mass_j/rho0_j) W_ij)`. Equal/opposite pair forces and unlike
material inertia are retained. Physical viscosity, source heads, gravity,
sampling, surface-cohesion settings and timestep limits were not changed to
obtain the passing reproduction.

Both regression tests were also run after temporarily restoring the old force:
the energy-gradient fixture failed with the values above, and the floor fixture
failed at step 6,252 (6.245465 s) as a parcel crossed its 0.8 m regression bound.
The corrected source was restored and both pass. The height bound applies only
to this controlled reproduction; it is not a general claim that physical
splashes can never rise above an emitter.

Raw reproduction output:
`/tmp/rheon-bounce-continuous-before.log`,
`/tmp/rheon-bounce-conservative.log`,
`/tmp/rheon-bounce-fixed-final.log`,
`/tmp/rheon-pressure-gradient-red.log`,
`/tmp/rheon-floor-rebound-red.log` and
`/tmp/rheon-rebound-preview-tests.log`.

The old preview control stopped at 128³, and the history imposed an additional
hidden 32³ ceiling. Material requests now reach 512³. The history budget,
defaulting to 96 MiB, can be raised to 4,096 MiB; planning reports actual stored
resolution and bytes before startup. A real observed worker captures 256³
frames in a two-frame history under the default budget. Longer histories still
adapt resolution rather than exceed the selected budget. The viewport reports
the stored frame's physical voxel size. Physical detail belongs to the separate
particle-sampling control, now permitting 64 per side. The original carrier
solver retains its 128³ grid admission and existing numerical-array budget.

The routed Standards MCP guidance remains the authority for implementation,
contracts, independent oracles and verification. The diagnostic skill's loop
was used to obtain a red-capable reproduction before testing ranked pressure,
boundary and timestep hypotheses. No production debug logging was added.

Verification: 361 full release desktop tests passed, including the original
carrier contracts, both pressure/height regressions, exact particle thread-count
replay and real 256³ preview capture. Core-only pressure/height regressions and
desktop/core all-target Clippy passed; both binaries built, formatting and diff
checks passed.
