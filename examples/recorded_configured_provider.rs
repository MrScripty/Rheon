//! Record actual configured accepted intervals, then inspect in another process.
//! The authored candidate runs the native solver: no model/training claim.
use rheon::{
    recording::{provider::*, *},
    solver_provider::{ProviderRejection, configured::*},
    *,
};
use std::{error::Error, path::Path, sync::Arc};

const CAP: usize = 4 * 1024 * 1024;
fn grid() -> GridGeometry {
    GridGeometry::new([5; 3], [0.25; 3], [0.0; 3]).unwrap()
}
fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 2.0,
            memory_limit: CAP,
            pressure: PressureSettings {
                relative_residual: 1e-14,
                absolute_residual: 1e-16,
                divergence_limit: 1e-7,
                max_iterations: 3000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0 / 128.0,
        },
        represented_density: 2.0,
        volume_stamp: VolumeStamp { id: 40, version: 0 },
        carrier_id: 41,
    }
}
struct AuthoredNativeCandidate {
    pressure: PressureWorkspace,
    reject: bool,
    identity: Arc<ContextProviderIdentity>,
}
impl ContextPressureProvider for AuthoredNativeCandidate {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        Arc::clone(&self.identity)
    }
    fn supports(&self, _: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        Ok(())
    }
    fn predict(
        &mut self,
        request: &ContextPressureRequest<'_>,
        out: &mut [f64],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        let context = &request.context;
        self.pressure
            .solve_rhs(
                &PressureOperator::new(context.grid(), context.density()).unwrap(),
                context.rhs(),
                context.dt(),
                config().carrier.pressure,
                cancelled,
            )
            .map_err(|_| ProviderRejection::Execution("authored native example solve failed"))?;
        out.copy_from_slice(self.pressure.pressure());
        if self.reject {
            out[1] += 100.0;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("record") if args.len() == 3 => record(Path::new(&args[1]), &args[2]),
        Some("inspect") if args.len() == 3 => inspect(Path::new(&args[1]), args[2].parse()?),
        _ => Err("usage: recorded_configured_provider record NEW.sqlite SOURCE_REVISION | inspect RUN.sqlite ORDINAL".into()),
    }
}

fn record(path: &Path, revision: &str) -> Result<(), Box<dyn Error>> {
    let mut session = SimulationSession::new(
        SessionIdentity {
            run_id: 902,
            source_revision: revision.into(),
        },
        grid(),
        config(),
        PressureImplementation::JacobiPcgV1,
        vec![1.0; grid().cell_len()],
        CAP,
        3,
    )?;
    let mut workspace =
        ProviderStepWorkspace::new(&grid(), 2.0, CAP, PressureImplementation::JacobiPcgV1)?;
    let mut viscosity = ViscosityWorkspace::new(grid(), CAP)?;
    let mut candidate = AuthoredNativeCandidate {
        pressure: PressureWorkspace::new(&grid(), CAP)?,
        reject: false,
        identity: Arc::new(ContextProviderIdentity {
            id: "authored-native-recording-example".into(),
            version: "1".into(),
            manifest_sha256: [3; 32],
            checkpoint_sha256: None,
            training_run_id: None,
            runtime: "in-process-native-example".into(),
            source_revision: revision.into(),
        }),
    };
    let limits = RecordingLimits {
        max_database_bytes: 8 * 1024 * 1024,
        max_record_bytes: 1024 * 1024,
        max_records: 4,
    };
    let run = configured_run(
        &session.snapshot(),
        "CPU authored native candidate; caller-declared source; build/toolchain unspecified; no learned model, measurements or restart",
    )?;
    let mut recorder = StepRecorder::create(path, run, limits)?;
    recorder.append(&configured_publication(
        &session.snapshot(),
        0,
        limits.max_record_bytes,
    )?)?;
    let forces = [BodyForce {
        value: [0.125, 0.0625, 0.0625],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    let result = (|| -> Result<(), Box<dyn Error>> {
        for interval in 0..3 {
            candidate.reject = interval == 2;
            let preferred =
                (interval != 0).then_some(&mut candidate as &mut dyn ContextPressureProvider);
            session.step_with_configured_pressure(
                LiquidStepInputs {
                    requested_dt: 0.0625,
                    smoke_source: None,
                    forces: &forces,
                    inlet: LiquidInlet::new(VolumeStamp { id: 42, version: 0 }, [[0.0; 2]; 3])?,
                    source: None,
                    volume: LiquidVolumeSettings {
                        max_outward_courant: 1.0,
                        actual_divergence_limit: 1e-5,
                    },
                },
                &mut workspace,
                Some((&mut viscosity, 0.0078125)),
                preferred,
                None,
                |_| false,
            )?;
            recorder.append(&configured_publication(
                &session.snapshot(),
                recorder.recorded_count(),
                limits.max_record_bytes,
            )?)?;
        }
        Ok(())
    })();
    let generation = session.snapshot().state().carrier.generation;
    let detail = match &result {
        Ok(()) => "all accepted configured intervals retained".into(),
        Err(error) => format!("{error}; accepted generation {generation}; tail may be unrecorded"),
    };
    recorder.finish(
        if result.is_ok() {
            "completed"
        } else {
            "failed"
        },
        &detail,
        Some(generation),
    )?;
    result?;
    println!(
        "{} publications recorded to {}; native / authored candidate / QualityGate fallback; no model accuracy claim",
        recorder.recorded_count(),
        path.display()
    );
    Ok(())
}

fn inspect(path: &Path, ordinal: usize) -> Result<(), Box<dyn Error>> {
    // Historical inspection constructs no simulation, provider or runtime.
    let reader = RecordingReader::open(path, RecordingLimits::default())?;
    reader.verify()?;
    let coverage = verify_configured_recording(&reader)?;
    let step = reader.select(ordinal)?;
    let selected = RecordedConfiguredSession::decode(reader.run(), &step)?;
    println!(
        "run {} source {} ordinal {} generation {} publication {} s dt {:?}; {:?}",
        reader.run().identity.run_id,
        reader.run().identity.source_revision,
        ordinal,
        selected.generation,
        selected.time,
        selected.dt,
        reader.status()?
    );
    println!(
        "prior history: {} verified links, {} unavailable links; native face sample counts {:?}; fraction cells {}",
        coverage.verified_prior_links,
        coverage.unavailable_prior_links,
        [selected.x.len(), selected.y.len(), selected.z.len()],
        selected.fraction.len()
    );
    if let Some((start, pressure)) = &selected.interval_pressure {
        println!(
            "interval projection pressure: [{start}, {}] s, {} Pa samples, cell0 gauge {} Pa",
            selected.time,
            pressure.len(),
            pressure[0]
        );
    }
    if let Some(receipt) = selected.receipt {
        println!(
            "pressure selection {:?}; rejection {:?}; prediction called {}; viscosity {} Pa s",
            receipt.selected,
            receipt.rejection,
            receipt.prediction_called,
            receipt.dynamic_viscosity
        );
        println!(
            "pressure-call durations only: total {:?}, inference {:?}, qualification {:?}, fallback {:?}; excludes construction and full transport",
            receipt.total, receipt.inference, receipt.qualification, receipt.fallback
        );
        if let Some(binding) = receipt.binding {
            println!(
                "declared package {} version {} runtime {} source {}",
                binding.id, binding.version, binding.runtime, binding.source_revision
            );
        }
    } else {
        println!("constructor: no accepted pressure receipt");
    }
    Ok(())
}
