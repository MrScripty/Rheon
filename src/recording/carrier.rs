//! Fixed sealed-box native carrier. Capture immediately after step/step_with_emitter;
//! later attempts invalidate interval pressure. No fraction/mask/checkpoint invented.
use super::*;
use crate::{Axis, PressureImplementation, Simulation, StateView};
const CELL: &str = "cell_center; offset [0.5,0.5,0.5]; Cartesian XYZ; x-fast";
const FACE: [&str; 3] = [
    "x_face; offset [0,0.5,0.5]; Cartesian XYZ; x-fast",
    "y_face; offset [0.5,0,0.5]; Cartesian XYZ; x-fast",
    "z_face; offset [0.5,0.5,0]; Cartesian XYZ; x-fast",
];
const PRESSURE_STAGE: &str = "accepted_interval_projection; cell0 gauge=0; not endpoint pressure";
const NAMES: [&str; 5] = [
    "velocity_x",
    "velocity_y",
    "velocity_z",
    "tracer",
    "interval_pressure",
];
/// Actual fixed-box method/configuration/geometry, declared host source provenance.
pub fn carrier_run(
    identity: SessionIdentity,
    sim: &Simulation,
    provenance: &str,
) -> Result<RecordingRun, RecordingError> {
    let run = RecordingRun {
        identity,
        solver: format!(
            "rheon-carrier-full-box-v1/crate-{}/{}",
            env!("CARGO_PKG_VERSION"),
            sim.implementation().id()
        ),
        configuration: format!(
            "{:?}; fixed sealed box; no fraction/solid mask",
            sim.config()
        ),
        counts: sim.grid().counts().map(|n| n as u64),
        spacing: sim.grid().spacing(),
        origin: sim.grid().origin(),
        provenance: provenance.into(),
    };
    run.validate()?;
    Ok(run)
}
/// Native bits and success-stamped interval; actual inputs/report remain opaque
/// provenance. Unsupported APIs or intervening rejected attempts fail closed.
pub fn carrier_publication(
    sim: &Simulation,
    ordinal: usize,
    limit: usize,
) -> Result<RecordedStep, RecordingError> {
    let state = sim.state();
    let authored = sim.accepted_force_interval_pressure();
    let pressure = sim.accepted_interval_pressure().or_else(|| {
        authored.as_ref().map(|p| crate::AcceptedIntervalPressure {
            values: p.values,
            interval_start: p.interval_start,
            report: p.report,
            controls: p.controls,
        })
    });
    let constructor = state.generation == 0 && state.time == 0.0;
    if !constructor && pressure.is_none() {
        return Err(RecordingError::Unsupported(
            "carrier pressure not success-qualified; capture immediately after step/step_with_emitter",
        ));
    }
    let (dt, controls, start) = if let Some(p) = pressure {
        if p.report.generation != state.generation || p.report.time != state.time {
            return Err(RecordingError::Invalid("carrier pressure stamp"));
        }
        (
            Some(p.report.dt),
            if let Some(authored) = &authored {
                format!(
                    "external-force-metadata-v1; source_controls={:?}; report={:?}; external_stage_work={:?}",
                    p.controls, p.report, authored.work
                )
            } else {
                format!("{:?}; report={:?}", p.controls, p.report)
            },
            Some(p.interval_start),
        )
    } else {
        (None, "constructor; no accepted interval".into(), None)
    };
    let raw = [
        state.x.len(),
        state.y.len(),
        state.z.len(),
        state.tracer.len(),
    ]
    .into_iter()
    .try_fold(0usize, |total, n| {
        n.checked_mul(4)
            .and_then(|n| total.checked_add(n))
            .ok_or(RecordingError::Limit("carrier shape"))
    })?;
    let pressure_bytes = pressure
        .map_or(Some(0), |p| p.values.len().checked_mul(8))
        .ok_or(RecordingError::Limit("carrier shape"))?;
    let raw = raw
        .checked_add(pressure_bytes)
        .and_then(|n| n.checked_add(controls.len()))
        .and_then(|n| n.checked_add(4096))
        .ok_or(RecordingError::Limit("carrier shape"))?;
    if raw > limit {
        return Err(RecordingError::Limit("carrier record payload"));
    }
    let stage = if constructor {
        "constructor"
    } else {
        "accepted"
    };
    let mut fields = Vec::new();
    for (d, values) in [state.x, state.y, state.z].into_iter().enumerate() {
        let mut data = Vec::new();
        data.try_reserve_exact(values.len() * 4)
            .map_err(|_| RecordingError::Limit("allocation"))?;
        for v in values {
            data.extend_from_slice(&v.to_le_bytes());
        }
        fields.push(RecordedField {
            name: NAMES[d].into(),
            units: "m/s".into(),
            association: FACE[d].into(),
            stage: stage.into(),
            evaluation_time: None,
            dimensions: sim
                .grid()
                .face_counts([Axis::X, Axis::Y, Axis::Z][d])
                .map(|n| n as u64),
            components: 1,
            scalar_type: "f32".into(),
            data,
        });
    }
    let mut data = Vec::new();
    data.try_reserve_exact(state.tracer.len() * 4)
        .map_err(|_| RecordingError::Limit("allocation"))?;
    for v in state.tracer {
        data.extend_from_slice(&v.to_le_bytes());
    }
    fields.push(RecordedField {
        name: "tracer".into(),
        units: "1".into(),
        association: CELL.into(),
        stage: stage.into(),
        evaluation_time: None,
        dimensions: sim.grid().counts().map(|n| n as u64),
        components: 1,
        scalar_type: "f32".into(),
        data,
    });
    if let Some(p) = pressure {
        let mut data = Vec::new();
        data.try_reserve_exact(p.values.len() * 8)
            .map_err(|_| RecordingError::Limit("allocation"))?;
        for v in p.values {
            data.extend_from_slice(&v.to_le_bytes());
        }
        fields.push(RecordedField {
            name: "interval_pressure".into(),
            units: "Pa".into(),
            association: CELL.into(),
            stage: PRESSURE_STAGE.into(),
            evaluation_time: start,
            dimensions: sim.grid().counts().map(|n| n as u64),
            components: 1,
            scalar_type: "f64".into(),
            data,
        });
    }
    let mut step = RecordedStep {
        ordinal,
        generation: state.generation,
        time: state.time,
        dt,
        stage: if constructor {
            SnapshotStage::Constructor
        } else {
            SnapshotStage::Accepted
        },
        controls,
        fields,
    };
    if let Some(authored) = authored {
        force_fields::append_fields(&mut step, authored.fields, limit)?;
    }
    if step.payload_bytes()? > limit {
        return Err(RecordingError::Limit("carrier record payload"));
    }
    Ok(step)
}
/// Constructor metadata or checked actual accepted metadata. The same fields
/// must bind the RecordingRun through with_force_fields before recorder creation.
pub fn carrier_publication_with_force_fields(
    sim: &Simulation,
    fields: &crate::ForceSnapshot,
    ordinal: usize,
    limit: usize,
) -> Result<RecordedStep, RecordingError> {
    let mut step = carrier_publication(sim, ordinal, limit)?;
    force_fields::require_fields(&mut step, fields, limit)?;
    Ok(step)
}
/// Selected owned carrier values; one bounded step, never a live simulation.
pub struct RecordedCarrier {
    pub grid: GridGeometry,
    pub implementation: PressureImplementation,
    pub generation: u64,
    pub time: f64,
    pub dt: Option<f64>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub z: Vec<f32>,
    pub tracer: Vec<f32>,
    /// Actual interval start and Pa values; cell0 gauge, not endpoint pressure.
    pub interval_pressure: Option<(f64, Vec<f64>)>,
    pub force_fields: Option<crate::ForceSnapshot>,
}
impl RecordedCarrier {
    pub fn decode(run: &RecordingRun, step: &RecordedStep) -> Result<Self, RecordingError> {
        step.payload_bytes()?;
        run.validate()?;
        if !run.solver.starts_with("rheon-carrier-full-box-v1/") {
            return Err(RecordingError::Unsupported("carrier solver identity"));
        }
        let implementation = run
            .solver
            .rsplit('/')
            .next()
            .and_then(PressureImplementation::from_id)
            .ok_or(RecordingError::Unsupported("carrier method"))?;
        let grid = GridGeometry::new(run.counts, run.spacing, run.origin)
            .map_err(|_| RecordingError::Invalid("carrier geometry"))?;
        validate_run_force_fields(run, step)?;
        let force_fields = recorded_force_fields(step)?;
        let initial = step.stage == SnapshotStage::Constructor;
        if step.fields.len() != (if initial { 4 } else { 5 }) + usize::from(force_fields.is_some())
            || step
                .fields
                .iter()
                .any(|f| !NAMES.contains(&f.name.as_str()) && f.name != force_fields::FIELD)
        {
            return Err(RecordingError::Invalid("carrier field inventory"));
        }
        let stage = if initial { "constructor" } else { "accepted" };
        let mut faces = Vec::new();
        for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
            let field = required(
                step,
                NAMES[d],
                "f32",
                "m/s",
                FACE[d],
                stage,
                grid.face_counts(axis),
                None,
            )?;
            faces.push(f32_values(field)?);
        }
        let tracer = f32_values(required(
            step,
            "tracer",
            "f32",
            "1",
            CELL,
            stage,
            grid.counts(),
            None,
        )?)?;
        if tracer.iter().any(|v| !(0.0..=1.0).contains(v)) {
            return Err(RecordingError::Invalid("carrier tracer range"));
        }
        let interval_pressure = if initial {
            None
        } else {
            let p = step
                .field("interval_pressure")
                .ok_or(RecordingError::Invalid("carrier field inventory"))?;
            let start = p
                .evaluation_time
                .ok_or(RecordingError::Invalid("carrier interval"))?;
            let dt = step.dt.ok_or(RecordingError::Invalid("carrier interval"))?;
            if start >= step.time
                || (step.time - start - dt).abs() > 8.0 * f64::EPSILON * step.time.max(dt)
            {
                return Err(RecordingError::Invalid("carrier interval"));
            }
            let p = required(
                step,
                "interval_pressure",
                "f64",
                "Pa",
                CELL,
                PRESSURE_STAGE,
                grid.counts(),
                Some(start),
            )?;
            let mut values = Vec::new();
            values
                .try_reserve_exact(grid.cell_len())
                .map_err(|_| RecordingError::Limit("allocation"))?;
            values.extend(
                p.data
                    .chunks_exact(8)
                    .map(|b| f64::from_le_bytes(b.try_into().unwrap())),
            );
            if values[0] != 0.0 {
                return Err(RecordingError::Invalid("carrier pressure gauge"));
            }
            Some((start, values))
        };
        let mut faces = faces.into_iter();
        Ok(Self {
            grid,
            implementation,
            generation: step.generation,
            time: step.time,
            dt: step.dt,
            x: faces.next().unwrap(),
            y: faces.next().unwrap(),
            z: faces.next().unwrap(),
            tracer,
            interval_pressure,
            force_fields,
        })
    }
    pub fn state(&self) -> StateView<'_> {
        StateView {
            x: &self.x,
            y: &self.y,
            z: &self.z,
            tracer: &self.tracer,
            time: self.time,
            generation: self.generation,
        }
    }
    pub fn write_vti<W: std::io::Write>(
        &self,
        out: &mut W,
        identity: &SessionIdentity,
    ) -> std::io::Result<()> {
        crate::write_carrier_vti(
            out,
            identity,
            &self.grid,
            self.state(),
            self.dt,
            self.interval_pressure
                .as_ref()
                .map(|(t, p)| (*t, p.as_slice())),
        )
    }
}
#[allow(clippy::too_many_arguments)]
fn required<'a>(
    step: &'a RecordedStep,
    name: &str,
    kind: &str,
    units: &str,
    association: &str,
    stage: &str,
    dims: [usize; 3],
    evaluation: Option<f64>,
) -> Result<&'a RecordedField, RecordingError> {
    let f = step
        .field(name)
        .ok_or(RecordingError::Invalid("carrier field inventory"))?;
    if f.scalar_type != kind
        || f.units != units
        || f.association != association
        || f.stage != stage
        || f.dimensions != dims.map(|n| n as u64)
        || f.components != 1
        || f.evaluation_time != evaluation
    {
        return Err(RecordingError::Invalid("carrier field schema"));
    }
    Ok(f)
}
fn f32_values(field: &RecordedField) -> Result<Vec<f32>, RecordingError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(field.data.len() / 4)
        .map_err(|_| RecordingError::Limit("allocation"))?;
    values.extend(
        field
            .data
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap())),
    );
    Ok(values)
}
