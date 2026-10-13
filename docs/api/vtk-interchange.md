# Native VTK interchange, version1

`write_session_vti(out, &SessionSnapshot)` writes full-box `ImageData`.
`write_material_vtp(out, &MaterialSnapshot)` writes canonical material snapshots;
`write_particles_vtp` accepts the existing inspector's coherent retained raw data.
These streaming writers borrow source data and return `io::Result<()>`. Validation
precedes output; I/O failure can leave a partial stream. The caller owns final
flush/file lifecycle. The desktop creates new files exclusively and removes its
incomplete file on failure. ASCII encoding preserves native float precision.

VTI origin, spacing and extents preserve physical coordinates and x-fast order.
CellData contains Float64 `fraction`, Float32 `tracer`, and Float64
`velocity_cell_m_s` (adjacent native face average). `interval_pressure_pa` is
absent at construction; after acceptance it is the projection interval pressure,
with cell0 gauge and explicit interval metadata. It is not a free-surface or
absolute-pressure measurement. Native Float32 `velocity_x/y/z` arrays remain in
FieldData with dimensions and stagger offsets. Fraction is not a validity mask.

VTP Points and PointData contain Float64 accepted positions, velocity, mass,
temperature, material and UInt64 run-local append-only particle IDs. Diagnostic
density, pressure and acceleration are separate **FieldData**, associated by
particle index with explicit pre-integration evaluation time. They are not
PointData at the later accepted positions. Missing initial diagnostics remain
absent. Constructor/accepted publication stages are explicit.

Both carry `rheon_vtk_contract_version`, run ID, generation, time and caller-declared
source identity. Text metadata uses `*_utf8` UInt8 arrays; decode bytes as UTF-8
without a trailing NUL. Source identity is not authenticated provenance. Units and
pressure semantics are encoded. No replay, restore or stable cross-run particle
identity is promised.

Independent verification uses VTK9.3.1, not a second custom XML parser:

```sh
cargo test --locked --no-default-features --test vtk_export_contract
cargo run --locked --no-default-features --example vtk_export_fixture -- /tmp/rheon-vtk
python3 tools/test_vtk_roundtrip.py /tmp/rheon-vtk
```

The pinned CI reader checks seven authored/real-session files, asymmetric physical
coordinates, exact native float bits (including signed zero), UInt64 identities,
field associations, Unicode, raw faces and reconstruction, missing constructor
pressure, and separately staged real material diagnostics. Fixtures explicitly
mark synthetic interchange values; desktop views never use fixture data.
