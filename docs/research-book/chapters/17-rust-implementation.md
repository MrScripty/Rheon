# 17 A staged Rust implementation

Production should begin in Rust, with Zig and Odin following a stable behavioral contract. This research package is not that production solver. The checked standards snapshot dcc56f26e884ade260770beceba2501d3746200d emphasizes owned state, explicit failures and claim-matched evidence [S20].

## Core ownership

GridShape validates dimensions, spacing, origin, strides and capacity. BoundarySnapshot owns classification and prescribed data for one version. FluidState owns accepted fields and time. Workspace owns reusable scratch. Stepper produces a candidate and diagnostics. A separate renderer consumes an immutable accepted snapshot.

The numerical core should not depend on GUI, model weights or windowing. A demonstration application is one consumer. Physics configuration and presentation configuration remain separate.

## Geometry and pressure first

Implement checked indexing, shapes, coordinates and boundary classification. Test uneven dimensions, minimum grids, overflow, every orientation and linear pressure ramps. Compare independently assembled small matrices with production stencils. Verify adjointness, nullspace and connected-component compatibility.

Then add a typed pressure solver, initially diagonal-preconditioned CG on a gauge-fixed system if selected. Exercise zero residual, incompatible flux, isolated cells, disconnected domains, invalid coefficients and budget exhaustion. Compare direct small solves and hand calculations. Compute residual and divergence through separate paths.

Binary64 simplifies diagnosis; binary32 may be required for memory but needs independent qualification. The real-arithmetic proofs select neither production precision nor hardware.

## Transport and forces

Implement component-aware sampling, tracing, collisions and source scheduling. Bounded scalar transport precedes optional correction. Test constants, bounds, translation and limiter behavior. Gravity and constant viscosity follow with hydrostatic and decay tests. Projection and boundary reapplication follow the derived stage order, not buffer convenience.

A step publishes a complete accepted interval or an explicit incomplete outcome. Cancellation preserves accepted state and identifies completed physical time. Renderers must not see half-updated arrays.

## Guidance and application

Export calibrated silhouette, depth and normals for primitives and state. Retain camera and state IDs so hosts cannot combine stale images with new metadata. A lagging renderer may be intentional but must consume a coherent snapshot.

Build the demonstration around the core. Acceptance exercises create, advance, pause, reset, parameter changes, cancellation and export. None is proved by the research scripts or Lean algebra.

## Liquid and performance

Select an interface method after comparing volume drift, silhouette accuracy and memory. Add its boundary and surface tests before advertising liquid support. Then profile complete frames under intended host model allocation. Record hardware, build, scene, warm-up, variability and peak memory. Optimize measured contributors while retaining diagnostics.

Ports share fixtures, units, serialization and failure meanings. Matching signatures is insufficient. Stabilize Rust first so three languages do not evolve three interpretations of one equation.

## Concrete type and function contracts

The following Rust-shaped interfaces are a design specification, not compiled library code. They make coordinate domains and ownership visible:

~~~
struct CellShape { nx: usize, ny: usize, nz: usize, len: usize }
struct FaceShape { axis: Axis, dims: [usize; 3], len: usize }
struct GridGeometry { cells: CellShape, faces: [FaceShape; 3],
                      origin: [f64; 3], spacing: [f64; 3] }
struct BoundarySnapshot { version: u64, cell_kind: Vec<CellKind>,
                          face_data: [Vec<FaceBoundary>; 3] }
struct FaceField { x: Vec<f32>, y: Vec<f32>, z: Vec<f32> }
struct FluidState { id: u64, time: f64, velocity: FaceField,
                    tracer: Vec<f32> }
struct PressureWorkspace { p: Vec<f64>, rhs: Vec<f64>,
                           residual: Vec<f64>, direction: Vec<f64>,
                           product: Vec<f64>, preconditioned: Vec<f64> }
~~~

Private constructors validate lengths and finite geometry. A CellIndex cannot be constructed from an unchecked integer through the public API. FaceIndex includes axis or comes from an axis-specific view. Avoid retaining both an authoritative len and independently mutable dimensions; derived capacities remain immutable after construction.

A proposed apply_pressure_operator takes immutable topology, immutable pressure and a mutable output slice of validated length. It must overwrite every active output, including isolated or boundary rows according to classification. A divergence function consumes velocity and the same boundary snapshot but uses an independent direct face-flux traversal for validation. A project function returns a PressureOutcome containing diagnostics and never silently reclassifies invalid geometry.

A sample_velocity function accepts a world position and boundary policy and returns either a finite vector with validity metadata or a typed sample failure. Advection accepts immutable old fields and separate outputs. A step function accepts a physical interval and returns either an accepted snapshot or a failure carrying the last complete time. It must not expose mutable workspace through callbacks.

## Storage and invariants

Use x-fastest contiguous arrays initially. For cells, strides are \(1,n_x,n_xn_y\); for x-faces they are \(1,n_x+1,(n_x+1)n_y\). The y- and z-face strides follow their own dimensions. Validate the multiplication when building shapes, then retain strides for kernel use.

Boundary data should distinguish fluid neighbor, prescribed normal speed, prescribed pressure and inactive face. A floating coefficient of zero is not enough to encode all four meanings. PressureTopology owns reduced indices, component IDs, gauge choices and face transmissibilities. Its version depends on geometry, density policy and active-fluid classification.

State publication increments a version only after diagnostics accept the candidate. A renderer borrows an immutable accepted snapshot. Reset constructs a new coherent state rather than zeroing arrays piecemeal while consumers read them. Parameter changes that alter array shape or topology invalidate dependent workspace and preconditioners explicitly.

## Executable acceptance fixtures

The companion provides expected mathematical results for a production test port:

- The three-cell chain in Appendix E must yield pressure \((0,4,2)\) and zero corrected internal velocity at time step one half.
- The cycle fixture must retain equal circulation rather than force every face to zero.
- All four lattice samplers must reproduce an affine scalar within precision tolerance; the intentionally wrong offset must fail.
- The pressure operator must match an independently assembled small matrix on rectangular dimensions.
- A closed component with nonzero prescribed net flux must return incompatible flux before iteration.
- A zero right-hand side must terminate without dividing by zero.
- The rational flux fixture must preserve total amount; floating ports compare with an appropriate absolute budget.
- A deliberately truncated particle support must expose moment failure rather than pass the full-support affine assertion.

Add integration fixtures for cancellation after each stage, resource rejection before mutation and stale boundary versions. Those are not mathematical experiments; they require the actual Rust state owner and host boundary.

## Dependency and implementation choices

Use established sparse and numerical libraries for reference solves where licensing and deployment allow, while retaining a transparent matrix-free production operator if justified by the memory contract. Pin toolchains and dependencies for qualification, and record the exact compiler and feature set used in numerical comparisons.

No unsafe code is required for the first bounded CPU implementation. Introduce unsafe indexing or SIMD only after profiling identifies a meaningful contributor and the validated shape contract supports a reviewed safety argument. Bounds-check elimination by the compiler is preferable to a manually duplicated unchecked path when performance is equivalent.

GPU kernels require a separate buffer-layout and dispatch contract, including alignment, workgroup limits, synchronization and readback. A CPU test cannot establish a shader's binding layout or race freedom. Keep backend-independent fixtures but qualify each backend through its real execution path before advertising support.
