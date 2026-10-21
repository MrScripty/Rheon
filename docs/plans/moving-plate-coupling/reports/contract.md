# Moving-plate consumer contract (experimental)

Path-import experiments/plate_coupling.rs, as in the native example. Borrow an
existing StaticObstacleShear; choose one tangent axis different from its normal,
two Plate modes (Prescribed or Dynamic with positive mass_kg), and a combined
payload cap. Construction owns only scratch and a previous accepted report.
The cap includes borrowed shear storage and new vector capacities, excludes
geometry, caller velocity arrays, stack, allocator metadata and RSS.

step receives mutable fluid layer speeds and two mutable wall speeds in m/s,
a positive dt in s and a cancellation callback. Both caller arrays and the
previous report publish only after all solve, interval and cancellation gates.
Prescribed speeds stay fixed for this requested step; dynamic speeds evolve.
There is no time/generation owner: an API session must attach those atomically.
The same immutable geometry and plate modes apply throughout a workspace's life.

Report carries dt, tangent axis, boundary impulses onto fluid (N s), prescribed
wall work (J), combined kinetic energies/increment energy, dissipation (J),
momentum/energy defect enclosures and budgets, and worst node equation defect.
Use the plate impulse as the negative of the corresponding fluid impulse.
Dynamic work is internal; prescribed actuator work is external. Normal pressure,
loads/torque and arbitrary rigid-body motion are unavailable, not zero reports.

Shapes: one cross-section-spanning lower box and connected fluid extrusion as
admitted by StaticObstacleShear, including represented partial first layer.
The lower wall uses that owner's no-slip/Navier closure; upper is no-slip.
Both tangent directions are periodic/invariant. Tangential sliding leaves the
normal geometry invariant. No finite-edge geometry, end walls, contact, wall
position, fluid advection, free interface, variable material, GPU or engineering
accuracy promise. Production Simulation and public API are unchanged.

All scalar inputs must be finite normal values or zero. Positive masses/dt are
required. Arithmetic uncertainty, unsupported platform, singular/overflowed
proposal, invalid shape, insufficient cap and cancellation are typed failures;
no hidden retry, tolerance relaxation or partial publication occurs. The reused
interval backend assumes Linux x86_64 default round-to-nearest binary64 controls.

## Run and independently check

```sh
cargo test --locked --no-default-features --test plate_coupling_contract
cargo build --locked --no-default-features --example plate_coupling
python3 tools/check_plate_coupling.py --native "$CARGO_TARGET_DIR/debug/examples/plate_coupling" --output /tmp/plate-new-evidence
```

Use target/debug/examples/plate_coupling when CARGO_TARGET_DIR is unset. The
example emits JSON to stdout. The checker requires a new external directory and
clean source by default; it records exact source/binary identities and evaluates
real fields with independent dense rational equations. No rendered preview or
historical receipt is treated as physical evidence.
