//! Adapter for the canonical material snapshot/VTK fields. Physics remains owned
//! by ParticleFluidSimulation; no re-evaluation occurs during recording or reload.
use super::*;
use crate::{
    FluidParticle, MaterialSnapshot, MaterialStepControls, ParticleDiagnostic,
    ParticleFluidSimulation, ParticleStepReport,
};

fn solver_identity() -> String {
    let capabilities = crate::MATERIAL_SESSION_CAPABILITIES;
    format!(
        "rheon-sph-preview/crate-{}/material-api-v{}/emitter-controls-v{}",
        capabilities.crate_version, capabilities.api_version, capabilities.emitter_controls_version
    )
}

/// Bind actual grid/configuration and collision-content identity. Provenance is
/// declared by the host and must disclose unspecified source/build identities.
pub fn material_run(
    identity: SessionIdentity,
    simulation: &ParticleFluidSimulation,
    provenance: &str,
) -> Result<RecordingRun, RecordingError> {
    let collision = if let Some(mesh) = simulation.collision_mesh() {
        let mut h = Sha256::new();
        for v in mesh.vertices().iter().flatten() {
            h.update(v.to_le_bytes());
        }
        for triangle in mesh.triangles() {
            for index in triangle {
                h.update((*index as u64).to_le_bytes());
            }
        }
        format!(
            "policy={:?}; vertices={}; triangles={}; coordinates_m_and_indices_sha256={:x}; geometry referenced by hash, not retained",
            mesh.policy(),
            mesh.vertices().len(),
            mesh.triangles().len(),
            h.finalize()
        )
    } else {
        "none; impermeable axis-aligned domain walls".into()
    };
    let grid = simulation.grid();
    let run = RecordingRun {
        identity,
        solver: solver_identity(),
        configuration: format!("{:?}; collision={collision}", simulation.config()),
        counts: grid.counts().map(|n| n as u64),
        spacing: grid.spacing(),
        origin: grid.origin(),
        provenance: provenance.into(),
    };
    run.validate()?;
    Ok(run)
}

/// Bind the actual fixed material API configuration and geometry. MaterialSession
/// does not expose collision mesh installation; its snapshot uses domain walls.
pub fn material_snapshot_run(
    snapshot: &MaterialSnapshot<'_>,
    provenance: &str,
) -> Result<RecordingRun, RecordingError> {
    let grid = snapshot.grid();
    let run = RecordingRun {
        identity: snapshot.identity().clone(),
        solver: solver_identity(),
        configuration: format!(
            "{:?}; collision=none; impermeable axis-aligned domain walls",
            snapshot.config()
        ),
        counts: grid.counts().map(|n| n as u64),
        spacing: grid.spacing(),
        origin: grid.origin(),
        provenance: provenance.into(),
    };
    run.validate()?;
    Ok(run)
}

/// Encode the accepted simulation publication, using the same canonical arrays
/// and times as MaterialSnapshot. Call only after a successful step. Payload is
/// checked before allocation; constructor diagnostics remain absent.
pub fn material_publication(
    sim: &ParticleFluidSimulation,
    report: Option<ParticleStepReport>,
    controls: Option<MaterialStepControls>,
    ordinal: usize,
    limit: usize,
) -> Result<RecordedStep, RecordingError> {
    publication(
        sim.generation(),
        sim.time(),
        sim.particles(),
        sim.particle_diagnostics(),
        report,
        controls,
        ordinal,
        limit,
    )
}
/// Public API consumers use their canonical immutable snapshot directly.
pub fn material_snapshot_publication(
    snapshot: &MaterialSnapshot<'_>,
    ordinal: usize,
    limit: usize,
) -> Result<RecordedStep, RecordingError> {
    publication(
        snapshot.generation(),
        snapshot.time(),
        snapshot.particles(),
        snapshot.particle_diagnostics(),
        snapshot.report().copied(),
        snapshot.accepted_controls(),
        ordinal,
        limit,
    )
}

#[allow(clippy::too_many_arguments)]
fn publication(
    generation: u64,
    time: f64,
    particles: &[FluidParticle],
    diagnostics: Option<(f64, &[ParticleDiagnostic])>,
    report: Option<ParticleStepReport>,
    controls: Option<MaterialStepControls>,
    ordinal: usize,
    limit: usize,
) -> Result<RecordedStep, RecordingError> {
    let n = particles.len();
    if diagnostics.is_some_and(|(_, d)| d.len() != n)
        || (generation == 0) != (report.is_none())
        || report.is_some_and(|r| r.particles != n)
        || (generation != 0 && (diagnostics.is_none() || controls.is_none()))
    {
        return Err(RecordingError::Invalid("material publication association"));
    }
    let metadata = format!("accepted_controls={controls:?}; report={report:?}");
    // Eight f64 particle attributes + one u8 category; five f64 diagnostics.
    // Descriptor slack is bounded independently of particle count.
    let required = n
        .checked_mul(if diagnostics.is_some() { 105 } else { 65 })
        .and_then(|b| b.checked_add(metadata.len() + 2048))
        .ok_or(RecordingError::Limit("material payload overflow"))?;
    if required > limit {
        return Err(RecordingError::Limit("material record payload"));
    }
    let dims = [
        u64::try_from(n).map_err(|_| RecordingError::Limit("particle count"))?,
        1,
        1,
    ];
    let stage = if generation == 0 {
        "constructor"
    } else {
        "accepted"
    };
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(8)
        .map_err(|_| RecordingError::Limit("allocation"))?;
    fields.push(float_field(
        "position",
        "m",
        3,
        dims,
        stage,
        None,
        particles.iter().flat_map(|p| p.position),
    )?);
    fields.push(float_field(
        "velocity",
        "m/s",
        3,
        dims,
        stage,
        None,
        particles.iter().flat_map(|p| p.velocity),
    )?);
    fields.push(float_field(
        "mass",
        "kg",
        1,
        dims,
        stage,
        None,
        particles.iter().map(|p| p.mass),
    )?);
    fields.push(float_field(
        "temperature",
        "K",
        1,
        dims,
        stage,
        None,
        particles.iter().map(|p| p.temperature_kelvin),
    )?);
    let mut categories = Vec::new();
    categories
        .try_reserve_exact(n)
        .map_err(|_| RecordingError::Limit("allocation"))?;
    categories.extend(particles.iter().map(|p| p.material));
    fields.push(RecordedField {
        name: "material".into(),
        units: "category: 0 emitted; 1 surrounding".into(),
        association: "particle_index_within_run".into(),
        stage: stage.into(),
        evaluation_time: None,
        dimensions: dims,
        components: 1,
        scalar_type: "u8".into(),
        data: categories,
    });
    if let Some((t, d)) = diagnostics {
        fields.push(float_field(
            "density",
            "kg/m^3",
            1,
            dims,
            "post_emission_preintegration_excludes_wall_response",
            Some(t),
            d.iter().map(|v| v.density),
        )?);
        fields.push(float_field(
            "pressure",
            "Pa",
            1,
            dims,
            "post_emission_preintegration_excludes_wall_response",
            Some(t),
            d.iter().map(|v| v.pressure),
        )?);
        fields.push(float_field(
            "acceleration",
            "m/s^2",
            3,
            dims,
            "post_emission_preintegration_excludes_wall_response",
            Some(t),
            d.iter().flat_map(|v| v.acceleration),
        )?);
    }
    let step = RecordedStep {
        ordinal,
        generation,
        time,
        dt: report.map(|r| r.dt),
        stage: if generation == 0 {
            SnapshotStage::Constructor
        } else {
            SnapshotStage::Accepted
        },
        controls: metadata,
        fields,
    };
    if step.payload_bytes()? > limit {
        return Err(RecordingError::Limit("record payload"));
    }
    Ok(step)
}
fn float_field(
    name: &str,
    units: &str,
    components: usize,
    dimensions: [u64; 3],
    stage: &str,
    evaluation_time: Option<f64>,
    values: impl Iterator<Item = f64>,
) -> Result<RecordedField, RecordingError> {
    let bytes = usize::try_from(dimensions[0])
        .ok()
        .and_then(|n| n.checked_mul(components))
        .and_then(|n| n.checked_mul(8))
        .ok_or(RecordingError::Limit("field bytes"))?;
    let mut data = Vec::new();
    data.try_reserve_exact(bytes)
        .map_err(|_| RecordingError::Limit("allocation"))?;
    for v in values {
        data.extend_from_slice(&v.to_le_bytes());
    }
    Ok(RecordedField {
        name: name.into(),
        units: units.into(),
        association: "particle_index_within_run".into(),
        stage: stage.into(),
        evaluation_time,
        dimensions,
        components,
        scalar_type: "f64".into(),
        data,
    })
}

/// Selected, decoded canonical particle publication. No current simulation or
/// evaluation is consulted. Only the promised inspection fields are preserved.
pub struct RecordedMaterial {
    pub generation: u64,
    pub time: f64,
    pub dt: Option<f64>,
    pub particles: Vec<FluidParticle>,
    pub diagnostics: Option<(f64, Vec<ParticleDiagnostic>)>,
}
impl RecordedMaterial {
    pub fn decode(step: &RecordedStep) -> Result<Self, RecordingError> {
        step.payload_bytes()?;
        let position = required(step, "position", 3, "m")?;
        let n = usize::try_from(position.dimensions[0])
            .map_err(|_| RecordingError::Invalid("particle count"))?;
        let velocity = required(step, "velocity", 3, "m/s")?;
        let mass = required(step, "mass", 1, "kg")?;
        let temperature = required(step, "temperature", 1, "K")?;
        let material = step
            .field("material")
            .ok_or(RecordingError::Invalid("missing material"))?;
        if material.scalar_type != "u8"
            || material.units != "category: 0 emitted; 1 surrounding"
            || material.association != "particle_index_within_run"
            || material.stage != position.stage
            || material.evaluation_time.is_some()
            || material.components != 1
            || material.dimensions != position.dimensions
            || [velocity, mass, temperature]
                .iter()
                .any(|f| f.dimensions != position.dimensions)
            || material.data.iter().any(|v| *v > 1)
        {
            return Err(RecordingError::Invalid("material field schema"));
        }
        let mut particles = Vec::new();
        particles
            .try_reserve_exact(n)
            .map_err(|_| RecordingError::Limit("allocation"))?;
        for i in 0..n {
            let p = FluidParticle {
                position: std::array::from_fn(|d| value(position, 3 * i + d)),
                velocity: std::array::from_fn(|d| value(velocity, 3 * i + d)),
                mass: value(mass, i),
                temperature_kelvin: value(temperature, i),
                material: material.data[i],
            };
            if p.mass <= 0.0 || p.temperature_kelvin <= 0.0 {
                return Err(RecordingError::Invalid("material values"));
            }
            particles.push(p);
        }
        let diagnostics = match (
            step.field("density"),
            step.field("pressure"),
            step.field("acceleration"),
        ) {
            (None, None, None) => None,
            (Some(d), Some(p), Some(a)) => {
                if d.units != "kg/m^3" || p.units != "Pa" || a.units != "m/s^2" {
                    return Err(RecordingError::Invalid("diagnostic units"));
                }
                let t = d
                    .evaluation_time
                    .ok_or(RecordingError::Invalid("diagnostic time"))?;
                for (f, c) in [(d, 1), (p, 1), (a, 3)] {
                    if f.scalar_type != "f64"
                        || f.components != c
                        || f.dimensions != position.dimensions
                        || f.association != "particle_index_within_run"
                        || f.stage != "post_emission_preintegration_excludes_wall_response"
                        || f.evaluation_time != Some(t)
                    {
                        return Err(RecordingError::Invalid("diagnostic association"));
                    }
                }
                let mut diagnostic = Vec::new();
                diagnostic
                    .try_reserve_exact(n)
                    .map_err(|_| RecordingError::Limit("allocation"))?;
                for i in 0..n {
                    let v = ParticleDiagnostic {
                        density: value(d, i),
                        pressure: value(p, i),
                        acceleration: std::array::from_fn(|axis| value(a, 3 * i + axis)),
                    };
                    if v.density <= 0.0 {
                        return Err(RecordingError::Invalid("density"));
                    }
                    diagnostic.push(v);
                }
                Some((t, diagnostic))
            }
            _ => return Err(RecordingError::Invalid("incomplete diagnostic fields")),
        };
        if matches!(step.stage, SnapshotStage::Constructor) != diagnostics.is_none() {
            return Err(RecordingError::Invalid(
                "material stage diagnostic inventory",
            ));
        }
        Ok(Self {
            generation: step.generation,
            time: step.time,
            dt: step.dt,
            particles,
            diagnostics,
        })
    }
    /// Use the existing VTK particle contract with selected values and times.
    pub fn write_vtp<W: std::io::Write>(
        &self,
        out: &mut W,
        identity: &SessionIdentity,
    ) -> std::io::Result<()> {
        crate::write_particles_vtp(
            out,
            identity,
            self.generation,
            self.time,
            &self.particles,
            self.diagnostics.as_ref().map(|(t, d)| (*t, d.as_slice())),
        )
    }
}
fn required<'a>(
    step: &'a RecordedStep,
    name: &str,
    components: usize,
    units: &str,
) -> Result<&'a RecordedField, RecordingError> {
    let f = step
        .field(name)
        .ok_or(RecordingError::Invalid("missing particle field"))?;
    if f.scalar_type != "f64"
        || f.components != components
        || f.units != units
        || f.dimensions[1..] != [1, 1]
        || f.association != "particle_index_within_run"
        || f.evaluation_time.is_some()
        || f.stage
            != if step.stage == SnapshotStage::Constructor {
                "constructor"
            } else {
                "accepted"
            }
    {
        return Err(RecordingError::Invalid("particle field schema"));
    }
    Ok(f)
}
fn value(f: &RecordedField, index: usize) -> f64 {
    f64::from_le_bytes(f.data[8 * index..8 * index + 8].try_into().unwrap())
}
