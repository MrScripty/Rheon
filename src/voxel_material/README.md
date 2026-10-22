# Filled two-material voxel components

This module owns captured liquid/gas reference density and viscosity, a validated
cell-centered fraction lattice, and a binary64 MAC projection component. Gas is
occupied fluid with positive mass. It supplies no vacuum, gas EOS, heat, phase
change or surface-tension model. The complete coupled timestep and native mode
remain gated by the [implementation plan](../../docs/plans/voxel-multimaterial-initial-authoring/plan.md).

Start with `MaterialPair::water_air`, `VoxelMaterialState::new` and
`MaterialProjectionWorkspace::new`. The numerical coefficient constructor also
serves bounded independent fixtures; it does not qualify arbitrary fluids for
the native product. Native initial authoring will admit exact binary Water/Air
IDs; mixed constructor fractions are numerical inputs, not an authoring brush.

`VoxelMaterialState` starts at rest with no accepted pressure. Its immutable
inspection API cannot install a pressure projection as an accepted timestep.
`MaterialProjectionWorkspace::project` borrows that state and a predictor,
returns a success-only scratch view, and leaves the state unchanged on success,
failure or cancellation. Reusing the workspace invalidates the previous borrow.
The caller must not treat a projected velocity as a coupled material evolution.

Projection uses positive half-cell face densities, the existing Jacobi PCG,
full sealed-domain compatibility/gauge checking, and a separate direct stored
velocity divergence gate. Surface/column f64 operations refuse explicitly.
Typed failures identify shapes, fractions, materials, geometry, numerical
arithmetic, limits, solver failure or cancellation. Scratch can be overwritten
on a failed operation and is not authoritative state.

The state/workspace constructors check aggregate managed-array payload before
allocation and actual Vec capacities afterward, including the existing PCG's
six f64 cell buffers. The cap excludes caller-owned predictor/input bytes,
allocator bookkeeping, renderer/history/recording and process RSS. Those owners
must charge their own memory when composing the complete mode. Construction
supports counts4–64 per axis; it does not promise an allocation or runtime bound
from counts alone. There is no performance claim.

`tests/voxel_material_pressure_contract.rs` contains independent graph and
half-column pressure oracles plus validation/state-preservation cases. Existing
operator/pressure tests protect legacy scalar and f32 contracts. M1 component
evidence does not satisfy coupled transport, native editing, persistence or
end-to-end acceptance.
