//! Fresh bounded native I1 capture. Records are verification output, never a dataset/checkpoint.
#[path = "../experiments/submarine_stokes.rs"]
// The opt-in owner also exposes inspection APIs consumed by its contract tests.
#[allow(dead_code)]
mod stokes;

use gltf::json::{Value, deserialize::from_slice, serialize::to_vec};
use rheon::{GridGeometry, StaticObstacleGeometry, SurfaceSettings, SurfaceStamp, TriangleSurface};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, error::Error, fs, io::Write, path::Path};

const MAX_RECORD_BYTES: usize = 16 * 1024 * 1024;
const PAYLOAD_LIMIT: usize = 256 * 1024 * 1024;
// Parent-admitted v2 fixtures from validator dedf04fb0d094064f5a163a4f7b78e7237e4438f.
const FIXTURES_SHA256: &str = "3762540394238820ad47ca7ce4f8fc5b140e11abc0d52458a2451a4fe588f160";

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn object<const N: usize>(entries: [(&str, Value); N]) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect(),
    )
}
fn array(values: impl IntoIterator<Item = Value>) -> Value {
    Value::Array(values.into_iter().collect())
}
fn text(value: &str) -> Value {
    Value::String(value.to_owned())
}
fn bits(value: f64) -> Value {
    text(&format!("{:016x}", value.to_bits()))
}
fn vector(value: [f64; 3]) -> Value {
    array(value.into_iter().map(bits))
}
fn matrix(value: [[f64; 3]; 3]) -> Value {
    array(value.into_iter().map(vector))
}
fn number(value: usize) -> Value {
    Value::from(value as u64)
}
fn field(value: &[f64]) -> Value {
    array(value.iter().copied().map(bits))
}
// Explicitly sort recursively, including when a dependency enables insertion-order maps.
fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(entries) => {
            let entries: BTreeMap<_, _> = entries
                .iter()
                .map(|(k, v)| (k.clone(), sorted(v)))
                .collect();
            Value::Object(entries.into_iter().collect())
        }
        Value::Array(values) => array(values.iter().map(sorted)),
        _ => value.clone(),
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn member<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    value
        .get(key)
        .ok_or_else(|| format!("missing fixture member {key}").into())
}
fn finite(value: &Value) -> Result<f64> {
    let n: f64 = value
        .as_str()
        .ok_or("fixture physical value must be a decimal string")?
        .parse()?;
    if n.is_finite() {
        Ok(n)
    } else {
        Err("nonfinite fixture value".into())
    }
}
fn triple(value: &Value) -> Result<[f64; 3]> {
    let values = value.as_array().ok_or("expected vector")?;
    if values.len() != 3 {
        return Err("expected three components".into());
    }
    Ok([
        finite(&values[0])?,
        finite(&values[1])?,
        finite(&values[2])?,
    ])
}
fn integer(value: &Value) -> Result<usize> {
    Ok(usize::try_from(
        value.as_u64().ok_or("expected unsigned integer")?,
    )?)
}
fn counts(value: &Value) -> Result<[usize; 3]> {
    let v = value.as_array().ok_or("expected counts")?;
    if v.len() != 3 {
        return Err("expected three counts".into());
    }
    Ok([integer(&v[0])?, integer(&v[1])?, integer(&v[2])?])
}
fn gradients(value: &Value) -> Result<[[f64; 3]; 3]> {
    let v = value.as_array().ok_or("expected gradient")?;
    if v.len() != 3 {
        return Err("expected three gradient rows".into());
    }
    Ok([triple(&v[0])?, triple(&v[1])?, triple(&v[2])?])
}

fn admitted_fixtures(path: &Path) -> Result<Value> {
    let mut file = fs::File::open(path)?;
    use std::io::Read;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take((MAX_RECORD_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_RECORD_BYTES {
        return Err("fixture exceeds bounded input cap".into());
    }
    if hash(&bytes) != FIXTURES_SHA256 {
        return Err("fixture bytes do not match the admitted corrected sector contract".into());
    }
    let fixture: Value = from_slice(&bytes)?;
    if member(&fixture, "schema")?.as_str() != Some("rheon-submarine-i1-fixtures-v2") {
        return Err("unsupported fixture schema".into());
    }
    if member(&fixture, "contract_status")?.as_str()
        != Some("parent-admitted-sector-correction-native-qualification-pending")
    {
        return Err("fixture profile lacks parent admission".into());
    }
    Ok(fixture)
}

fn write_record(path: &Path, value: &Value) -> Result<()> {
    let bytes = to_vec(value)?;
    if bytes.len() > MAX_RECORD_BYTES {
        return Err("native record exceeds bounded output cap".into());
    }
    let tmp = path.with_extension("json.partial");
    let mut out = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&tmp)?;
    let result = (|| -> Result<()> {
        out.write_all(&bytes)?;
        out.sync_all()?;
        drop(out);
        // Output directory is fresh and exclusively owned by this invocation.
        fs::rename(&tmp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("native I1 capture refused: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut fixtures = None;
    let mut case = None;
    let mut output = None;
    while let Some(arg) = args.next() {
        let value = args.next().ok_or("option missing value")?;
        match arg.as_str() {
            "--fixtures" if fixtures.is_none() => fixtures = Some(value),
            "--case" if case.is_none() => case = Some(value),
            "--output" if output.is_none() => output = Some(value),
            _ => return Err("unknown or duplicate CLI option".into()),
        }
    }
    let fixtures = admitted_fixtures(Path::new(&fixtures.ok_or("--fixtures is required")?))?;
    let case = case.ok_or("--case is required")?;
    let output = output.ok_or("--output is required")?;
    let fixture = member(member(&fixtures, "cases")?, &case)?;
    let output = Path::new(&output);
    let name = output
        .file_name()
        .ok_or("output requires a fresh directory name")?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let checkout = fs::canonicalize(env!("CARGO_MANIFEST_DIR"))?;
    // The exact-source runner passes a fresh cases/CASE path. Establish its
    // existing ancestor before creating parent directories; never create them
    // below the checkout or interpret unresolved parent traversal.
    if parent
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("capture output must not contain parent traversal".into());
    }
    let mut ancestor = parent;
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or("output has no existing ancestor")?;
    }
    if fs::canonicalize(ancestor)?.starts_with(&checkout) {
        return Err("capture output must be outside the checkout".into());
    }
    fs::create_dir_all(parent)?;
    let parent = fs::canonicalize(parent)?;
    if parent.starts_with(&checkout) {
        return Err("capture output must be outside the checkout".into());
    }
    let output = parent.join(name);
    fs::create_dir(&output)?;
    let record = capture(&case, fixture, &fixtures)?;
    write_record(&output.join("record.json"), &record)?;
    println!("accepted native steady solve 1: {case}");
    Ok(())
}

fn config(fixture: &Value) -> Result<stokes::Config> {
    let lower = member(fixture, "body_lower")?;
    let upper = member(fixture, "body_upper")?;
    let body = if lower.is_null() && upper.is_null() {
        None
    } else if lower.is_null() || upper.is_null() {
        return Err("incomplete body bounds".into());
    } else {
        Some(stokes::Cuboid {
            lower: triple(lower)?,
            upper: triple(upper)?,
        })
    };
    Ok(stokes::Config {
        counts: counts(member(fixture, "counts")?)?,
        origin: triple(member(fixture, "origin")?)?,
        spacing: triple(member(fixture, "spacing")?)?,
        body,
        density: finite(member(fixture, "rho")?)?,
        viscosity: finite(member(fixture, "mu")?)?,
        u_ref: finite(member(fixture, "U_ref")?)?,
        l_ref: finite(member(fixture, "L_ref")?)?,
        dof_limit: integer(member(fixture, "max_dofs")?)?,
        payload_limit: PAYLOAD_LIMIT,
    })
}

fn request(fixture: &Value) -> Result<stokes::Request> {
    let boundary = match member(fixture, "boundary")?.as_str() {
        Some("uniform-dirichlet") => {
            stokes::AnalyticField::Uniform(triple(member(fixture, "velocity")?)?)
        }
        Some("polynomial-mms-dirichlet") => stokes::AnalyticField::PolynomialMms {
            amplitude: finite(member(fixture, "mms_amplitude")?)?,
            pressure_linear: triple(member(fixture, "pressure_linear")?)?,
            pressure_quadratic: triple(member(fixture, "pressure_quadratic")?)?,
        },
        _ => return Err("unsupported fixture boundary".into()),
    };
    Ok(stokes::Request {
        boundary,
        pressure_offset: finite(member(fixture, "pressure_gauge")?)?,
        moment_reference: triple(member(fixture, "moment_reference")?)?,
    })
}

fn obstacle(config: stokes::Config) -> Result<Option<StaticObstacleGeometry>> {
    let Some(body) = config.body else {
        return Ok(None);
    };
    let grid = GridGeometry::new(
        config.counts.map(|n| n as u64),
        config.spacing,
        config.origin,
    )?;
    // Original closed cuboid mesh; no external hull data or tessellation dependency.
    let vertices = (0..8)
        .map(|corner| {
            std::array::from_fn(|a| {
                if corner & (1 << a) == 0 {
                    body.lower[a]
                } else {
                    body.upper[a]
                }
            })
        })
        .collect();
    let triangles = vec![
        [0, 2, 3],
        [0, 3, 1],
        [4, 5, 7],
        [4, 7, 6],
        [0, 1, 5],
        [0, 5, 4],
        [2, 6, 7],
        [2, 7, 3],
        [0, 4, 6],
        [0, 6, 2],
        [1, 3, 7],
        [1, 7, 5],
    ];
    let surface = TriangleSurface::new(
        SurfaceStamp { id: 1, version: 1 },
        vertices,
        triangles,
        SurfaceSettings::default(),
    )?;
    Ok(Some(StaticObstacleGeometry::new(
        grid,
        surface,
        PAYLOAD_LIMIT,
        |_, _| false,
    )?))
}

fn variable_key(key: stokes::VariableKey) -> String {
    match key {
        stokes::VariableKey::Face {
            axis,
            coordinates: [i, j, k],
        } => format!("face:{axis}:{i}:{j}:{k}"),
        stokes::VariableKey::Ghost {
            axes: [a, b],
            coordinates: [i, j, k],
            component,
        } => format!("ghost:{a}:{b}:{i}:{j}:{k}:{component}"),
        stokes::VariableKey::Corner {
            axes: [a, b],
            coordinates: [i, j, k],
            component,
        } => format!("corner:{a}:{b}:{i}:{j}:{k}:{component}"),
    }
}

fn traces(traces: &[stokes::BoundaryTrace], outer: bool) -> Result<Value> {
    // Several prescribed scalar variables may share one full-vector trace.
    let mut unique = BTreeMap::new();
    for trace in traces {
        let point = trace.point.map(f64::to_bits);
        let (axis, side) = match trace.owner {
            stokes::Owner::Outer { axis, side } if outer => (axis, side),
            stokes::Owner::Body if !outer => (0, 0),
            _ => return Err("core trace owner mismatch".into()),
        };
        let velocity = trace.velocity.map(f64::to_bits);
        if let Some(old) = unique.insert((axis, side, point), velocity)
            && old != velocity
        {
            return Err("inconsistent native boundary trace".into());
        }
    }
    Ok(array(unique.into_iter().map(
        |((axis, side, point), velocity)| {
            let point = vector(point.map(f64::from_bits));
            let velocity = vector(velocity.map(f64::from_bits));
            if outer {
                object([
                    ("axis", number(axis)),
                    ("side", Value::from(i64::from(side))),
                    ("point", point),
                    ("velocity", velocity),
                ])
            } else {
                object([("point", point), ("velocity", velocity)])
            }
        },
    )))
}

fn wall_patch(patch: &stokes::WallPatch) -> Value {
    object([
        ("cell", number(patch.cell)),
        ("axis", number(patch.axis)),
        ("side", Value::from(i64::from(patch.side))),
        ("point", vector(patch.point)),
        ("normal", vector(patch.normal)),
        ("area", bits(patch.area)),
        ("velocity", vector(patch.velocity)),
        ("pressure", bits(patch.pressure)),
        ("gradient", matrix(patch.gradient)),
    ])
}

fn physical_loads(loads: &stokes::Loads) -> Value {
    object([
        ("pressure_force", vector(loads.pressure_force)),
        ("shear_force", vector(loads.shear_force)),
        ("pressure_moment", vector(loads.pressure_moment)),
        ("shear_moment", vector(loads.shear_moment)),
    ])
}

fn capture(case: &str, fixture: &Value, fixtures: &Value) -> Result<Value> {
    let config = config(fixture)?;
    let geometry_owner = obstacle(config)?;
    let mut solver = stokes::Solver::new(config, geometry_owner.as_ref())?;
    solver.solve(request(fixture)?, |_, _| false)?;
    let solution = solver.accepted().ok_or("native solve did not commit")?;
    let cells = solution.pressure.len();
    let geometry = object([
        ("counts", array(config.counts.into_iter().map(number))),
        ("origin", vector(config.origin)),
        ("spacing", vector(config.spacing)),
        (
            "body_lower",
            config.body.map_or(Value::Null, |b| vector(b.lower)),
        ),
        (
            "body_upper",
            config.body.map_or(Value::Null, |b| vector(b.upper)),
        ),
        (
            "cell_mask",
            array((0..cells).map(|i| number(usize::from(solver.is_fluid(i))))),
        ),
        (
            "face_masks",
            array(
                (0..3).map(|a| array(solver.face_masks(a).iter().map(|&v| number(usize::from(v))))),
            ),
        ),
        (
            "face_dual_volumes",
            array((0..3).map(|a| field(solver.face_dual_volumes(a)))),
        ),
        (
            "pressure_components",
            array((0..cells).map(|i| Value::from(if solver.is_fluid(i) { 0i64 } else { -1i64 }))),
        ),
        (
            "component_gauges",
            array([object([
                ("component", number(0)),
                ("cell", number(solver.gauge_cell())),
                ("pressure", bits(solution.pressure[solver.gauge_cell()])),
            ])]),
        ),
    ]);
    let geometry_sha256 = hash(&to_vec(&sorted(&geometry))?);
    if solution.prescribed.len() != solution.reactions.len() {
        return Err("incomplete native reactions".into());
    }
    let mut reactions = BTreeMap::new();
    let mut reaction_sites = BTreeMap::new();
    for (prescribed, &reaction) in solution.prescribed.iter().zip(&solution.reactions) {
        let key = variable_key(prescribed.key);
        let owner = match prescribed.owner {
            stokes::Owner::Body => "body",
            stokes::Owner::Outer { .. } => "outer",
        };
        reaction_sites.insert(
            key.clone(),
            object([
                ("component", number(prescribed.component)),
                ("owner", text(owner)),
                ("point", vector(prescribed.point)),
            ]),
        );
        if reactions.insert(key, bits(reaction)).is_some() {
            return Err("duplicate native reaction key".into());
        }
    }
    let r = &solution.report;
    Ok(object([
        ("schema", text("rheon-submarine-i1-mac-v2")),
        ("operator_profile", text(stokes::CORNER_RULE)),
        ("fixture_id", text(case)),
        ("fixtures_sha256", text(FIXTURES_SHA256)),
        ("controls", fixture.clone()),
        (
            "discretization",
            member(fixtures, "discretization")?.clone(),
        ),
        ("accepted", Value::Bool(true)),
        ("solve_id", Value::from(r.solve_id)),
        ("equations", text("steady-incompressible-newtonian-stokes")),
        ("frame", text("fixed-right-handed-SI")),
        (
            "units",
            object([
                ("velocity", text("m/s")),
                ("pressure", text("Pa")),
                ("gradient", text("1/s")),
                ("force", text("N")),
                ("moment", text("N m")),
                ("volume_forcing", text("N/m^3")),
            ]),
        ),
        (
            "layout",
            object([
                ("ordering", text("x-fast")),
                (
                    "velocity_offsets",
                    array(
                        [[0.0, 0.5, 0.5], [0.5, 0.0, 0.5], [0.5, 0.5, 0.0]]
                            .into_iter()
                            .map(vector),
                    ),
                ),
                ("pressure_offset", vector([0.5; 3])),
            ]),
        ),
        ("geometry", geometry),
        ("geometry_sha256", text(&geometry_sha256)),
        (
            "fields",
            object([
                (
                    "velocity",
                    array(solution.face_fields.iter().map(|v| field(v))),
                ),
                (
                    "volume_forcing",
                    array(solution.volume_forcing.iter().map(|v| field(v))),
                ),
                ("pressure", field(&solution.pressure)),
            ]),
        ),
        ("forcing_semantics", member(fixture, "forcing")?.clone()),
        ("outer_traces", traces(&solution.outer_traces, true)?),
        ("body_traces", traces(&solution.body_traces, false)?),
        (
            "wall_patches",
            array(solution.wall_patches.iter().map(wall_patch)),
        ),
        (
            "algebraic_reactions",
            Value::Object(reactions.into_iter().collect()),
        ),
        (
            "reaction_sites",
            Value::Object(reaction_sites.into_iter().collect()),
        ),
        (
            "algebraic_loads",
            object([
                ("body_force", vector(solution.loads.body_force)),
                ("outer_force", vector(solution.loads.outer_force)),
                ("body_moment", vector(solution.loads.body_moment)),
                ("outer_moment", vector(solution.loads.outer_moment)),
            ]),
        ),
        ("physical_loads", physical_loads(&solution.loads)),
        ("dissipation", bits(r.dissipation)),
        (
            "solve_report",
            object([
                ("backend", text("dense-scaled-partial-pivot")),
                ("actual_dofs", number(r.actual_dofs)),
                ("active_velocity_dofs", number(r.active_velocity_dofs)),
                ("nongauge_pressure_dofs", number(r.nongauge_pressure_dofs)),
                ("min_scaled_pivot", bits(r.min_scaled_pivot)),
                ("retained_payload_bytes", number(r.retained_payload_bytes)),
                ("payload_limit_bytes", number(r.payload_limit_bytes)),
                ("max_momentum_defect", bits(r.max_momentum_defect)),
                ("max_divergence_defect", bits(r.max_divergence_defect)),
                ("max_residual_ratio", bits(r.max_residual_ratio)),
                ("body_work", bits(r.body_work)),
                ("outer_work", bits(r.outer_work)),
                ("forcing_work", bits(r.forcing_work)),
                ("work_defect", bits(r.work_defect)),
                ("work_budget", bits(r.work_budget)),
            ]),
        ),
        ("constitutive_controls", constitutive_controls(fixtures)?),
    ]))
}

fn constitutive_controls(fixtures: &Value) -> Result<Value> {
    let presets = member(fixtures, "control_presets")?
        .as_object()
        .ok_or("expected frozen control presets")?;
    if presets.len() != 34 {
        return Err("expected complete 34 frozen constitutive controls".into());
    }
    let lower = triple(member(fixtures, "control_body_lower")?)?;
    let upper = triple(member(fixtures, "control_body_upper")?)?;
    let references = member(fixtures, "control_references")?
        .as_array()
        .ok_or("missing frozen control references")?;
    if references.is_empty() {
        return Err("missing frozen control reference".into());
    }
    // Alternate the admitted references, exercising shifted moments as well as COM.
    let mut controls = Vec::with_capacity(presets.len());
    for (index, (id, preset)) in presets.iter().enumerate() {
        let reference = triple(&references[index % references.len()])?;
        let closed = member(preset, "closed")?
            .as_bool()
            .ok_or("control closure must be boolean")?;
        let offset = finite(member(preset, "pressure_offset")?)?;
        let slope = triple(member(preset, "pressure_gradient")?)?;
        let declared_gradient = member(preset, "analytic_gradient")?;
        let analytic_gradient = if declared_gradient.is_null() {
            None
        } else {
            Some(gradients(declared_gradient)?)
        };
        let mut patches = Vec::new();
        let mut total = stokes::Loads::default();
        for patch_index in 0..if closed { 6 } else { 2 } {
            let (axis, side, point, normal, area) = if closed {
                let axis = patch_index / 2;
                let side = if patch_index % 2 == 0 { -1 } else { 1 };
                let mut point = std::array::from_fn(|a| (lower[a] + upper[a]) / 2.0);
                point[axis] = if side < 0 { lower[axis] } else { upper[axis] };
                let mut normal = [0.0; 3];
                normal[axis] = f64::from(side);
                let area = (upper[(axis + 1) % 3] - lower[(axis + 1) % 3])
                    * (upper[(axis + 2) % 3] - lower[(axis + 2) % 3]);
                (axis, side, point, normal, area)
            } else {
                let y = patch_index as f64;
                let side = if patch_index == 0 { 1 } else { -1 };
                (1, side, [0.0, y, 0.0], [0.0, f64::from(side), 0.0], 1.0)
            };
            let gradient =
                analytic_gradient.unwrap_or([[0.0, 1.0 - 2.0 * point[1], 0.0], [0.0; 3], [0.0; 3]]);
            let velocity = if id == "couette" {
                [point[1], 0.0, 0.0]
            } else if id == "poiseuille" {
                [point[1] * (1.0 - point[1]), 0.0, 0.0]
            } else {
                std::array::from_fn(|a| (0..3).map(|b| gradient[a][b] * point[b]).sum())
            };
            let pressure = offset + (0..3).map(|a| slope[a] * point[a]).sum::<f64>();
            let patch = stokes::WallPatch {
                cell: patch_index,
                axis,
                side,
                point,
                normal,
                area,
                velocity,
                pressure,
                gradient,
            };
            let load = stokes::integrate_patch(&patch, 1.0, reference)?;
            for a in 0..3 {
                total.pressure_force[a] += load.pressure_force[a];
                total.shear_force[a] += load.shear_force[a];
                total.pressure_moment[a] += load.pressure_moment[a];
                total.shear_moment[a] += load.shear_moment[a];
            }
            patches.push(object([
                ("point", vector(point)),
                ("normal", vector(normal)),
                ("area", bits(area)),
                ("pressure", bits(pressure)),
                ("gradient", matrix(gradient)),
                ("wall_velocity", vector(velocity)),
                ("mechanical_work", bits(load.work)),
                ("pressure_force", vector(load.pressure_force)),
                ("shear_force", vector(load.shear_force)),
                ("pressure_moment", vector(load.pressure_moment)),
                ("shear_moment", vector(load.shear_moment)),
            ]));
        }
        controls.push(object([
            ("kind", text("constitutive-control")),
            ("control_id", text(id)),
            ("closed", Value::Bool(closed)),
            ("mu", bits(1.0)),
            ("moment_reference", vector(reference)),
            ("pressure_offset", bits(offset)),
            ("pressure_gradient", vector(slope)),
            (
                "analytic_gradient",
                analytic_gradient.map_or(Value::Null, matrix),
            ),
            ("patches", array(patches)),
            ("pressure_force", vector(total.pressure_force)),
            ("shear_force", vector(total.shear_force)),
            ("pressure_moment", vector(total.pressure_moment)),
            ("shear_moment", vector(total.shear_moment)),
        ]));
    }
    Ok(array(controls))
}
