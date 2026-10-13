include!(concat!(
    env!("RHEON_CANDIDATE"),
    "/tests/provider_step_contract.rs"
));
fn publication(s: &SimulationSession) -> (Vec<u8>, String) {
    (bytes(s), format!("{:?}", s.snapshot().provider_report()))
}
struct Observed {
    model: LabModalPressure,
    dt: std::cell::Cell<f64>,
}
impl PressureProvider for Observed {
    fn metadata(&self) -> SolverMetadata {
        self.model.metadata()
    }
    fn supports(&self, r: &PressureRequest<'_>) -> Result<(), ProviderRejection> {
        self.dt.set(r.dt);
        self.model.supports(r)
    }
    fn predict(
        &mut self,
        r: &PressureRequest<'_>,
        p: &mut [f64],
        c: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        self.model.predict(r, p, c)
    }
}
fn heterogeneous_session() -> SimulationSession {
    let g = grid(32);
    let fractions = (0..g.cell_len())
        .map(|i| 0.25 + 0.5 * (i % 32) as f64 / 31.0)
        .collect();
    SimulationSession::new(
        SessionIdentity {
            run_id: 710,
            source_revision: "external-production-validation".into(),
        },
        g,
        LiquidTransportConfig {
            carrier: config(),
            represented_density: 1.25,
            volume_stamp: VolumeStamp { id: 11, version: 0 },
            carrier_id: 12,
        },
        PressureImplementation::JacobiPcgV1,
        fractions,
        CAP,
        24,
    )
    .unwrap()
}
fn main() {
    use std::io::Write;
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    std::fs::create_dir_all(&out).unwrap();
    let model_bytes = std::fs::read(std::env::var("RHEON_AI_EXPORT").unwrap()).unwrap();
    let mut observer = Observed {
        model: LabModalPressure::from_export(&model_bytes).unwrap(),
        dt: std::cell::Cell::new(0.),
    };
    let mut tight = config();
    tight.max_courant = 1e-8;
    let mut shortened = session(32, tight);
    let mut scratch = workspace(32);
    let force = forces();
    let (short, report) = shortened
        .step_with_pressure_provider(
            inputs(&force),
            &mut scratch,
            Some(&mut observer),
            None,
            |_| false,
        )
        .unwrap();
    assert!(short.carrier.step.dt < 1. / 60.);
    assert_eq!(short.carrier.step.dt, observer.dt.get());
    assert!(matches!(
        report.rejection,
        Some(ProviderRejection::Unsupported(_))
    ));
    println!(
        "Automatically shortened requested dt 1/60 to {}; exact actual dt reached model support gate and fell back",
        short.carrier.step.dt
    );
    let mut a = LabModalPressure::from_export(&model_bytes).unwrap();
    let mut b = LabModalPressure::from_export(&model_bytes).unwrap();
    let mut reference = heterogeneous_session();
    let mut retry = heterogeneous_session();
    let mut wa = workspace(32);
    let mut wb = workspace(32);
    let f = forces();
    let mut receipt = std::fs::File::create(out.join("frames.csv")).unwrap();
    writeln!(receipt, "frame,time,dt,selected,divergence").unwrap();
    for frame in 1..=24 {
        let before = publication(&retry);
        if frame == 12 {
            assert!(
                retry
                    .step_with_pressure_provider(inputs(&f), &mut wb, Some(&mut b), None, |s| s
                        == LiquidStepStage::BeforeCommit)
                    .is_err()
            );
            assert_eq!(publication(&retry), before);
            assert!(matches!(
                retry.step_with_pressure_provider(
                    inputs(&f),
                    &mut wb,
                    Some(&mut b),
                    Some(std::time::Instant::now()),
                    |_| panic!("expired deadline callback")
                ),
                Err(SessionError::Deadline)
            ));
            assert_eq!(publication(&retry), before);
            let mut invalid = inputs(&f);
            invalid.requested_dt = f64::NAN;
            assert!(
                retry
                    .step_with_pressure_provider(invalid, &mut wb, Some(&mut b), None, |_| false)
                    .is_err()
            );
            assert_eq!(publication(&retry), before);
            let rates = vec![100.; grid(32).cell_len()];
            let mut invalid = inputs(&f);
            invalid.source =
                Some(LiquidVolumeSource::new(VolumeStamp { id: 19, version: 2 }, &rates).unwrap());
            let mut projected = false;
            assert!(
                retry
                    .step_with_pressure_provider(invalid, &mut wb, Some(&mut b), None, |s| {
                        projected |= s == LiquidStepStage::Carrier(StepStage::BeforeCommit);
                        false
                    })
                    .is_err()
            );
            assert!(projected);
            assert_eq!(publication(&retry), before);
        }
        let dt = if frame == 9 { 1. / 120. } else { 1. / 60. };
        let mut ia = inputs(&f);
        ia.requested_dt = dt;
        let mut ib = inputs(&f);
        ib.requested_dt = dt;
        let (_, pa) = reference
            .step_with_pressure_provider(ia, &mut wa, Some(&mut a), None, |_| false)
            .unwrap();
        let (step, pb) = retry
            .step_with_pressure_provider(ib, &mut wb, Some(&mut b), None, |_| false)
            .unwrap();
        assert_eq!(bytes(&reference), bytes(&retry));
        assert_eq!(pa.selected, pb.selected);
        assert_eq!(format!("{:?}", pa.attempted), format!("{:?}", pb.attempted));
        assert_eq!(pa.rejection, pb.rejection);
        assert_eq!(
            pb.selected,
            if frame == 9 {
                PressureImplementation::JacobiPcgV1.id()
            } else {
                "lab-modal-pressure-v1"
            }
        );
        let snap = retry.snapshot();
        assert_eq!(snap.frame(), frame);
        assert_eq!(snap.identity().run_id, 710);
        assert_eq!(
            format!("{:?}", snap.provider_report().unwrap()),
            format!("{:?}", pb)
        );
        let v = snap.state();
        assert_eq!(v.carrier.generation, frame as u64);
        let mut data = std::fs::File::create(out.join(format!("{frame:03}.bin"))).unwrap();
        for &value in v.pressure {
            data.write_all(&value.to_le_bytes()).unwrap();
        }
        for values in [v.carrier.x, v.carrier.y, v.carrier.z, v.carrier.tracer] {
            for &value in values {
                data.write_all(&(value as f64).to_le_bytes()).unwrap();
            }
        }
        for &value in v.liquid.fraction {
            data.write_all(&value.to_le_bytes()).unwrap();
        }
        writeln!(
            receipt,
            "{frame},{},{dt},{},{}",
            v.carrier.time, pb.selected, step.carrier.step.actual_divergence_max
        )
        .unwrap();
    }
    println!(
        "24 accepted intervals; 23 model selections; unsupported-dt fallback; four rejected calls preserve complete publication; every recovered frame byte-identical to uninterrupted reference"
    );
}
