# Static OBJ and glTF collision meshes

The native **liquid / gas material simulation** can import one static OBJ,
glTF 2.0 or GLB model and collide its particle centers against that geometry.
This is a preview boundary response, not validated real-fluid wall traction.
The original grid carrier mode does not support this mesh boundary and rejects
such a request. No GPU, moving-body or two-way coupling is implemented here.

## Run it

```sh
cargo run --locked --release --features desktop --bin rheon-desktop
# Reproducible model below the default rotating emitter:
cargo run --locked --release --features desktop --bin rheon-desktop -- \
  --collision-mesh tests/fixtures/collision/domain-box.obj \
  --collision-policy solid --collision-run
# Headless actual particle stepping, water or air:
cargo run --locked --release --example collision_mesh -- \
  tests/fixtures/collision/domain-box.obj solid water
```

Open **Collision mesh (OBJ / glTF / GLB)**, enter a local path, choose scale,
translation and admission policy, then **Import / apply mesh**. Import failure
retains the prior mesh. A successful apply/removal clears old recorded history;
**Reset timeline (keep mesh)** clears history while retaining the selected model. Simulate
creates a fresh particle state and captures the mesh in an immutable run request.
Controls are disabled while a worker owns a running simulation; cancel first.
Amber wireframe is the actual collision geometry, intentionally shown through
fluid as an inspection overlay. It is not a shaded material/texture preview.

Placement uses positive uniform scale followed by XYZ translation in metres.
glTF hierarchy transformations are applied first. Source axes are preserved
(the viewer is Y-up); OBJ has no implicit unit inference or automatic centering.
Use the glTF model's node transforms for rotation; native per-axis rotation and
nonuniform placement controls are outside this milestone. Bounds must fit the
entire simulation domain; out-of-bounds geometry is rejected, never clipped.

## Physical admission and response

**Validated convex watertight solid** requires one connected, closed,
consistently wound indexed surface with every undirected edge appearing twice
in opposite directions, no unused vertices, and every vertex on the interior
side of each face plane. Global inward winding is accepted and normals are
oriented outward internally. Concave shapes, holes, inconsistent winding,
disconnected objects and duplicated seam vertices that break indexed closure
are refused. There is no automatic vertex welding, hole filling, convex hull,
voxelization or silent bounding-box replacement. Export a welded convex mesh,
or explicitly choose the surface policy when that is the intended model.

**Two-sided surface barrier** admits open/disconnected/concave triangle surfaces
without inferring a solid interior. Winding does not select a permitted crossing
direction. Duplicated or intersecting surfaces may cause conservative sticking;
there is no manifoldness, self-intersection or arbitrary solid reliability claim.
Particles initially on either side remain on their respective side unless they
go around an edge. This is not a way to fill the interior of a nonconvex solid.

The collision owner builds an immutable balanced AABB BVH and sweeps each
particle's proposed center segment. It stops displacement before the earliest
contact, removes normal velocity at a single plane, and stops all velocity at
simultaneous nonparallel contacts. Tangential displacement resumes next step;
unused time after impact is discarded. Numerically coplanar/ambiguous contact
can stop conservatively. Motion is swept rather than endpoint-only, so a fast
center crossing a resolved reference face is intercepted. Geometrically
ill-conditioned triangles and excessively small/extreme coordinates are refused.
Public sweeps also reject coordinate/displacement scales whose floating-point
spacing cannot resolve numerical clearance (`QueryPrecisionUnsupported`).

This is **point-center, zero-thickness collision**. Particle rendering/splat and
SPH kernel support can overlap the visual surface; their radii are not contact
radii. Pairwise SPH kernels are not occluded by barriers. No ghost-particle wall
support, calibrated boundary pressure, mesh friction/heat transfer, no-slip law,
boundary force ledger or two-way body motion is supplied. The existing domain
wall behavior remains separate. Reduced-stiffness liquid and ideal-gas preview
material limitations still apply. See [material model](material-fluids.md).

At initial attachment, centers inside/on a convex solid are excluded along with
their mass; mass is not redistributed. Removing it does not restore excluded
particles in a library simulation; creating/resetting a simulation does.
Later attachment that overlaps an accepted center rejects transactionally.
An emitter attempting to create a center inside/on the solid rejects the step;
it does not silently discard emitted mass. Failed/cancelled steps preserve the
accepted state, clock and emission bookkeeping. Static collision uses no
independent body clock and does not change the particle neighbor algorithm.

## Import restrictions and resource ownership

OBJ accepts UTF-8 `v x y z`, positive/relative negative position indices, triangle
faces and strictly convex planar polygons up to 64 corners. Polygons are fan
triangulated after convexity/planarity checks. Concave, self-crossing, nonplanar
or collinear polygons must be triangulated explicitly before import. Position
indices from slash-separated face fields are used; texture/normal fields,
materials, smoothing and groups do not alter collision. Freeform geometry,
homogeneous/colored vertices and other unsupported records reject. Material
libraries are never opened.

glTF 2.0 supports the default scene, or the sole scene when no default is named,
with recursive node matrix/TRS transforms and mesh instances. Reflection flips
triangle winding; nonfinite, singular or non-affine transforms reject.
Only unit-quaternion TRS or an unambiguous nonsheared local matrix is admitted;
quaternion norm-squared and local column-orthogonality tolerances are 1e-5 to
accommodate glTF f32 representation. Matrix plus TRS rejects. Only
TRIANGLES primitives, FLOAT VEC3 positions, and optional unsigned scalar indices
are admitted. Accessor offsets, strides and declared resource bounds are checked
before decoding. Geometry expansion is counted across instances. Animation,
skinning, morph targets, sparse accessors, nontriangle primitives and declared
extensions (including compressed/quantized geometry) reject. Materials, cameras,
lights, textures and images supply no collision semantics and images are not read.
The format reference is the
[Khronos glTF 2.0 specification](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html).

GLB binary chunks, supported base64 buffer data URIs and local relative buffer
files are supported. External URLs are never fetched. Buffer paths with schemes,
absolute paths, traversal, percent escapes, backslashes, queries or fragments
reject. Held directory capabilities prevent symlink escapes during buffer use.
Only regular local files are admitted. File I/O has no real-time deadline; do not
interpret bounded byte/geometry limits as a filesystem latency guarantee.

| Owner | Bound |
| --- | --- |
| Import | 16 MiB primary source; 64 MiB aggregate primary source and buffers |
| Expanded geometry | 200,000 vertices and 200,000 triangles in importer |
| Hierarchy | 4,096 nodes, depth 64 |
| Collision owner | 100,000 vertex/triangle capacity each |
| Convex admission | 4,096 faces; at most 4,000,000 face/vertex checks |
| Native rendering | 12,000 triangles; larger models explicitly rejected by UI |

Limits are cumulative constraints, not promises that every mesh below them is
physically admitted. Retained mesh array capacities count toward the particle
simulation's configured memory budget on attachment. Parser/temporary storage,
UI projections, shared Arc copies, allocator metadata and process RSS are not a
hard whole-program memory cap. BVH queries allocate no heap storage. Build and
worst-case traversal remain bounded by admitted geometry, not constant time.

Dependencies are owned by `mesh_import`: pinned `gltf` for the standardized
document/accessor format (image/import loaders disabled), `base64` for embedded
buffers, and `cap-std` for capability-relative resource reads. Unix `rustix` supplies
nonblocking file-open flags so FIFO replacement cannot block the import thread. gltf/base64 are
MIT OR Apache-2.0; cap-std offers Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR
MIT; rustix has the same license alternatives. The narrow OBJ polygon grammar is locally owned to refuse ambiguous
physical triangulation rather than adopt permissive visual repair behavior.

## Library integration and evidence

`mesh_import::import_mesh(path, placement)` returns world-space geometry only.
`CollisionMesh::new(vertices, triangles, policy)` establishes the selected
collision contract; share it as `Arc<CollisionMesh>` and attach through
`ParticleFluidSimulation::set_collision_mesh`. Public getters provide geometry,
bounds, policy and retained array bytes for native/CAD consumers. The API owner
can wrap this existing owner in MaterialSession later; this branch does not
fork the unmerged shared facade, diagnostics, CAD/VTK or AI-provider contracts.

Run `cargo test --locked --test mesh_import_contract --test mesh_collision_contract`
and `cargo test --locked --lib particle_fluid::physics_tests` for focused evidence.
`cargo run --locked --release --example collision_mesh -- --profile` measures BVH
build/query workloads and serial liquid/gas stepping with/without a reference
barrier. Its analytical checks are independent of triangle traversal. Numerical
tests cover simple reference shapes, not arbitrary imported-geometry reliability.
See [verification report](plans/static-collision-mesh/reports/verification.md)
for exact environment, outcomes, timings, screenshots and remaining limitations.
