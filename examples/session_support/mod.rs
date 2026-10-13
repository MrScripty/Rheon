use rheon::*;
pub fn fixture(counts: [u64; 3], version: u64) -> (SimulationSession, BoxFluxStepBoundary) {
    let g = GridGeometry::new(
        counts,
        [
            1.0 / counts[0] as f64,
            1.0 / counts[1] as f64,
            1.0 / counts[2] as f64,
        ],
        [0.0; 3],
    )
    .unwrap();
    let mut f = vec![0.0; g.cell_len()];
    for k in 0..counts[2] as usize {
        for j in 0..counts[1] as usize {
            for i in counts[0] as usize / 4..counts[0] as usize / 2 {
                f[g.cell_index([i, j, k]).unwrap()] = 0.5;
            }
        }
    }
    let c = LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1000.0,
            memory_limit: 16 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 10000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 800.0,
        volume_stamp: VolumeStamp {
            id: u64::MAX,
            version,
        },
        carrier_id: u64::MAX - 1,
    };
    let method = PressureImplementation::JacobiPcgV1;
    let session = SimulationSession::new(
        SessionIdentity {
            run_id: 1,
            source_revision: "example-authored-fixture".into(),
        },
        g,
        c,
        method,
        f,
        16 * 1024 * 1024,
        8,
    )
    .unwrap();
    (
        session,
        BoxFluxStepBoundary {
            flux: PrescribedBoxFlux::new(
                BoxFluxStamp { id: 47, version: 0 },
                [[-0.25, 0.25], [0.0; 2], [0.0; 2]],
            )
            .unwrap(),
            tracer: BoxFluxTracerPolicy::ClampedAppearance,
        },
    )
}
pub fn inputs() -> LiquidStepInputs<'static> {
    LiquidStepInputs {
        requested_dt: 0.0625,
        smoke_source: None,
        forces: &[],
        inlet: LiquidInlet::new(VolumeStamp { id: 53, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
