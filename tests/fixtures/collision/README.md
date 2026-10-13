# Collision import fixtures

These original fixtures are distributed under the repository Apache-2.0 license.

- `cube.obj`: outward-oriented unit cube with six convex quad faces; exercises exact fan triangulation.
- `domain-box.obj`: outward box spanning `(0.25, 0.15, 0.25)` to `(0.75, 0.35, 0.75)` for native unit-domain demonstrations.
- `cube.bin`: eight float32 XYZ vertices and 36 uint16 triangle indices.
- `hierarchy.gltf`: external-buffer cube instanced under parent translation `(2,0,0)` and child scale `(-1,2,1)`. Result spans `(1,0,0)` to `(2,2,1)` with outward winding after reflection correction.
- `embedded.gltf`: identical geometry/hierarchy using an embedded base64 buffer.
- `cube.glb`: identical geometry/hierarchy in glTF 2.0 binary container.

The contract tests generate invalid mutations in isolated temporary directories, including nonfinite positions, bad indices/ranges, oversized input, singular transforms, cycles/deep hierarchy, unsupported extensions and primitives, remote/traversal/escaped URIs, missing buffers, and symlink escape. No fixture requests a network resource.

Import acceptance means only that triangle geometry was decoded. It does not establish that a visual mesh is a valid collision solid.

## Synthetic authoring and binary layout

Fixtures were authored specifically for this change from the eight binary-coordinate unit-cube corners, without third-party model sources or generated solver output. OBJ lists those corners explicitly. Each outward quad `(a,b,c,d)` becomes triangles `(a,b,c)` and `(a,c,d)` in the glTF buffers, with indices converted from OBJ's one-based convention to zero-based indices. Bounds and signed volumes in tests are analytical expectations, not values copied from the importer.

`cube.bin` is exactly 168 bytes, little-endian: offsets 0–95 contain eight tightly packed float32 XYZ triples; offsets 96–167 contain 36 unsigned 16-bit indices (twelve triangles). `hierarchy.gltf` declares these two buffer views and accessors; `embedded.gltf` substitutes the standard base64 encoding of those same bytes. `cube.glb` contains a 12-byte glTF-v2 header, a JSON chunk with space padding to a four-byte boundary, and a BIN chunk containing the same 168 bytes. Chunk lengths exclude their eight-byte chunk headers; the container header length includes the entire file.

The importer supports strictly convex planar OBJ polygons (up to 64 corners), and static glTF TRIANGLES with ordinary non-sparse accessors. Animation, skins, morph targets, extensions/compression, ambiguous scene selection, and external URLs are unsupported. Textures/materials carry no collision semantics and are ignored. Source coordinates retain their axes; glTF dimensions are metres, and OBJ units require explicit caller scale. Import tests separately mutate a parent matrix to a 90-degree Z rotation and verify individual transformed vertices and bounds against a written analytical formula.

Transform validation rejects a node containing both matrix and TRS fields, non-unit quaternions, and local matrices with shear (relative float32 tolerance `1e-5`). The importer checks local matrices before composing the hierarchy: a valid parent scale composed with child rotation can produce a sheared world matrix and remains supported. Quaternions are validated rather than silently normalized.
