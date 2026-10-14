//! Bounded static collision-geometry import. Parsing never establishes solid validity.
//!
//! OBJ accepts positions and strictly convex planar polygon faces (maximum 64 corners).
//! glTF 2.0/GLB accepts the default scene, or the sole scene, with triangle primitives.
//! Materials, normals and images have no collision meaning and are not loaded. Animation,
//! skinning, morph targets, sparse accessors and extensions are rejected. glTF units are
//! metres; OBJ units are caller-defined via `MeshPlacement::scale`. Both retain source axes.
use base64::Engine;
use std::{
    fmt,
    io::Read,
    path::{Component, Path},
};

pub const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_RESOURCE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_VERTICES: usize = 200_000;
pub const MAX_TRIANGLES: usize = 200_000;
pub const MAX_NODES: usize = 4096;
pub const MAX_DEPTH: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshPlacement {
    pub scale: f64,
    pub translation: [f64; 3],
}
impl Default for MeshPlacement {
    fn default() -> Self {
        Self {
            scale: 1.0,
            translation: [0.0; 3],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshBounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshFormat {
    Obj,
    Gltf,
    Glb,
}
#[derive(Clone, Debug)]
pub struct ImportedMesh {
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[usize; 3]>,
    pub bounds: MeshBounds,
    pub source_format: MeshFormat,
    pub diagnostics: Vec<String>,
}
#[derive(Debug)]
pub enum MeshImportError {
    Io(std::io::Error),
    Invalid(String),
    Unsupported(String),
    Limit(&'static str),
}
impl fmt::Display for MeshImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "mesh resource I/O: {e}"),
            Self::Invalid(s) => write!(f, "invalid mesh: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported mesh: {s}"),
            Self::Limit(s) => write!(f, "mesh resource limit: {s}"),
        }
    }
}
impl std::error::Error for MeshImportError {}
impl From<std::io::Error> for MeshImportError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
type Geometry = (Vec<[f64; 3]>, Vec<[usize; 3]>);
type Result<T> = std::result::Result<T, MeshImportError>;
fn invalid(s: impl Into<String>) -> MeshImportError {
    MeshImportError::Invalid(s.into())
}
fn unsupported(s: impl Into<String>) -> MeshImportError {
    MeshImportError::Unsupported(s.into())
}
// O_NONBLOCK prevents FIFO replacement between metadata and open from blocking
// the native thread; regular files are unaffected. The held handle is checked again.
#[cfg(unix)]
fn nonblocking_flag() -> Result<i32> {
    i32::try_from(rustix::fs::OFlags::NONBLOCK.bits())
        .map_err(|_| unsupported("platform nonblocking file flag"))
}
fn bounded_read(mut file: impl Read, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    file.by_ref()
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(MeshImportError::Limit("file/resource bytes"));
    }
    Ok(bytes)
}
/// Import geometry without asserting watertightness or suitability for any solver.
/// External buffer paths are opened relative to a held directory capability. Schemes,
/// percent escapes, traversal, absolute paths, and symlinks escaping that directory fail.
pub fn import_mesh(path: &Path, placement: MeshPlacement) -> Result<ImportedMesh> {
    if !placement.scale.is_finite()
        || placement.scale <= 0.0
        || !placement.translation.iter().all(|x| x.is_finite())
    {
        return Err(invalid(
            "placement requires positive finite scale and finite translation",
        ));
    }
    if !std::fs::metadata(path)?.is_file() {
        return Err(invalid("mesh resource must be regular file"));
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(nonblocking_flag()?);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid("mesh resource must be regular file"));
    }
    let bytes = bounded_read(file, MAX_FILE_BYTES)?;
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let format = match extension.as_str() {
        "obj" => MeshFormat::Obj,
        "gltf" => MeshFormat::Gltf,
        "glb" => MeshFormat::Glb,
        _ => return Err(unsupported("expected .obj, .gltf or .glb")),
    };
    let (mut vertices, triangles) = if format == MeshFormat::Obj {
        obj(&bytes)?
    } else {
        gltf_mesh(path, &bytes)?
    };
    if vertices.is_empty() || triangles.is_empty() {
        return Err(invalid("no triangle geometry in selected scene"));
    }
    let mut bounds = MeshBounds {
        min: [f64::INFINITY; 3],
        max: [f64::NEG_INFINITY; 3],
    };
    for p in &mut vertices {
        for (axis, coordinate) in p.iter_mut().enumerate() {
            *coordinate = *coordinate * placement.scale + placement.translation[axis];
            if !coordinate.is_finite() {
                return Err(invalid("nonfinite transformed position"));
            }
            bounds.min[axis] = bounds.min[axis].min(*coordinate);
            bounds.max[axis] = bounds.max[axis].max(*coordinate);
        }
    }
    for t in &triangles {
        validate_triangle(vertices[t[0]], vertices[t[1]], vertices[t[2]])?;
    }
    Ok(ImportedMesh{vertices,triangles,bounds,source_format:format,diagnostics:vec!["Geometry only: solid admission, resolution and boundary policy require separate validation. Materials/textures ignored; source axes preserved.".into()]})
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn validate_triangle(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Result<()> {
    let u = sub(b, a);
    let v = sub(c, a);
    let n = cross(u, v);
    let area = dot(n, n);
    let scale = dot(u, u) * dot(v, v);
    if !area.is_finite() || !scale.is_finite() || area <= scale * 1e-24 || scale == 0.0 {
        return Err(invalid("degenerate or numerically unresolvable triangle"));
    }
    Ok(())
}
fn obj(bytes: &[u8]) -> Result<Geometry> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid("OBJ must be UTF-8"))?;
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let mut tokens = line.split('#').next().unwrap_or("").split_whitespace();
        let Some(kind) = tokens.next() else { continue };
        match kind {
            "v" => {
                if vertices.len() >= MAX_VERTICES {
                    return Err(MeshImportError::Limit("vertices"));
                }
                let mut p = [0.; 3];
                for x in &mut p {
                    *x = tokens
                        .next()
                        .ok_or_else(|| invalid("OBJ position requires xyz"))?
                        .parse::<f64>()
                        .map_err(|_| invalid("OBJ position number"))?;
                }
                if tokens.next().is_some() {
                    return Err(unsupported("OBJ homogeneous/colored vertices"));
                }
                if !p.iter().all(|x| x.is_finite()) {
                    return Err(invalid("nonfinite OBJ position"));
                }
                vertices.push(p);
            }
            "f" => {
                let mut face = Vec::new();
                for token in tokens {
                    if face.len() >= 64 {
                        return Err(MeshImportError::Limit("OBJ face corners (64)"));
                    }
                    let index = token
                        .split('/')
                        .next()
                        .unwrap_or("")
                        .parse::<i64>()
                        .map_err(|_| invalid("OBJ face index"))?;
                    let n =
                        i64::try_from(vertices.len()).map_err(|_| invalid("OBJ vertex count"))?;
                    let index = if index > 0 {
                        index - 1
                    } else if index < 0 {
                        n + index
                    } else {
                        return Err(invalid("OBJ indices are nonzero"));
                    };
                    if index < 0 || index >= n {
                        return Err(invalid(format!(
                            "OBJ face index out of range at line {}",
                            line_no + 1
                        )));
                    }
                    face.push(usize::try_from(index).map_err(|_| invalid("OBJ index"))?);
                }
                if face.len() < 3 {
                    return Err(invalid("OBJ face needs three corners"));
                }
                if triangles.len() + face.len() - 2 > MAX_TRIANGLES {
                    return Err(MeshImportError::Limit("triangles"));
                }
                // Strict convexity/planarity makes a fan exact; reject concave/self-crossing
                // polygons instead of guessing an ear-clipping interpretation.
                let a = vertices[face[0]];
                let n = cross(sub(vertices[face[1]], a), sub(vertices[face[2]], a));
                let n2 = dot(n, n);
                if n2 == 0.0 || !n2.is_finite() {
                    return Err(invalid("degenerate OBJ face"));
                }
                for i in 0..face.len() {
                    let p = vertices[face[i]];
                    let d = sub(p, a);
                    if dot(n, d).abs() > 1e-9 * n2.sqrt() * dot(d, d).sqrt().max(1e-30) {
                        return Err(unsupported("nonplanar OBJ polygon; triangulate explicitly"));
                    }
                    let q = vertices[face[(i + 1) % face.len()]];
                    for j in 0..face.len() {
                        if j == i || j == (i + 1) % face.len() {
                            continue;
                        }
                        if dot(n, cross(sub(q, p), sub(vertices[face[j]], p))) <= 0.0 {
                            return Err(unsupported(
                                "OBJ polygon must be strictly convex and simple",
                            ));
                        }
                    }
                }
                for i in 1..face.len() - 1 {
                    triangles.push([face[0], face[i], face[i + 1]]);
                }
            }
            "vt" | "vn" | "o" | "g" | "s" | "usemtl" | "mtllib" => {}
            _ => {
                return Err(unsupported(format!(
                    "OBJ record {kind}; only polygon geometry supported"
                )));
            }
        }
    }
    Ok((vertices, triangles))
}
type Mat = [[f64; 4]; 4];
const IDENTITY: Mat = [
    [1., 0., 0., 0.],
    [0., 1., 0., 0.],
    [0., 0., 1., 0.],
    [0., 0., 0., 1.],
];
fn mul(a: Mat, b: Mat) -> Mat {
    std::array::from_fn(|c| std::array::from_fn(|r| (0..4).map(|k| a[k][r] * b[c][k]).sum()))
}
fn determinant(m: Mat) -> f64 {
    dot(
        [m[0][0], m[0][1], m[0][2]],
        cross([m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]]),
    )
}
fn gltf_mesh(path: &Path, bytes: &[u8]) -> Result<Geometry> {
    let parsed = gltf::Gltf::from_slice(bytes).map_err(|e| invalid(format!("glTF parse: {e}")))?;
    if parsed.extensions_used().next().is_some() || parsed.extensions_required().next().is_some() {
        return Err(unsupported("glTF extensions (including compression)"));
    }
    if parsed.animations().next().is_some() || parsed.skins().next().is_some() {
        return Err(unsupported("animated/skinned glTF"));
    }
    if parsed.nodes().len() > MAX_NODES {
        return Err(MeshImportError::Limit("nodes"));
    }
    // gltf-json validates indices/types, but does not enforce exclusive matrix/TRS
    // representation or unit quaternions. Validate original fields before the
    // high-level transform accessor can discard an ambiguous representation.
    // A 1e-5 relative tolerance accommodates float32 rotation serialization.
    const TRANSFORM_TOLERANCE: f64 = 1e-5;
    for node in &parsed.as_json().nodes {
        if node.weights.is_some() {
            return Err(unsupported("node morph weights"));
        }
        if node.matrix.is_some()
            && (node.rotation.is_some() || node.scale.is_some() || node.translation.is_some())
        {
            return Err(invalid("glTF node cannot combine matrix and TRS"));
        }
        if let Some(rotation) = &node.rotation {
            let norm_squared: f64 = rotation.0.iter().map(|x| f64::from(*x).powi(2)).sum();
            if !norm_squared.is_finite() || (norm_squared - 1.0).abs() > TRANSFORM_TOLERANCE {
                return Err(invalid("glTF rotation quaternion must have unit length"));
            }
        }
        if let Some(matrix) = node.matrix {
            let columns: [[f64; 3]; 3] =
                std::array::from_fn(|c| std::array::from_fn(|r| f64::from(matrix[c * 4 + r])));
            for a in 0..3 {
                for b in a + 1..3 {
                    let scale = (dot(columns[a], columns[a]) * dot(columns[b], columns[b])).sqrt();
                    if dot(columns[a], columns[b]).abs() > TRANSFORM_TOLERANCE * scale {
                        return Err(invalid(
                            "glTF node matrix contains shear; must decompose to TRS",
                        ));
                    }
                }
            }
        }
    }
    for mesh in parsed.meshes() {
        if mesh.weights().is_some() {
            return Err(unsupported("morph weights"));
        }
        for p in mesh.primitives() {
            if p.morph_targets().next().is_some() {
                return Err(unsupported("morph targets"));
            }
            if p.mode() != gltf::mesh::Mode::Triangles {
                return Err(unsupported("glTF primitive must be TRIANGLES"));
            }
        }
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let dir = cap_std::fs::Dir::open_ambient_dir(parent, cap_std::ambient_authority())?;
    let mut buffers = Vec::new();
    let mut total = bytes.len();
    for buffer in parsed.buffers() {
        if buffer.length() > MAX_RESOURCE_BYTES.saturating_sub(total) {
            return Err(MeshImportError::Limit("total buffer bytes"));
        }
        let data = match buffer.source() {
            gltf::buffer::Source::Bin => parsed
                .blob
                .clone()
                .ok_or_else(|| invalid("GLB missing binary chunk"))?,
            gltf::buffer::Source::Uri(uri) => {
                if let Some(data) = uri
                    .strip_prefix("data:application/octet-stream;base64,")
                    .or_else(|| uri.strip_prefix("data:application/gltf-buffer;base64,"))
                {
                    if data.len() > MAX_RESOURCE_BYTES.saturating_sub(total) * 4 / 3 + 4 {
                        return Err(MeshImportError::Limit("base64 bytes"));
                    }
                    base64::engine::general_purpose::STANDARD
                        .decode(data)
                        .map_err(|_| invalid("buffer base64"))?
                } else {
                    let resource = Path::new(uri);
                    if uri.is_empty()
                        || uri.contains([':', '%', '\\', '?', '#'])
                        || resource
                            .components()
                            .any(|c| !matches!(c, Component::Normal(_)))
                    {
                        return Err(unsupported(
                            "buffer URI must be a plain relative local path without traversal",
                        ));
                    }
                    if !dir.metadata(resource)?.is_file() {
                        return Err(invalid("buffer resource must be regular file"));
                    }
                    let mut options = cap_std::fs::OpenOptions::new();
                    options.read(true);
                    #[cfg(unix)]
                    {
                        use cap_std::fs::OpenOptionsExt;
                        options.custom_flags(nonblocking_flag()?);
                    }
                    let file = dir.open_with(resource, &options)?;
                    if !file.metadata()?.is_file() {
                        return Err(invalid("buffer resource must be regular file"));
                    }
                    bounded_read(file, MAX_RESOURCE_BYTES.saturating_sub(total))?
                }
            }
        };
        total = total
            .checked_add(data.len())
            .ok_or(MeshImportError::Limit("total bytes"))?;
        if total > MAX_RESOURCE_BYTES {
            return Err(MeshImportError::Limit("total bytes"));
        }
        if data.len() < buffer.length() {
            return Err(invalid("buffer shorter than declared byteLength"));
        }
        buffers.push(data);
    }
    // The crate's reader assumes valid ranges. Validate every view/accessor before it
    // constructs iterators, including unused accessors in an untrusted document.
    for view in parsed.views() {
        let end = view
            .offset()
            .checked_add(view.length())
            .ok_or_else(|| invalid("buffer view overflow"))?;
        if end > view.buffer().length() {
            return Err(invalid("buffer view exceeds declared buffer"));
        }
    }
    for accessor in parsed.accessors() {
        if accessor.sparse().is_some() {
            return Err(unsupported("sparse glTF accessors"));
        }
        let view = accessor
            .view()
            .ok_or_else(|| unsupported("accessor without buffer view"))?;
        if accessor.count() == 0 || accessor.count() > MAX_VERTICES.max(MAX_TRIANGLES * 3) {
            return Err(MeshImportError::Limit("accessor count"));
        }
        let stride = view.stride().unwrap_or(accessor.size());
        if stride < accessor.size() {
            return Err(invalid("accessor stride smaller than element"));
        }
        let end = (accessor.count() - 1)
            .checked_mul(stride)
            .and_then(|v| v.checked_add(accessor.offset()))
            .and_then(|v| v.checked_add(accessor.size()))
            .ok_or_else(|| invalid("accessor range overflow"))?;
        if end > view.length() {
            return Err(invalid("accessor exceeds buffer view"));
        }
    }
    let scene = parsed
        .default_scene()
        .or_else(|| {
            if parsed.scenes().len() == 1 {
                parsed.scenes().next()
            } else {
                None
            }
        })
        .ok_or_else(|| unsupported("glTF needs default scene or exactly one scene"))?;
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    let mut seen = vec![false; parsed.nodes().len()];
    let mut stack = Vec::new();
    for node in scene.nodes() {
        if stack.len() >= MAX_NODES {
            return Err(MeshImportError::Limit("scene root nodes"));
        }
        stack.push((node, IDENTITY, 0usize));
    }
    while let Some((node, parent, depth)) = stack.pop() {
        if depth >= MAX_DEPTH {
            return Err(MeshImportError::Limit("node hierarchy depth"));
        }
        if seen[node.index()] {
            return Err(invalid("cyclic or multiply-parented scene hierarchy"));
        }
        seen[node.index()] = true;
        if node.weights().is_some() {
            return Err(unsupported("node morph weights"));
        }
        let local = node.transform().matrix().map(|c| c.map(f64::from));
        let world = mul(parent, local);
        let det = determinant(world);
        if !world.iter().flatten().all(|x| x.is_finite())
            || !det.is_finite()
            || det.abs() < 1e-30
            || world[0][3] != 0.
            || world[1][3] != 0.
            || world[2][3] != 0.
            || world[3][3] != 1.
        {
            return Err(invalid("nonfinite, singular or non-affine node transform"));
        }
        for child in node.children() {
            if stack.len() >= MAX_NODES {
                return Err(MeshImportError::Limit("hierarchy expansion"));
            }
            stack.push((child, world, depth + 1));
        }
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                let position = primitive
                    .get(&gltf::Semantic::Positions)
                    .ok_or_else(|| invalid("primitive missing POSITION"))?;
                if position.data_type() != gltf::accessor::DataType::F32
                    || position.dimensions() != gltf::accessor::Dimensions::Vec3
                    || position.normalized()
                {
                    return Err(unsupported("POSITION must be nonnormalized FLOAT VEC3"));
                }
                let count = position.count();
                let base = vertices.len();
                if base + count > MAX_VERTICES {
                    return Err(MeshImportError::Limit("expanded vertices"));
                }
                let reader = primitive.reader(|b| Some(buffers[b.index()].as_slice()));
                for p in reader
                    .read_positions()
                    .ok_or_else(|| invalid("POSITION cannot be decoded"))?
                {
                    let p = p.map(f64::from);
                    vertices.push(std::array::from_fn(|r| {
                        world[0][r] * p[0] + world[1][r] * p[1] + world[2][r] * p[2] + world[3][r]
                    }));
                }
                let indices: Vec<usize> = if let Some(a) = primitive.indices() {
                    if a.dimensions() != gltf::accessor::Dimensions::Scalar
                        || a.normalized()
                        || !matches!(
                            a.data_type(),
                            gltf::accessor::DataType::U8
                                | gltf::accessor::DataType::U16
                                | gltf::accessor::DataType::U32
                        )
                    {
                        return Err(unsupported("indices must be unsigned scalar"));
                    }
                    if a.count() > MAX_TRIANGLES * 3 {
                        return Err(MeshImportError::Limit("indices"));
                    }
                    reader
                        .read_indices()
                        .ok_or_else(|| invalid("indices cannot be decoded"))?
                        .into_u32()
                        .map(|i| {
                            usize::try_from(i).map_err(|_| invalid("index does not fit usize"))
                        })
                        .collect::<Result<_>>()?
                } else {
                    (0..count).collect()
                };
                if !indices.len().is_multiple_of(3) {
                    return Err(invalid("triangle index count not divisible by three"));
                }
                if triangles.len() + indices.len() / 3 > MAX_TRIANGLES {
                    return Err(MeshImportError::Limit("expanded triangles"));
                }
                for t in indices.chunks_exact(3) {
                    if t.iter().any(|i| *i >= count) {
                        return Err(invalid("primitive index out of range"));
                    }
                    let mut t = [base + t[0], base + t[1], base + t[2]];
                    if det < 0.0 {
                        t.swap(1, 2);
                    }
                    triangles.push(t);
                }
            }
        }
    }
    Ok((vertices, triangles))
}
