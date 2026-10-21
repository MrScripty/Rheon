use rheon::mesh_import::{MAX_FILE_BYTES, MeshFormat, MeshImportError, MeshPlacement, import_mesh};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/collision")
        .join(name)
}
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "rheon-import-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn write(&self, name: &str, data: impl AsRef<[u8]>) -> PathBuf {
        let p = self.0.join(name);
        std::fs::write(&p, data).unwrap();
        p
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[test]
fn obj_quad_triangulation_and_explicit_placement() {
    let m = import_mesh(
        &fixture("cube.obj"),
        MeshPlacement {
            scale: 0.5,
            translation: [1., 2., 3.],
        },
    )
    .unwrap();
    assert_eq!(m.vertices.len(), 8);
    assert_eq!(m.triangles.len(), 12);
    assert_eq!(m.bounds.min, [1., 2., 3.]);
    assert_eq!(m.bounds.max, [1.5, 2.5, 3.5]);
    assert_eq!(m.source_format, MeshFormat::Obj);
}
#[test]
fn gltf_external_embedded_and_glb_apply_hierarchy_reflection() {
    for name in ["hierarchy.gltf", "embedded.gltf", "cube.glb"] {
        let m = import_mesh(&fixture(name), MeshPlacement::default()).unwrap();
        assert_eq!(m.vertices.len(), 8);
        assert_eq!(m.triangles.len(), 12);
        assert_eq!(m.bounds.min, [1., 0., 0.]);
        assert_eq!(m.bounds.max, [2., 2., 1.]);
        let volume: f64 = m
            .triangles
            .iter()
            .map(|t| {
                let a = m.vertices[t[0]];
                let b = m.vertices[t[1]];
                let c = m.vertices[t[2]];
                (a[0] * (b[1] * c[2] - b[2] * c[1])
                    + a[1] * (b[2] * c[0] - b[0] * c[2])
                    + a[2] * (b[0] * c[1] - b[1] * c[0]))
                    / 6.
            })
            .sum();
        assert!((volume - 2.).abs() < 1e-10);
    }
}
fn assert_error(path: &std::path::Path, category: &str, message: &str) {
    let error = import_mesh(path, MeshPlacement::default()).unwrap_err();
    let (actual_category, actual_message) = match error {
        MeshImportError::Invalid(s) => ("invalid", s),
        MeshImportError::Unsupported(s) => ("unsupported", s),
        MeshImportError::Limit(s) => ("limit", s.to_owned()),
        MeshImportError::Io(e) => panic!("unexpected resource I/O failure: {e}"),
    };
    assert_eq!(actual_category, category);
    assert_eq!(actual_message, message);
}
#[test]
fn obj_invalid_data_rejected() {
    let tmp = Temp::new();
    for (text, category, message) in [
        ("v NaN 0 0\n", "invalid", "nonfinite OBJ position"),
        (
            "v 0 0 0\nv 1 0 0\nv 2 0 0\nf 1 2 3\n",
            "invalid",
            "degenerate OBJ face",
        ),
        (
            "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 0 2 3\n",
            "invalid",
            "OBJ indices are nonzero",
        ),
        (
            "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 4\n",
            "invalid",
            "OBJ face index out of range at line 4",
        ),
        (
            "v 0 0 0\nv 1 0 0\nv 0.2 0.2 0\nv 0 1 0\nf 1 2 3 4\n",
            "unsupported",
            "OBJ polygon must be strictly convex and simple",
        ),
        (
            "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 1\nf 1 2 3 4\n",
            "unsupported",
            "nonplanar OBJ polygon; triangulate explicitly",
        ),
        (
            "curv 1 2 3\n",
            "unsupported",
            "OBJ record curv; only polygon geometry supported",
        ),
    ] {
        assert_error(&tmp.write("bad.obj", text), category, message);
    }
}
#[test]
fn obj_negative_indices_and_visual_resources_do_not_load() {
    let tmp = Temp::new();
    let p=tmp.write("triangle.obj","mtllib https://invalid.example/no.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf -3/1/1 -2/2/1 -1/3/1\n");
    assert_eq!(
        import_mesh(&p, MeshPlacement::default()).unwrap().triangles,
        vec![[0, 1, 2]]
    );
}
#[test]
fn placement_nonfinite_zero_negative_and_file_limit_rejected() {
    for scale in [0., -1., f64::INFINITY, f64::NAN] {
        let error = import_mesh(
            &fixture("cube.obj"),
            MeshPlacement {
                scale,
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, MeshImportError::Invalid(ref message) if message == "placement requires positive finite scale and finite translation")
        );
    }
    let tmp = Temp::new();
    let p = tmp.write("big.obj", vec![b' '; MAX_FILE_BYTES + 1]);
    assert_error(&p, "limit", "file/resource bytes");
}
fn modified_gltf(tmp: &Temp, old: &str, new: &str) -> PathBuf {
    let s = std::fs::read_to_string(fixture("hierarchy.gltf")).unwrap();
    tmp.write("asset.gltf", s.replace(old, new))
}
#[test]
fn unsafe_missing_and_remote_resource_uris_rejected() {
    let tmp = Temp::new();
    for uri in [
        "../cube.bin",
        "/tmp/cube.bin",
        "https://example.org/a.bin",
        "file:buffer.bin",
        "%2e%2e/cube.bin",
        "a\\\\b.bin",
    ] {
        let p = modified_gltf(&tmp, "cube.bin", uri);
        assert_error(
            &p,
            "unsupported",
            "buffer URI must be a plain relative local path without traversal",
        );
    }
    let missing = modified_gltf(&tmp, "cube.bin", "missing.bin");
    assert!(
        matches!(import_mesh(&missing, MeshPlacement::default()), Err(MeshImportError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound)
    );
}
#[test]
fn invalid_accessor_transform_and_features_rejected() {
    let tmp = Temp::new();
    tmp.write("cube.bin", std::fs::read(fixture("cube.bin")).unwrap());
    for (old, new, category, message) in [
        (
            "\"byteLength\": 96",
            "\"byteLength\": 4",
            "invalid",
            "accessor exceeds buffer view",
        ),
        (
            "\"count\": 8",
            "\"count\": 2000000000",
            "limit",
            "accessor count",
        ),
        (
            "[-1, 2, 1]",
            "[0, 2, 1]",
            "invalid",
            "nonfinite, singular or non-affine node transform",
        ),
        (
            "\"mesh\": 0",
            "\"mesh\": 0, \"children\": [0]",
            "invalid",
            "cyclic or multiply-parented scene hierarchy",
        ),
        (
            "\"asset\":",
            "\"extensionsUsed\": [\"KHR_draco_mesh_compression\"], \"asset\":",
            "unsupported",
            "glTF extensions (including compression)",
        ),
        (
            "\"attributes\":",
            "\"mode\": 1, \"attributes\":",
            "unsupported",
            "glTF primitive must be TRIANGLES",
        ),
    ] {
        let p = modified_gltf(&tmp, old, new);
        assert_error(&p, category, message);
    }
}
#[cfg(unix)]
#[test]
fn symlink_escape_rejected() {
    let tmp = Temp::new();
    std::os::unix::fs::symlink(fixture("cube.bin"), tmp.0.join("cube.bin")).unwrap();
    let p = tmp.write(
        "asset.gltf",
        std::fs::read(fixture("hierarchy.gltf")).unwrap(),
    );
    assert!(
        matches!(import_mesh(&p, MeshPlacement::default()), Err(MeshImportError::Io(e)) if e.kind() == std::io::ErrorKind::PermissionDenied)
    );
}
#[test]
fn malformed_glb_and_json_return_errors() {
    let tmp = Temp::new();
    for (name, data) in [
        ("bad.glb", b"glTF\x02\0\0\0\xff\xff\xff\xff".as_slice()),
        ("bad.gltf", b"{\"asset\":[]}"),
    ] {
        assert!(
            matches!(import_mesh(&tmp.write(name, data), MeshPlacement::default()), Err(MeshImportError::Invalid(message)) if message.starts_with("glTF parse: "))
        );
    }
}

#[test]
fn gltf_nonfinite_binary_and_out_of_range_indices_rejected() {
    let tmp = Temp::new();
    let path = tmp.write(
        "asset.gltf",
        std::fs::read(fixture("hierarchy.gltf")).unwrap(),
    );
    let original = std::fs::read(fixture("cube.bin")).unwrap();
    let mut bad = original.clone();
    bad[0..4].copy_from_slice(&f32::NAN.to_le_bytes());
    tmp.write("cube.bin", &bad);
    assert_error(&path, "invalid", "nonfinite transformed position");
    bad = original;
    bad[96..98].copy_from_slice(&100u16.to_le_bytes());
    tmp.write("cube.bin", &bad);
    assert_error(&path, "invalid", "primitive index out of range");
}

#[test]
fn gltf_declared_limit_and_hierarchy_depth_rejected() {
    let tmp = Temp::new();
    let path = modified_gltf(&tmp, "\"byteLength\": 168", "\"byteLength\": 1073741824");
    assert_error(&path, "limit", "total buffer bytes");
    tmp.write("cube.bin", std::fs::read(fixture("cube.bin")).unwrap());
    let source = std::fs::read_to_string(fixture("hierarchy.gltf")).unwrap();
    let nodes = (0..66)
        .map(|i| {
            if i == 65 {
                "{\"mesh\":0}".into()
            } else {
                format!("{{\"children\":[{}]}}", i + 1)
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    let start = source.find("\"nodes\": [").unwrap();
    let end = source.find(", \"scenes\"").unwrap();
    let deep = format!(
        "{}\"nodes\":[{}]{}",
        &source[..start],
        nodes,
        &source[end..]
    );
    let path = tmp.write("deep.gltf", deep);
    assert_error(&path, "limit", "node hierarchy depth");
}

#[test]
fn imports_are_repeatable_and_do_not_mutate_sources() {
    let path = fixture("cube.glb");
    let before = std::fs::read(&path).unwrap();
    for _ in 0..4 {
        let mesh = import_mesh(&path, MeshPlacement::default()).unwrap();
        assert_eq!(mesh.triangles.len(), 12);
    }
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn nonregular_primary_and_buffer_inputs_fail_without_blocking() {
    let tmp = Temp::new();
    let primary = tmp.0.join("pipe.obj");
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        &primary,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
    )
    .unwrap();
    let start = std::time::Instant::now();
    assert_error(&primary, "invalid", "mesh resource must be regular file");
    let buffer = tmp.0.join("cube.bin");
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        &buffer,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
    )
    .unwrap();
    let scene = tmp.write(
        "asset.gltf",
        std::fs::read(fixture("hierarchy.gltf")).unwrap(),
    );
    assert_error(&scene, "invalid", "buffer resource must be regular file");
    assert!(start.elapsed().as_secs() < 2);
}

#[test]
fn gltf_rotated_matrix_hierarchy_and_placement_match_analytical_points() {
    let tmp = Temp::new();
    tmp.write("cube.bin", std::fs::read(fixture("cube.bin")).unwrap());
    // Parent rotates +90 degrees about Z, then translates (2,3,4).
    // Child maps (x,y,z) to (-x,2y,z); composed world point is (2-2y,3-x,4+z).
    let path = modified_gltf(
        &tmp,
        "\"translation\": [2, 0, 0]",
        "\"matrix\": [0,1,0,0,-1,0,0,0,0,0,1,0,2,3,4,1]",
    );
    let mesh = import_mesh(
        &path,
        MeshPlacement {
            scale: 0.5,
            translation: [10., 20., 30.],
        },
    )
    .unwrap();
    assert_eq!(mesh.bounds.min, [10., 21., 32.]);
    assert_eq!(mesh.bounds.max, [11., 21.5, 32.5]);
    assert_eq!(mesh.vertices[0], [11., 21.5, 32.]);
    assert_eq!(mesh.vertices[1], [11., 21., 32.]);
    assert_eq!(mesh.vertices[2], [10., 21., 32.]);
}

#[test]
fn gltf_transform_representation_is_unambiguous_and_spec_valid() {
    let tmp = Temp::new();
    tmp.write("cube.bin", std::fs::read(fixture("cube.bin")).unwrap());
    for (replacement, category, message) in [
        (
            "\"rotation\": [0,0,0,0]",
            "invalid",
            "glTF rotation quaternion must have unit length",
        ),
        (
            "\"rotation\": [0,0,0,2]",
            "invalid",
            "glTF rotation quaternion must have unit length",
        ),
        (
            "\"matrix\": [1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1], \"translation\": [2,0,0]",
            "invalid",
            "glTF node cannot combine matrix and TRS",
        ),
        (
            "\"matrix\": [1,0,0,0,0.5,1,0,0,0,0,1,0,0,0,0,1]",
            "invalid",
            "glTF node matrix contains shear; must decompose to TRS",
        ),
        ("\"weights\": []", "unsupported", "node morph weights"),
    ] {
        let path = modified_gltf(&tmp, "\"translation\": [2, 0, 0]", replacement);
        assert_error(&path, category, message);
    }
}
