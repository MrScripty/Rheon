//! Inert, bounded loading of trusted-local CPU adapter packages.
//!
//! Package discovery never executes Python. Hashes identify bytes, not authors or
//! scientific quality. The trusted host must exclude concurrent package and
//! interpreter mutation from validation through child execution; canonical paths
//! are not a sandbox or a capability against concurrent replacement.
use rheon::GridGeometry;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};

pub(crate) const SCHEMA: &str = "rheon-pressure-adapter-v1";
pub(crate) const STAGE: &str = "sealed-mac-pressure-context-v1";
const MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_FILES: usize = 64;
const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
const MAX_PYTHON_BYTES: u64 = 128 * 1024 * 1024;
const MAX_INPUTS: usize = 32;
const MAX_CELLS: usize = 65_536;
const MAX_NATIVE_STEPS: usize = 512;

#[derive(Debug)]
pub(crate) enum AdapterError {
    Invalid(String),
    Unsupported(String),
    Unavailable(String),
}
impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, message) = match self {
            Self::Invalid(message) => ("invalid", message),
            Self::Unsupported(message) => ("unsupported", message),
            Self::Unavailable(message) => ("unavailable", message),
        };
        write!(f, "CPU adapter {kind}: {message}")
    }
}
impl std::error::Error for AdapterError {}
fn invalid(message: &str) -> AdapterError {
    AdapterError::Invalid(message.into())
}
fn unavailable(operation: &str, error: std::io::Error) -> AdapterError {
    // Fixed operation names avoid reflecting arbitrary manifest text in errors.
    AdapterError::Unavailable(format!("{operation}: {error}"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InputSemantic {
    Rhs,
    PreviousPressure,
    InitialResidual,
    X,
    Y,
    Z,
    Density,
    Dt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OutputRole {
    FullPressure,
    PressureCorrection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PriorPressure {
    Required,
    Optional,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContextPolicy {
    pub minimum_native_steps: usize,
    pub prior_pressure: PriorPressure,
}

/// Exact physical scalars and inclusive geometry bounds, not tensor metadata.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Support {
    pub density_kg_m3: f64,
    pub dynamic_viscosity_pa_s: f64,
    pub dt_s: f64,
    pub counts_min: [usize; 3],
    pub counts_max: [usize; 3],
    pub max_cells: usize,
    pub spacing_min_m: f64,
    pub spacing_max_m: f64,
    origin: String,
    cells: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileSpec {
    sha256: String,
    bytes: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Python {
    executable: String,
    sha256: String,
    bytes: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    role: OutputRole,
    dtype: String,
    layout: String,
    gauge: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    id: String,
    version: String,
    stage: String,
    python: Python,
    entrypoint: String,
    #[serde(deserialize_with = "unique_files")]
    files: BTreeMap<String, FileSpec>,
    support: Support,
    context: ContextPolicy,
    #[serde(deserialize_with = "unique_inputs")]
    inputs: BTreeMap<String, InputSemantic>,
    output: Output,
}

fn bounded_map<'de, D, V>(deserializer: D, limit: usize) -> Result<BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct Entries<V>(usize, std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> serde::de::Visitor<'de> for Entries<V> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a bounded object with unique keys")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some(key) = map.next_key::<String>()? {
                if result.len() == self.0 || result.contains_key(&key) {
                    return Err(serde::de::Error::custom(
                        "duplicate key or object entry budget exceeded",
                    ));
                }
                result.insert(key, map.next_value()?);
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(Entries(limit, std::marker::PhantomData))
}
fn unique_files<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, FileSpec>, D::Error> {
    bounded_map(d, MAX_FILES)
}
fn unique_inputs<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, InputSemantic>, D::Error> {
    bounded_map(d, MAX_INPUTS)
}

/// Validated immutable package metadata consumed by the existing process owner.
pub(crate) struct AdapterPackage {
    manifest: Manifest,
    python: PathBuf,
    entrypoint: PathBuf,
    manifest_sha256: String,
}
impl AdapterPackage {
    /// Read and verify all declared material without starting an interpreter.
    pub fn load(path: &Path) -> Result<Self, AdapterError> {
        let path = path
            .canonicalize()
            .map_err(|e| unavailable("resolve manifest", e))?;
        let mut raw = Vec::new();
        // Reject special files before open, which could block on a FIFO. The
        // trusted-host no-mutation requirement covers this check/open interval.
        if !std::fs::metadata(&path)
            .map_err(|e| unavailable("inspect manifest", e))?
            .is_file()
        {
            return Err(invalid("manifest must be a regular file"));
        }
        let file = File::open(&path).map_err(|e| unavailable("open manifest", e))?;
        file.take(MANIFEST_BYTES + 1)
            .read_to_end(&mut raw)
            .map_err(|e| unavailable("read manifest", e))?;
        if raw.len() > MANIFEST_BYTES as usize {
            return Err(invalid("manifest exceeds 1 MiB"));
        }
        let manifest: Manifest = serde_json::from_slice(&raw)
            .map_err(|_| invalid("malformed or non-closed manifest"))?;
        validate_manifest(&manifest)?;
        let root = path
            .parent()
            .ok_or_else(|| invalid("manifest parent missing"))?;
        let python = Path::new(&manifest.python.executable)
            .canonicalize()
            .map_err(|e| unavailable("resolve interpreter", e))?;
        verify_file(&python, manifest.python.bytes, &manifest.python.sha256)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if std::fs::metadata(&python)
                .map_err(|e| unavailable("inspect interpreter", e))?
                .permissions()
                .mode()
                & 0o111
                == 0
            {
                return Err(invalid("interpreter is not executable"));
            }
        }
        let mut identities = BTreeSet::new();
        for (relative, spec) in &manifest.files {
            let absolute = root
                .join(relative)
                .canonicalize()
                .map_err(|e| unavailable("resolve package file", e))?;
            if !absolute.starts_with(root) || !identities.insert(absolute.clone()) {
                return Err(invalid("package file escapes root or aliases another file"));
            }
            if absolute == path || absolute == python {
                return Err(invalid("inventory cannot contain manifest or interpreter"));
            }
            verify_file(&absolute, spec.bytes, &spec.sha256)?;
        }
        let entrypoint = root
            .join(&manifest.entrypoint)
            .canonicalize()
            .map_err(|e| unavailable("resolve entrypoint", e))?;
        Ok(Self {
            manifest,
            python,
            entrypoint,
            manifest_sha256: format!("{:x}", Sha256::digest(&raw)),
        })
    }
    pub fn python(&self) -> &Path {
        &self.python
    }
    pub fn entrypoint(&self) -> &Path {
        &self.entrypoint
    }
    pub fn id(&self) -> &str {
        &self.manifest.id
    }
    pub fn version(&self) -> &str {
        &self.manifest.version
    }
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }
    pub fn support(&self) -> &Support {
        &self.manifest.support
    }
    pub fn context(&self) -> &ContextPolicy {
        &self.manifest.context
    }
    pub fn inputs(&self) -> &BTreeMap<String, InputSemantic> {
        &self.manifest.inputs
    }
    pub fn output_role(&self) -> OutputRole {
        self.manifest.output.role
    }

    /// Check the host-owned physical request before constructing adapter inputs.
    pub fn validate_support(
        &self,
        grid: &GridGeometry,
        rho: f64,
        mu: f64,
        dt: f64,
        completed_steps: usize,
        has_prior: bool,
    ) -> Result<(), AdapterError> {
        let s = self.support();
        let counts = grid.counts();
        let spacing = grid.spacing();
        if rho != s.density_kg_m3
            || mu != s.dynamic_viscosity_pa_s
            || dt != s.dt_s
            || grid.origin() != [0.0; 3]
            || spacing[0] != spacing[1]
            || spacing[0] != spacing[2]
            || spacing[0] < s.spacing_min_m
            || spacing[0] > s.spacing_max_m
            || grid.cell_len() > s.max_cells
            || (0..3).any(|d| counts[d] < s.counts_min[d] || counts[d] > s.counts_max[d])
        {
            return Err(AdapterError::Unsupported(
                "physical request is outside declared support".into(),
            ));
        }
        if completed_steps < self.context().minimum_native_steps
            || (!has_prior
                && (self.context().prior_pressure == PriorPressure::Required
                    || self
                        .inputs()
                        .values()
                        .any(|v| *v == InputSemantic::PreviousPressure)))
        {
            return Err(AdapterError::Unsupported(
                "required prior pressure or native context is missing".into(),
            ));
        }
        Ok(())
    }
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn relative_file(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && !value.contains(['\\', ':'])
        && value
            .split('/')
            .all(|v| !v.is_empty() && v != "." && v != "..")
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'/'))
        && Path::new(value)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
fn validate_manifest(m: &Manifest) -> Result<(), AdapterError> {
    if m.schema != SCHEMA || m.stage != STAGE {
        return Err(AdapterError::Unsupported(
            "schema or physical stage version".into(),
        ));
    }
    if !identifier(&m.id) || !identifier(&m.version) {
        return Err(invalid("id/version must be bounded ASCII identifiers"));
    }
    if m.python.executable.len() > 4096
        || !Path::new(&m.python.executable).is_absolute()
        || !digest(&m.python.sha256)
        || m.python.bytes == 0
        || m.python.bytes > MAX_PYTHON_BYTES
    {
        return Err(invalid("invalid interpreter identity or size"));
    }
    if !relative_file(&m.entrypoint) || !m.files.contains_key(&m.entrypoint) || m.files.is_empty() {
        return Err(invalid(
            "entrypoint must be a measured relative inventory file",
        ));
    }
    let mut total = 0_u64;
    for (path, file) in &m.files {
        if !relative_file(path) || !digest(&file.sha256) || file.bytes > MAX_FILE_BYTES {
            return Err(invalid("invalid inventory path, digest or file size"));
        }
        total = total
            .checked_add(file.bytes)
            .ok_or_else(|| invalid("inventory length overflow"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(invalid("inventory exceeds total file byte budget"));
        }
    }
    let s = &m.support;
    if !s.density_kg_m3.is_finite()
        || s.density_kg_m3 <= 0.0
        || !s.dynamic_viscosity_pa_s.is_finite()
        || s.dynamic_viscosity_pa_s < 0.0
        || !s.dt_s.is_finite()
        || s.dt_s <= 0.0
        || !s.spacing_min_m.is_finite()
        || s.spacing_min_m <= 0.0
        || !s.spacing_max_m.is_finite()
        || s.spacing_max_m < s.spacing_min_m
        || s.max_cells == 0
        || s.max_cells > MAX_CELLS
        || (0..3).any(|d| {
            s.counts_min[d] == 0 || s.counts_min[d] > s.counts_max[d] || s.counts_max[d] > MAX_CELLS
        })
    {
        return Err(invalid("invalid physical support bounds"));
    }
    let minimum_cells = s
        .counts_min
        .into_iter()
        .try_fold(1_usize, |n, d| n.checked_mul(d));
    if minimum_cells.is_none_or(|n| n > s.max_cells) {
        return Err(invalid("minimum geometry exceeds cell budget"));
    }
    if s.origin != "zero" || s.cells != "cubic" {
        return Err(AdapterError::Unsupported(
            "only zero-origin cubic cells are supported".into(),
        ));
    }
    if m.context.minimum_native_steps > MAX_NATIVE_STEPS
        || m.inputs.is_empty()
        || m.inputs.keys().any(|key| !identifier(key))
    {
        return Err(invalid("invalid native context or named input budget"));
    }
    if m.output.dtype != "f64" || m.output.layout != "x_fast" || m.output.gauge != "cell_zero" {
        return Err(AdapterError::Unsupported(
            "output requires f64, x_fast layout and cell_zero gauge".into(),
        ));
    }
    if m.output.role == OutputRole::PressureCorrection
        && m.context.prior_pressure != PriorPressure::Required
    {
        return Err(invalid("pressure correction requires prior pressure"));
    }
    Ok(())
}

fn verify_file(path: &Path, bytes: u64, expected: &str) -> Result<(), AdapterError> {
    if !std::fs::metadata(path)
        .map_err(|e| unavailable("inspect measured file", e))?
        .is_file()
    {
        return Err(invalid("measured file must be regular"));
    }
    let file = File::open(path).map_err(|e| unavailable("open measured file", e))?;
    let metadata = file
        .metadata()
        .map_err(|e| unavailable("inspect measured file", e))?;
    if !metadata.is_file() || metadata.len() != bytes {
        return Err(invalid("measured file type or length drift"));
    }
    let mut read = file.take(bytes + 1);
    let mut hash = Sha256::new();
    let mut count = 0_u64;
    let mut buffer = [0_u8; 65_536];
    loop {
        let n = read
            .read(&mut buffer)
            .map_err(|e| unavailable("read measured file", e))?;
        if n == 0 {
            break;
        }
        count += n as u64;
        hash.update(&buffer[..n]);
    }
    if count != bytes || format!("{:x}", hash.finalize()) != expected {
        return Err(invalid("measured file length or digest drift"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::sync::atomic::{AtomicU64, Ordering};

    static SERIAL: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        root: PathBuf,
        value: Value,
    }
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "rheon-adapter-{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&root).unwrap();
            let python = root.join("python");
            // If invoked this fixture interpreter would leave an observable file.
            let interpreter = format!(
                "#!/bin/sh\ntouch '{}'\nexit 1\n",
                root.join("executed").display()
            );
            std::fs::write(&python, &interpreter).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&python, std::fs::Permissions::from_mode(0o700)).unwrap();
            }
            let entrypoint = b"raise RuntimeError('discovery must be inert')\n";
            std::fs::write(root.join("adapter.py"), entrypoint).unwrap();
            let value = json!({
                "schema": SCHEMA, "id": "fixture", "version": "1.0", "stage": STAGE,
                "python": {"executable": python, "sha256": format!("{:x}", Sha256::digest(interpreter.as_bytes())), "bytes": interpreter.len()},
                "entrypoint": "adapter.py", "files": {"adapter.py": {"sha256": format!("{:x}", Sha256::digest(entrypoint)), "bytes": entrypoint.len()}},
                "support": {"density_kg_m3": 1000.0, "dynamic_viscosity_pa_s": 0.001, "dt_s": 1.0/60.0,
                    "counts_min": [5,5,5], "counts_max": [128,128,128], "max_cells": 65536,
                    "spacing_min_m": 0.01, "spacing_max_m": 1.0, "origin": "zero", "cells": "cubic"},
                "context": {"minimum_native_steps": 15, "prior_pressure": "required"},
                "inputs": {"custom_rhs": "rhs", "prior": "previous_pressure"},
                "output": {"role": "full_pressure", "dtype": "f64", "layout": "x_fast", "gauge": "cell_zero"}
            });
            Self { root, value }
        }
        fn load(&self) -> Result<AdapterPackage, AdapterError> {
            let path = self.root.join("manifest.json");
            std::fs::write(&path, serde_json::to_vec(&self.value).unwrap()).unwrap();
            AdapterPackage::load(&path)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    fn grid(counts: [u64; 3], spacing: [f64; 3], origin: [f64; 3]) -> GridGeometry {
        GridGeometry::new(counts, spacing, origin).unwrap()
    }

    #[test]
    fn discovery_verifies_material_without_executing_and_accepts_unequal_counts() {
        let f = Fixture::new();
        let p = f.load().unwrap();
        assert_eq!(p.id(), "fixture");
        assert_eq!(p.version(), "1.0");
        assert_eq!(p.inputs()["custom_rhs"], InputSemantic::Rhs);
        assert_eq!(p.output_role(), OutputRole::FullPressure);
        assert_eq!(p.manifest_sha256().len(), 64);
        assert!(!f.root.join("executed").exists());
        assert!(
            p.validate_support(
                &grid([5, 6, 7], [0.1; 3], [0.0; 3]),
                1000.0,
                0.001,
                1.0 / 60.0,
                15,
                true
            )
            .is_ok()
        );
    }
    #[test]
    fn physical_and_history_mismatches_are_unsupported() {
        let f = Fixture::new();
        let p = f.load().unwrap();
        for g in [
            grid([5; 3], [0.1, 0.2, 0.1], [0.0; 3]),
            grid([5; 3], [0.1; 3], [1.0, 0.0, 0.0]),
            grid([4, 5, 5], [0.1; 3], [0.0; 3]),
            grid([41; 3], [0.1; 3], [0.0; 3]),
            grid([5; 3], [0.001; 3], [0.0; 3]),
        ] {
            assert!(matches!(
                p.validate_support(&g, 1000.0, 0.001, 1.0 / 60.0, 15, true),
                Err(AdapterError::Unsupported(_))
            ));
        }
        let g = grid([5; 3], [0.1; 3], [0.0; 3]);
        for (rho, mu, dt, steps, prior) in [
            (999.0, 0.001, 1.0 / 60.0, 15, true),
            (1000.0, 0.0, 1.0 / 60.0, 15, true),
            (1000.0, 0.001, 0.02, 15, true),
            (1000.0, 0.001, 1.0 / 60.0, 14, true),
            (1000.0, 0.001, 1.0 / 60.0, 15, false),
            (f64::NAN, 0.001, 1.0 / 60.0, 15, true),
        ] {
            assert!(matches!(
                p.validate_support(&g, rho, mu, dt, steps, prior),
                Err(AdapterError::Unsupported(_))
            ));
        }
    }
    #[test]
    fn closed_records_and_unknown_contracts_fail() {
        for (pointer, value) in [
            ("/schema", json!("rheon-pressure-adapter-v2")),
            ("/stage", json!("free-surface")),
            ("/support/origin", json!("arbitrary")),
            ("/output/layout", json!("z_fast")),
            ("/inputs/custom_rhs", json!("velocity")),
        ] {
            let mut f = Fixture::new();
            *f.value.pointer_mut(pointer).unwrap() = value;
            assert!(f.load().is_err(), "{pointer}");
        }
        for pointer in [
            "",
            "/python",
            "/support",
            "/context",
            "/output",
            "/files/adapter.py",
        ] {
            let mut f = Fixture::new();
            f.value
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".into(), json!(true));
            assert!(matches!(f.load(), Err(AdapterError::Invalid(_))));
        }
    }
    #[test]
    fn traversal_unmeasured_entrypoint_and_invalid_budgets_fail() {
        for path in [
            "../adapter.py",
            "/adapter.py",
            "a/../adapter.py",
            "./adapter.py",
            "a\\adapter.py",
            "a//adapter.py",
            "C:adapter.py",
        ] {
            assert!(!relative_file(path));
            let mut f = Fixture::new();
            let spec = f.value["files"]["adapter.py"].take();
            f.value["files"] = json!({path:spec});
            f.value["entrypoint"] = json!(path);
            assert!(matches!(f.load(), Err(AdapterError::Invalid(_))));
        }
        for (pointer, value) in [
            ("/entrypoint", json!("missing.py")),
            ("/support/max_cells", json!(65537)),
            ("/support/counts_min", json!([128, 128, 128])),
            ("/context/minimum_native_steps", json!(513)),
            ("/files/adapter.py/bytes", json!(MAX_FILE_BYTES + 1)),
            ("/python/executable", json!("python")),
            ("/python/sha256", json!("0")),
            ("/support/dt_s", json!(0.0)),
            ("/id", json!("bad id")),
        ] {
            let mut f = Fixture::new();
            *f.value.pointer_mut(pointer).unwrap() = value;
            assert!(
                matches!(f.load(), Err(AdapterError::Invalid(_))),
                "{pointer}"
            );
        }
    }
    #[test]
    fn digest_and_length_drift_are_rejected() {
        for file in ["adapter.py", "python"] {
            let f = Fixture::new();
            let path = f.root.join(file);
            let original = std::fs::read(&path).unwrap();
            let mut same_length = original.clone();
            same_length[0] ^= 1;
            std::fs::write(&path, same_length).unwrap();
            assert!(matches!(f.load(), Err(AdapterError::Invalid(_))));
            std::fs::write(&path, b"short").unwrap();
            assert!(matches!(f.load(), Err(AdapterError::Invalid(_))));
        }
    }
    #[test]
    fn duplicate_keys_and_entry_limits_fail() {
        let f = Fixture::new();
        let path = f.root.join("manifest.json");
        let raw = serde_json::to_string(&f.value).unwrap().replace(
            "\"custom_rhs\":\"rhs\"",
            "\"custom_rhs\":\"rhs\",\"custom_rhs\":\"dt\"",
        );
        std::fs::write(&path, raw).unwrap();
        assert!(matches!(
            AdapterPackage::load(&path),
            Err(AdapterError::Invalid(_))
        ));
        for (field, limit) in [("files", MAX_FILES), ("inputs", MAX_INPUTS)] {
            let mut f = Fixture::new();
            let map = f.value[field].as_object_mut().unwrap();
            for n in 0..limit {
                map.insert(
                    format!("port{n}"),
                    if field == "inputs" {
                        json!("rhs")
                    } else {
                        json!({"sha256":"0".repeat(64),"bytes":0})
                    },
                );
            }
            assert!(matches!(f.load(), Err(AdapterError::Invalid(_))));
        }
        std::fs::write(&path, vec![b' '; MANIFEST_BYTES as usize + 1]).unwrap();
        assert!(matches!(
            AdapterPackage::load(&path),
            Err(AdapterError::Invalid(_))
        ));
    }
    #[cfg(unix)]
    #[test]
    fn escaping_and_aliased_symlink_inventory_is_rejected() {
        use std::os::unix::fs::symlink;
        let mut f = Fixture::new();
        let outside = Fixture::new();
        symlink(outside.root.join("adapter.py"), f.root.join("escape.py")).unwrap();
        f.value["files"]["escape.py"] = f.value["files"]["adapter.py"].clone();
        assert!(matches!(f.load(), Err(AdapterError::Invalid(_))));
        f.value["files"]
            .as_object_mut()
            .unwrap()
            .remove("escape.py");
        symlink(f.root.join("adapter.py"), f.root.join("alias.py")).unwrap();
        f.value["files"]["alias.py"] = f.value["files"]["adapter.py"].clone();
        assert!(matches!(f.load(), Err(AdapterError::Invalid(_))));
    }
    #[test]
    fn correction_and_selected_optional_prior_require_context() {
        let mut f = Fixture::new();
        f.value["context"]["prior_pressure"] = json!("optional");
        let p = f.load().unwrap();
        assert!(matches!(
            p.validate_support(
                &grid([5; 3], [0.1; 3], [0.0; 3]),
                1000.0,
                0.001,
                1.0 / 60.0,
                15,
                false
            ),
            Err(AdapterError::Unsupported(_))
        ));
        f.value["output"]["role"] = json!("pressure_correction");
        assert!(matches!(f.load(), Err(AdapterError::Invalid(_))));
        f.value["context"]["prior_pressure"] = json!("required");
        assert_eq!(
            f.load().unwrap().output_role(),
            OutputRole::PressureCorrection
        );
    }
}
