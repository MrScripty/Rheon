# Static imported collision meshes

**Plan status:** Active. **Current phase:** static particle collision milestone.
**Next slice:** integrate importer, immutable collision owner and native controls.
**Acceptance status:** pending.
Operation: start, authorized by the explicit implement/push/draft-PR request.
See [ledger](execution-ledger.md), [issues](issues.md), and [reports](reports/).

## Objective and scope

Import a local OBJ, glTF 2.0 or GLB, place it in the native liquid/gas domain,
and observe actual static particle collision response. Provide reproducible
reference tests, explicit admission rules, bounded input/compute, and measured
CPU cost. No network resource downloads, main merge or deployment.

Convex watertight solids and explicitly selected two-sided surface barriers are
distinct policies. A visual mesh is never silently assumed to be a valid solid.
Collision acts on particle centers; it is not resolved finite-radius contact,
calibrated pressure/shear, no-slip SPH wall support, or moving-body coupling.
The original grid carrier and box-only obstacle solvers remain unchanged.

## Binding decisions and inspected contracts

Base: main `2209012bc5f902693d0f695ec1c2d854126af317`, isolated branch
`feat/static-collision-mesh`. The initial workspace was old `6ba41c7`; fetched
current main before implementation. No unmerged milestone was blindly merged.
Read particle solver, TriangleSurface (two-sided queries explicitly do not infer
solid volume), StaticObstacleGeometry (axis-aligned closed box only), native
worker/history/viewer, PR53 material session, PR52 planar coupling, PR54 CAD plan,
and PR55 production-provider contracts. PR47/48/53/54/55 shared facade/diagnostic
and visualization ownership remains with their existing owners.

Mesh importer owns decoding, local asset security and world-space placement.
CollisionMesh owns immutable physical admission, BVH and swept center response.
ParticleFluidSimulation owns attach/step publication and initial solid exclusion.
Desktop owns draft input versus accepted run geometry. A run retains its mesh Arc;
changing the next input must never relabel retained history.

## Milestone and write set

M1 (Active): runnable static import/collision/native UI with reference evidence.

- Import delegate: `src/mesh_import.rs`, optional `src/mesh_import/`,
  `tests/mesh_import_contract.rs`, `tests/fixtures/collision/`, Cargo dependencies.
- Collision delegate: `src/mesh_collision.rs`, `src/particle_fluid.rs`,
  `tests/mesh_collision_contract.rs`.
- Integration owner: `src/lib.rs`, `src/bin/rheon-desktop.rs`,
  `src/bin/desktop/collision_ui.rs`, `src/bin/desktop/viewport.rs`,
  `src/bin/desktop/README.md`, `examples/collision_mesh.rs`,
  `docs/collision-meshes.md`, this plan directory and a focused CI workflow.
- Forbidden: book and source-binding repair; session facade; CAD inspector/VTK
  owners; numerical pressure/shear and moving-plate owner implementations.

Shared hooks for parent integration: immutable `Arc<CollisionMesh>` on run
request and `ParticleFluidSimulation::set_collision_mesh`; importer returns
world-space vertices/triangles; viewport consumes the exact accepted run mesh.
Facade owner can add a construction option later without copying admission.
CAD owner can consume public vertices/triangles and bounds. Integration owner
alone modifies shared exports/composition. Independent delegates have disjoint
write sets; no stale shared-plan proposal protocol is necessary.

## Objective acceptance and oracles

| ID | Observable criterion | Kind / environment / mode | Status |
| --- | --- | --- | --- |
| A1 | OBJ/glTF/GLB fixtures have analytically expected world coordinates; malformed, unsafe, missing and unsupported inputs reject specifically | contract / Linux Rust 1.92 / automated | pending |
| A2 | BVH sweeps and liquid/gas reference particles do not cross reference plane/box; solid admission refuses invalid solids; failures preserve publication | contract / CPU / automated | pending |
| A3 | Native import, placement, removal, reset/repeat and retained geometry are usable; actual rendered screenshots if display available | GUI / Linux display / automated+manual | pending |
| A4 | Release CPU build/query/step cost and retained storage measured without GPU or real-time claim | performance / named CPU / automated | pending |
| A5 | Independent review, format, strict lint, tests, pushed commit and draft PR; hosted CI disposition recorded | integration / local+GitHub / automated+review | pending |

Reference expectations come from analytical plane/box geometry and Khronos glTF
transform/accessor semantics, not agreement between two local projections.
Fixtures cannot establish arbitrary geometry reliability. Negative cases must
reach their intended validation boundary. Initial physical model limitations
remain visible even when these checks pass.

## Simplicity and ownership review: applicable

1. Parsing changes with formats/security; collision changes with geometry and
   response; simulation changes with transaction/lifecycle; UI presents them.
2. Geometry is immutable after admission. Placement is a new value, not mutation
   of a worker's mesh. State time belongs to the simulation; no new identity registry.
3. Callers know a path, SI scale/translation, an explicit policy and bounded mesh
   owner. They need no buffer decoding, BVH traversal or collision math.
4. Adding an import format touches importer/tests; changing response touches
   collision/tests; facade integration uses the same owner; UI layout stays local.
5. APIs pass ordinary coordinates and immutable ownership, not hidden parser or
   BVH representation. Existing history keeps its request's immutable mesh.
6. Import and geometric queries test independently; particle integration adds
   transactional evidence; UI failure cannot partially replace accepted geometry.
7. Removing importer or collision modules scatters required validation/math into
   consumers. No new provider registry, cache service, wire schema or generic adapter.
8. Necessary complexity is safe standardized decoding, convex admission and
   accelerated swept queries. Unqualified arbitrary solid topology is refused.

## Standards and constraints

No repository AGENTS.md or .agents skills found. Read public coding-standards
`188beda1`: Core/Router; Planning/Discovery/Implementation/Verification/GUI/oracles,
Development Proportionality, Commit/Documentation; Library, Rust/API/dependencies/
security; Architecture/Contracts/Performance/Security and dependency ownership.
This is source-guided application work, not a Standards Engine certificate.
External parser dependencies must have compatible licenses and explicit limits.

## Blockers and re-plan triggers

No implementation blocker. Re-plan for unsupported physical response being
presented as real, unbounded import/query cost, inability to retain immutable
history geometry, or changes requiring the other owners' facade/book/CAD writes.
Any missing GUI/CI environment evidence is recorded, never replaced by a fake image.
Retain checkout/feature branch for parent review; no worktree cleanup requested.

## Final acceptance

Pending named evidence; draft PR is a review boundary, not acceptance of arbitrary
geometry, production physical accuracy, GPU performance or main integration.
