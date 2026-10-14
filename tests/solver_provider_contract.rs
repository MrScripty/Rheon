use rheon::solver_provider::*;
use rheon::*;

fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-10,
        absolute_residual: 1e-12,
        divergence_limit: 1e-8,
        max_iterations: 1000,
    }
}
fn grid() -> GridGeometry {
    GridGeometry::new([4, 3, 2], [0.25, 0.3, 0.4], [0.0; 3]).unwrap()
}
struct Candidate {
    values: Vec<f64>,
    fail: Option<ProviderRejection>,
    calls: usize,
}
impl PressureProvider for Candidate {
    fn metadata(&self) -> SolverMetadata {
        SolverMetadata {
            id: "test-double-not-ai",
            version: "1",
            stage: "pressure",
            input: TensorMetadata {
                shape_xyz: [4, 3, 2],
                layout: "x-fastest-cell-centered",
                scalar: "f64",
                units: "m^3/s^2",
            },
            output: TensorMetadata {
                shape_xyz: [4, 3, 2],
                layout: "x-fastest-cell-centered",
                scalar: "f64",
                units: "Pa",
            },
            domain: "test only",
            provenance: None,
        }
    }
    fn supports(&self, _: &PressureRequest<'_>) -> Result<(), ProviderRejection> {
        if let Some(ProviderRejection::Unsupported(s)) = self.fail {
            Err(ProviderRejection::Unsupported(s))
        } else {
            Ok(())
        }
    }
    fn predict(
        &mut self,
        _: &PressureRequest<'_>,
        out: &mut [f64],
        _: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        self.calls += 1;
        if let Some(e) = &self.fail {
            return Err(e.clone());
        }
        // Short writes deliberately exercise unwritten-cell detection.
        out[..self.values.len()].copy_from_slice(&self.values);
        Ok(())
    }
}
fn fixture(g: &GridGeometry) -> (Vec<f64>, Vec<f64>) {
    let pressure: Vec<_> = (0..g.cell_len()).map(|i| (i as f64 * 0.3).sin()).collect();
    let mut rhs = vec![0.0; g.cell_len()];
    PressureOperator::new(g, 1.25)
        .unwrap()
        .apply_full(&pressure, &mut rhs)
        .unwrap();
    (pressure, rhs)
}
#[test]
fn exact_candidate_is_recomputed_and_no_fallback_runs() {
    let g = grid();
    let (pressure, rhs) = fixture(&g);
    let mut p = Candidate {
        values: pressure.clone(),
        fail: None,
        calls: 0,
    };
    let mut w = ProviderWorkspace::new(&g, 100000, PressureImplementation::JacobiPcgV1).unwrap();
    let report = w
        .solve(
            PressureRequest {
                grid: &g,
                density: 1.25,
                dt: 0.01,
                rhs: &rhs,
            },
            settings(),
            Some(&mut p),
            || false,
        )
        .unwrap();
    assert_eq!(report.selected, "test-double-not-ai");
    assert!(report.rejection.is_none());
    assert_eq!(w.pressure(), pressure);
    assert_eq!(report.accepted.true_residual_l2, 0.0);
    assert!(report.total >= report.inference + report.qualification);
    assert_eq!(report.fallback, std::time::Duration::ZERO);
}
#[test]
fn failures_fall_back_to_both_reference_implementations() {
    let g = grid();
    let (pressure, rhs) = fixture(&g);
    let cases = [
        (vec![], None, ProviderRejection::NonFiniteOutput),
        (
            vec![f64::INFINITY; g.cell_len()],
            None,
            ProviderRejection::NonFiniteOutput,
        ),
        (
            vec![2.0; g.cell_len()],
            None,
            ProviderRejection::NonZeroGauge,
        ),
        (
            vec![0.0; g.cell_len()],
            None,
            ProviderRejection::QualityGate,
        ),
        (
            vec![],
            Some(ProviderRejection::Execution("offline")),
            ProviderRejection::Execution("offline"),
        ),
        (
            vec![],
            Some(ProviderRejection::Unsupported("shape")),
            ProviderRejection::Unsupported("shape"),
        ),
    ];
    for method in PressureImplementation::ALL {
        for (values, fail, expected) in &cases {
            let mut p = Candidate {
                values: values.clone(),
                fail: fail.clone(),
                calls: 0,
            };
            let mut w = ProviderWorkspace::new(&g, 100000, method).unwrap();
            let report = w
                .solve(
                    PressureRequest {
                        grid: &g,
                        density: 1.25,
                        dt: 0.01,
                        rhs: &rhs,
                    },
                    settings(),
                    Some(&mut p),
                    || false,
                )
                .unwrap();
            assert_eq!(report.selected, method.id());
            assert_eq!(report.rejection.as_ref(), Some(expected));
            assert!(
                w.pressure()
                    .iter()
                    .zip(&pressure)
                    .all(|(a, b)| (a - b).abs() < 1e-8)
            );
            if matches!(expected, ProviderRejection::Unsupported(_)) {
                assert_eq!(p.calls, 0);
            }
        }
    }
}
#[test]
fn invalid_inputs_and_cancellation_never_become_success() {
    let g = grid();
    let (_, rhs) = fixture(&g);
    let mut w = ProviderWorkspace::new(&g, 100000, PressureImplementation::JacobiPcgV1).unwrap();
    let mut p = Candidate {
        values: vec![],
        fail: None,
        calls: 0,
    };
    for bad in [
        vec![1.0; g.cell_len()],
        vec![f64::NAN; g.cell_len()],
        vec![0.0; 2],
        vec![f64::MAX; g.cell_len()],
    ] {
        assert!(
            w.solve(
                PressureRequest {
                    grid: &g,
                    density: 1.25,
                    dt: 0.01,
                    rhs: &bad
                },
                settings(),
                Some(&mut p),
                || false
            )
            .is_err()
        );
    }
    for dt in [0.0, f64::NAN, -1.0] {
        assert!(
            w.solve(
                PressureRequest {
                    grid: &g,
                    density: 1.25,
                    dt,
                    rhs: &rhs
                },
                settings(),
                Some(&mut p),
                || false
            )
            .is_err()
        );
    }
    assert_eq!(p.calls, 0);
    assert!(matches!(
        w.solve(
            PressureRequest {
                grid: &g,
                density: 1.25,
                dt: 0.01,
                rhs: &rhs
            },
            settings(),
            Some(&mut p),
            || true
        ),
        Err(PressureError::Cancelled { .. })
    ));
    p.fail = Some(ProviderRejection::Cancelled);
    assert!(matches!(
        w.solve(
            PressureRequest {
                grid: &g,
                density: 1.25,
                dt: 0.01,
                rhs: &rhs
            },
            settings(),
            Some(&mut p),
            || false
        ),
        Err(PressureError::Cancelled { .. })
    ));
    assert!(ProviderWorkspace::new(&g, 1, PressureImplementation::JacobiPcgV1).is_err());
}
#[test]
fn full_residual_includes_zero_cell_and_divergence_gate() {
    let g = grid();
    let (mut pressure, rhs) = fixture(&g);
    pressure[1] += 1e-3;
    let mut p = Candidate {
        values: pressure,
        fail: None,
        calls: 0,
    };
    let mut w = ProviderWorkspace::new(&g, 100000, PressureImplementation::JacobiPcgV1).unwrap();
    let mut loose = settings();
    loose.relative_residual = 1.0;
    let report = w
        .solve(
            PressureRequest {
                grid: &g,
                density: 1.25,
                dt: 0.01,
                rhs: &rhs,
            },
            loose,
            Some(&mut p),
            || false,
        )
        .unwrap();
    assert_eq!(report.rejection, Some(ProviderRejection::QualityGate));
    assert!(report.candidate.unwrap().predicted_divergence_max > loose.divergence_limit);
}
#[test]
fn missing_or_corrupt_real_artifacts_are_never_mocked() {
    for bytes in [vec![], vec![0; 270336], b"RHEONAI1".to_vec()] {
        assert!(LabModalPressure::from_export(&bytes).is_err());
    }
}

/// Explicit qualification test: does not quietly substitute fixtures when absent.
#[test]
#[ignore = "requires RHEON_AI_EXPORT from the hash-pinned real lab checkpoint"]
fn real_checkpoint_support_replay_and_corruption() {
    let path = std::env::var("RHEON_AI_EXPORT").expect("set RHEON_AI_EXPORT");
    let mut bytes = std::fs::read(path).unwrap();
    let mut model = LabModalPressure::from_export(&bytes).unwrap();
    let g = GridGeometry::new([32; 3], [1.0 / 32.0; 3], [0.0; 3]).unwrap();
    let rhs = vec![0.0; g.cell_len()];
    let request = PressureRequest {
        grid: &g,
        density: 1.25,
        dt: 1.0 / 60.0,
        rhs: &rhs,
    };
    let mut w =
        ProviderWorkspace::new(&g, 64 * 1024 * 1024, PressureImplementation::JacobiPcgV1).unwrap();
    let r = w
        .solve(request, settings(), Some(&mut model), || false)
        .unwrap();
    assert_eq!(r.selected, "lab-modal-pressure-v1");
    assert!(w.pressure().iter().all(|v| *v == 0.0));
    for bad in [
        PressureRequest {
            density: 1.0,
            ..request
        },
        PressureRequest {
            dt: 0.02,
            ..request
        },
    ] {
        let r = w
            .solve(bad, settings(), Some(&mut model), || false)
            .unwrap();
        assert_eq!(r.selected, "jacobi-pcg-v1");
        assert!(matches!(
            r.rejection,
            Some(ProviderRejection::Unsupported(_))
        ));
    }
    let mut out = vec![0.0; g.cell_len()];
    assert_eq!(
        model.predict(&request, &mut out, &mut || true),
        Err(ProviderRejection::Cancelled)
    );
    assert!(
        model
            .predict(&request, &mut out[..4], &mut || false)
            .is_err()
    );
    // Cancellation after a completed axis must not contaminate a later call.
    let mut polls = 0;
    assert_eq!(
        model.predict(&request, &mut out, &mut || {
            polls += 1;
            polls == 2
        }),
        Err(ProviderRejection::Cancelled)
    );
    for other in [
        GridGeometry::new([16; 3], [1.0 / 16.0; 3], [0.0; 3]).unwrap(),
        GridGeometry::new([32; 3], [0.1; 3], [0.0; 3]).unwrap(),
        GridGeometry::new([32; 3], [1.0 / 32.0; 3], [1.0; 3]).unwrap(),
    ] {
        assert!(matches!(
            model.supports(&PressureRequest {
                grid: &other,
                ..request
            }),
            Err(ProviderRejection::Unsupported(_))
        ));
    }
    let mut impulse = rhs.clone();
    impulse[0] = 0.001;
    impulse[1] = -0.001;
    model
        .predict(
            &PressureRequest {
                rhs: &impulse,
                ..request
            },
            &mut out,
            &mut || false,
        )
        .unwrap();
    let original = out.clone();
    for v in &mut impulse {
        *v *= -2.0;
    }
    model
        .predict(
            &PressureRequest {
                rhs: &impulse,
                ..request
            },
            &mut out,
            &mut || false,
        )
        .unwrap();
    assert!(
        out.iter()
            .zip(original)
            .all(|(a, b)| (a + 2.0 * b).abs() < 1e-12)
    );
    bytes[100] ^= 1;
    assert!(LabModalPressure::from_export(&bytes).is_err());
}
