//! ASCII VTK XML interchange, contract version 1. Numeric arrays retain their
//! native precision. Metadata text uses explicitly named UTF-8 UInt8 arrays;
//! consumers decode those bytes, without a trailing NUL. No XML text injection.
//!
//! Writers validate before output, borrow immutable state, and never flush or
//! close the caller's stream. An I/O failure may leave a partial document; the
//! caller owns atomic file replacement and final flush. Output memory is O(1)
//! beyond the borrowed state and formatting buffers. The format follows
//! <https://docs.vtk.org/en/latest/vtk_file_formats/vtkxml_file_format.html>.
use crate::{
    Axis, FluidParticle, ParticleDiagnostic, SessionIdentity, SessionSnapshot, SnapshotStage,
};
use std::{
    fmt::Display,
    io::{self, Write},
};

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
fn identity_valid(identity: &SessionIdentity) -> io::Result<()> {
    if !identity.is_valid() {
        return Err(invalid(
            "VTK source revision must be 1..1000 bytes without control characters",
        ));
    }
    Ok(())
}
fn array<W: Write, T: Display>(
    out: &mut W,
    kind: &str,
    name: &str,
    components: usize,
    tuples: usize,
    values: impl IntoIterator<Item = T>,
) -> io::Result<()> {
    write!(
        out,
        "<DataArray type=\"{kind}\" Name=\"{name}\" NumberOfComponents=\"{components}\" NumberOfTuples=\"{tuples}\" format=\"ascii\">"
    )?;
    for value in values {
        write!(out, "{value} ")?;
    }
    writeln!(out, "</DataArray>")
}
fn text<W: Write>(out: &mut W, name: &str, value: &str) -> io::Result<()> {
    array(out, "UInt8", name, 1, value.len(), value.bytes())
}
fn identity<W: Write>(
    out: &mut W,
    id: &SessionIdentity,
    generation: u64,
    time: f64,
) -> io::Result<()> {
    array(out, "UInt32", "rheon_vtk_contract_version", 1, 1, [1])?;
    array(out, "UInt64", "run_id", 1, 1, [id.run_id])?;
    array(out, "UInt64", "generation", 1, 1, [generation])?;
    array(out, "Float64", "TimeValue", 1, 1, [time])?;
    text(out, "source_revision_utf8", &id.source_revision)?;
    text(
        out,
        "coordinate_frame_utf8",
        "lab; Cartesian XYZ; position m; time s",
    )
}

/// Export the full-box session as ImageData with cell-centered scalar fields.
/// `interval_pressure_pa` exists only after an accepted step. Its gauge is cell
/// zero, and it belongs to the recorded interval, not an end-surface pressure.
/// Constructor pressure is explicitly unavailable, never a fabricated zero map.
/// Native face velocities remain Float32 FieldData arrays named `velocity_x/y/z`
/// with `*_dimensions` and `*_offset` (in grid units); ordering is x-fast.
/// `velocity_cell_m_s` is the Float64 adjacent-face average on full box cells.
/// Fraction is not a pressure/solid mask; tracer is dimensionless appearance.
///
/// # Errors
/// InvalidInput for malformed/nonfinite state or unsupported geometry; other
/// errors preserve the supplied writer's failure. No bytes precede validation.
pub fn write_session_vti<W: Write>(out: &mut W, snapshot: &SessionSnapshot<'_>) -> io::Result<()> {
    identity_valid(snapshot.identity())?;
    let g = snapshot.grid();
    let state = snapshot.state();
    let n = g.counts();
    let count = g.cell_len();
    let faces = [state.carrier.x, state.carrier.y, state.carrier.z];
    if state.pressure_surface.is_some()
        || state.pressure_columns.is_some()
        || state.reconstructed_surface.is_some()
        || state.flat_column_mac
    {
        return Err(invalid(
            "VTK session export supports full-box geometry only",
        ));
    }
    if !state.carrier.time.is_finite()
        || state.carrier.time < 0.0
        || state.liquid.time != state.carrier.time
        || state.liquid.fraction.len() != count
        || state.carrier.tracer.len() != count
        || state.pressure.len() != count
        || state
            .liquid
            .fraction
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || state.carrier.tracer.iter().any(|v| !v.is_finite())
        || state.pressure.iter().any(|v| !v.is_finite())
    {
        return Err(invalid("VTK session scalar shape, time, or values invalid"));
    }
    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        if faces[d].len() != g.face_len(axis) || faces[d].iter().any(|v| !v.is_finite()) {
            return Err(invalid("VTK native face field shape or values invalid"));
        }
    }
    if let Some(report) = snapshot.report()
        && (!report.carrier.step.dt.is_finite()
            || report.carrier.step.dt <= 0.0
            || report.carrier.step.time != state.carrier.time
            || state.carrier.time - report.carrier.step.dt < 0.0)
    {
        return Err(invalid("VTK interval metadata invalid"));
    }
    let o = g.origin();
    let h = g.spacing();
    writeln!(
        out,
        "<?xml version=\"1.0\"?>\n<VTKFile type=\"ImageData\" version=\"1.0\" byte_order=\"LittleEndian\">"
    )?;
    writeln!(
        out,
        "<ImageData WholeExtent=\"0 {} 0 {} 0 {}\" Origin=\"{} {} {}\" Spacing=\"{} {} {}\">",
        n[0], n[1], n[2], o[0], o[1], o[2], h[0], h[1], h[2]
    )?;
    writeln!(out, "<FieldData>")?;
    identity(
        out,
        snapshot.identity(),
        state.carrier.generation,
        state.carrier.time,
    )?;
    array(out, "UInt64", "frame", 1, 1, [snapshot.frame() as u64])?;
    text(
        out,
        "model_utf8",
        "full-box carrier projection and represented-volume transport; no solid mask",
    )?;
    text(
        out,
        "stage_utf8",
        if snapshot.stage() == SnapshotStage::Accepted {
            "accepted"
        } else {
            "constructor"
        },
    )?;
    text(
        out,
        "array_order_utf8",
        "x-fast; index = i + Nx * (j + Ny * k)",
    )?;
    text(
        out,
        "velocity_cell_semantics_utf8",
        "derived adjacent-face average; lab velocity m/s; full-fluid box cells",
    )?;
    text(
        out,
        "native_velocity_semantics_utf8",
        "m/s; lab frame; face positions = origin + (indices + offset) * spacing",
    )?;
    text(
        out,
        "scalar_semantics_utf8",
        "fraction: represented volume / full cell volume; tracer: dimensionless appearance",
    )?;
    array(
        out,
        "UInt64",
        "carrier_stamp",
        2,
        1,
        [state.carrier_stamp.id, state.carrier_stamp.version],
    )?;
    array(
        out,
        "UInt64",
        "volume_stamp",
        2,
        1,
        [state.liquid.stamp.id, state.liquid.stamp.version],
    )?;
    if let Some(report) = snapshot.report() {
        array(
            out,
            "Float64",
            "pressure_interval_s",
            2,
            1,
            [
                state.carrier.time - report.carrier.step.dt,
                state.carrier.time,
            ],
        )?;
        array(out, "UInt64", "pressure_gauge_cell", 1, 1, [0])?;
        text(
            out,
            "pressure_semantics_utf8",
            "available: last accepted interval projection pressure Pa; cell 0 gauge = 0; not end-surface pressure",
        )?;
    } else {
        text(
            out,
            "pressure_semantics_utf8",
            "unavailable: constructor has no accepted pressure interval",
        )?;
    }
    for (d, label) in ["x", "y", "z"].into_iter().enumerate() {
        let axis = [Axis::X, Axis::Y, Axis::Z][d];
        let mut offset = [0.5; 3];
        offset[d] = 0.0;
        array(
            out,
            "Float32",
            &format!("velocity_{label}"),
            1,
            faces[d].len(),
            faces[d].iter(),
        )?;
        array(
            out,
            "UInt64",
            &format!("velocity_{label}_dimensions"),
            3,
            1,
            g.face_counts(axis),
        )?;
        array(
            out,
            "Float64",
            &format!("velocity_{label}_offset"),
            3,
            1,
            offset,
        )?;
    }
    writeln!(
        out,
        "</FieldData>\n<Piece Extent=\"0 {} 0 {} 0 {}\">\n<CellData Scalars=\"fraction\" Vectors=\"velocity_cell_m_s\">",
        n[0], n[1], n[2]
    )?;
    array(
        out,
        "Float64",
        "fraction",
        1,
        count,
        state.liquid.fraction.iter(),
    )?;
    array(
        out,
        "Float32",
        "tracer",
        1,
        count,
        state.carrier.tracer.iter(),
    )?;
    if snapshot.report().is_some() {
        array(
            out,
            "Float64",
            "interval_pressure_pa",
            1,
            count,
            state.pressure.iter(),
        )?;
    }
    let velocity = (0..count).flat_map(|i| {
        let c = [i % n[0], i / n[0] % n[1], i / (n[0] * n[1])];
        std::array::from_fn::<_, 3, _>(|d| {
            let axis = [Axis::X, Axis::Y, Axis::Z][d];
            let mut upper = c;
            upper[d] += 1;
            (f64::from(faces[d][g.face_index(axis, c).unwrap()])
                + f64::from(faces[d][g.face_index(axis, upper).unwrap()]))
                * 0.5
        })
    });
    array(out, "Float64", "velocity_cell_m_s", 3, count, velocity)?;
    writeln!(out, "</CellData>\n</Piece>\n</ImageData>\n</VTKFile>")
}

/// Export the canonical material API publication without constructing a second
/// snapshot schema. Accepted attributes and optional committed diagnostics retain
/// the stage/time separation documented by [`write_particles_vtp`]. This is field
/// interchange, not a restart checkpoint; solver configuration/control history
/// is not serialized.
///
/// # Errors
/// Preserves the input validation and writer errors of [`write_particles_vtp`].
pub fn write_material_vtp<W: Write>(
    out: &mut W,
    snapshot: &crate::MaterialSnapshot<'_>,
) -> io::Result<()> {
    write_particles_vtp(
        out,
        snapshot.identity(),
        snapshot.generation(),
        snapshot.time(),
        snapshot.particles(),
        snapshot.particle_diagnostics(),
    )
}

/// Export accepted particle positions/attributes as PolyData vertices. Point IDs
/// are append-only array indices scoped to `identity.run_id`, not global IDs.
/// The caller must supply one consistent accepted publication and its identity.
/// Optional diagnostic arrays MUST be from that publication's committed
/// post-emission, pre-integration evaluation. They remain FieldData with explicit
/// stage/time and index association, never pressure sampled at accepted points.
/// Material-specific pressure conventions are retained, not unified as gauge.
/// No diagnostics means no diagnostic arrays (including at generation zero).
///
/// # Errors
/// InvalidInput for invalid identity, lengths, times or nonfinite/nonphysical
/// particle values. Writer errors propagate; invalid inputs emit no bytes.
pub fn write_particles_vtp<W: Write>(
    out: &mut W,
    id: &SessionIdentity,
    generation: u64,
    time: f64,
    particles: &[FluidParticle],
    diagnostics: Option<(f64, &[ParticleDiagnostic])>,
) -> io::Result<()> {
    identity_valid(id)?;
    if !time.is_finite()
        || time < 0.0
        || particles.len() > i64::MAX as usize
        || particles.iter().any(|p| {
            p.position
                .iter()
                .chain(p.velocity.iter())
                .any(|v| !v.is_finite())
                || !p.mass.is_finite()
                || p.mass <= 0.0
                || p.material > 1
                || !p.temperature_kelvin.is_finite()
                || p.temperature_kelvin <= 0.0
        })
    {
        return Err(invalid("VTK particle time, count, or values invalid"));
    }
    if let Some((t, values)) = diagnostics
        && (!t.is_finite()
            || t < 0.0
            || t > time
            || values.len() != particles.len()
            || values.iter().any(|p| {
                !p.density.is_finite()
                    || p.density <= 0.0
                    || !p.pressure.is_finite()
                    || p.acceleration.iter().any(|v| !v.is_finite())
            }))
    {
        return Err(invalid("VTK diagnostic time, length, or values invalid"));
    }
    let count = particles.len();
    writeln!(
        out,
        "<?xml version=\"1.0\"?>\n<VTKFile type=\"PolyData\" version=\"1.0\" byte_order=\"LittleEndian\">\n<PolyData>\n<FieldData>"
    )?;
    identity(out, id, generation, time)?;
    text(
        out,
        "stage_utf8",
        if generation == 0 {
            "constructor"
        } else {
            "accepted"
        },
    )?;
    text(
        out,
        "particle_id_semantics_utf8",
        "append-only array index within run_id; new run restarts namespace",
    )?;
    text(
        out,
        "particle_units_utf8",
        "position m; velocity m/s; mass kg; temperature K; material 0 emitted / 1 surrounding",
    )?;
    if let Some((t, values)) = diagnostics {
        array(out, "Float64", "diagnostic_time_s", 1, 1, [t])?;
        text(
            out,
            "diagnostic_semantics_utf8",
            "post-emission pre-integration; excludes wall response; indexed by particle_id; not sampled at accepted positions",
        )?;
        text(
            out,
            "diagnostic_pressure_semantics_utf8",
            "Pa; material-specific liquid compression / gas absolute EOS; no common gauge",
        )?;
        array(
            out,
            "Float64",
            "preintegration_density_kg_m3",
            1,
            count,
            values.iter().map(|v| v.density),
        )?;
        array(
            out,
            "Float64",
            "preintegration_pressure_pa",
            1,
            count,
            values.iter().map(|v| v.pressure),
        )?;
        array(
            out,
            "Float64",
            "preintegration_acceleration_m_s2",
            3,
            count,
            values.iter().flat_map(|v| v.acceleration),
        )?;
    } else {
        text(
            out,
            "diagnostic_semantics_utf8",
            "unavailable: no committed evaluation supplied",
        )?;
    }
    writeln!(
        out,
        "</FieldData>\n<Piece NumberOfPoints=\"{count}\" NumberOfVerts=\"{count}\" NumberOfLines=\"0\" NumberOfStrips=\"0\" NumberOfPolys=\"0\">\n<PointData Vectors=\"velocity_m_s\">"
    )?;
    array(out, "UInt64", "particle_id", 1, count, 0..count)?;
    array(
        out,
        "Float64",
        "velocity_m_s",
        3,
        count,
        particles.iter().flat_map(|p| p.velocity),
    )?;
    array(
        out,
        "Float64",
        "mass_kg",
        1,
        count,
        particles.iter().map(|p| p.mass),
    )?;
    array(
        out,
        "Float64",
        "temperature_k",
        1,
        count,
        particles.iter().map(|p| p.temperature_kelvin),
    )?;
    array(
        out,
        "UInt8",
        "material",
        1,
        count,
        particles.iter().map(|p| p.material),
    )?;
    writeln!(out, "</PointData>\n<Points>")?;
    array(
        out,
        "Float64",
        "position_m",
        3,
        count,
        particles.iter().flat_map(|p| p.position),
    )?;
    writeln!(out, "</Points>\n<Verts>")?;
    array(out, "Int64", "connectivity", 1, count, 0..count)?;
    array(out, "Int64", "offsets", 1, count, 1..=count)?;
    writeln!(out, "</Verts>\n</Piece>\n</PolyData>\n</VTKFile>")
}
