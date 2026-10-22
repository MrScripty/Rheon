use super::*;
use crate::solver_provider::{ProviderRejection, SolverMetadata, TensorMetadata};
use crate::{GridGeometry, PressureImplementation, PressureOperator, PressureReport};
use std::cell::RefCell;

fn grid() -> GridGeometry {
    GridGeometry::new([4, 3, 2], [0.25, 0.3, 0.4], [0.0; 3]).unwrap()
}
fn settings() -> PressureSettings {
    PressureSettings {
        relative_residual: 1e-10,
        absolute_residual: 1e-12,
        divergence_limit: 1e-8,
        max_iterations: 1000,
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
fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|x| x.to_bits()).collect()
}
fn numerical(report: PressureReport) -> (usize, u64, u64, u64) {
    (
        report.iterations,
        report.true_residual_l2.to_bits(),
        report.true_residual_max.to_bits(),
        report.predicted_divergence_max.to_bits(),
    )
}
fn same_report(a: &ProviderReport, b: &ProviderReport) {
    assert_eq!(a.selected, b.selected);
    assert_eq!(a.attempted, b.attempted);
    assert_eq!(a.rejection, b.rejection);
    assert_eq!(a.candidate.map(numerical), b.candidate.map(numerical));
    assert_eq!(numerical(a.accepted), numerical(b.accepted));
}

#[derive(Debug, Clone, PartialEq)]
struct Observation {
    grid: GridGeometry,
    density: u64,
    dt: u64,
    rhs: Vec<u64>,
}
#[derive(Clone)]
struct Candidate {
    values: Vec<f64>,
    fail: Option<ProviderRejection>,
    seen: RefCell<Vec<Observation>>,
    predictions: usize,
}
impl Candidate {
    fn new(values: Vec<f64>, fail: Option<ProviderRejection>) -> Self {
        Self {
            values,
            fail,
            seen: RefCell::new(Vec::new()),
            predictions: 0,
        }
    }
    fn observe(&self, request: &PressureRequest<'_>) {
        self.seen.borrow_mut().push(Observation {
            grid: request.grid.clone(),
            density: request.density.to_bits(),
            dt: request.dt.to_bits(),
            rhs: bits(request.rhs),
        });
    }
}
impl PressureProvider for Candidate {
    fn metadata(&self) -> SolverMetadata {
        let tensor = |units| TensorMetadata {
            shape_xyz: [4, 3, 2],
            layout: "x-fastest-cell-centered",
            scalar: "f64",
            units,
        };
        SolverMetadata {
            id: "c1a-classical-test-not-ai",
            version: "1",
            stage: "pressure",
            input: tensor("m^3/s^2"),
            output: tensor("Pa"),
            domain: "classical contract test only",
            provenance: None,
        }
    }
    fn supports(&self, request: &PressureRequest<'_>) -> Result<(), ProviderRejection> {
        self.observe(request);
        if let Some(ProviderRejection::Unsupported(reason)) = self.fail {
            Err(ProviderRejection::Unsupported(reason))
        } else {
            Ok(())
        }
    }
    fn predict(
        &mut self,
        request: &PressureRequest<'_>,
        out: &mut [f64],
        _: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        self.observe(request);
        self.predictions += 1;
        if let Some(error) = &self.fail {
            return Err(error.clone());
        }
        out[..self.values.len()].copy_from_slice(&self.values);
        Ok(())
    }
}

#[test]
fn c1a_pressure_exact_and_adversarial_candidates_match_original_dispatch() {
    let g = grid();
    let (pressure, rhs) = fixture(&g);
    let request = PressureRequest {
        grid: &g,
        density: 1.25,
        dt: 0.01,
        rhs: &rhs,
    };
    let cases = [
        (pressure.clone(), None, None),
        (vec![], None, Some(ProviderRejection::NonFiniteOutput)),
        (
            vec![f64::NAN; g.cell_len()],
            None,
            Some(ProviderRejection::NonFiniteOutput),
        ),
        (
            vec![f64::INFINITY; g.cell_len()],
            None,
            Some(ProviderRejection::NonFiniteOutput),
        ),
        (
            vec![2.0; g.cell_len()],
            None,
            Some(ProviderRejection::NonZeroGauge),
        ),
        (
            vec![0.0; g.cell_len()],
            None,
            Some(ProviderRejection::QualityGate),
        ),
        (
            vec![],
            Some(ProviderRejection::Unsupported("shape")),
            Some(ProviderRejection::Unsupported("shape")),
        ),
        (
            vec![],
            Some(ProviderRejection::Execution("offline")),
            Some(ProviderRejection::Execution("offline")),
        ),
        (
            vec![],
            Some(ProviderRejection::Cancelled),
            Some(ProviderRejection::Cancelled),
        ),
    ];
    for method in PressureImplementation::ALL {
        for (values, fail, rejection) in &cases {
            let mut original = ProviderWorkspace::new(&g, 100000, method).unwrap();
            let mut adapted = ProviderWorkspace::new(&g, 100000, method).unwrap();
            let mut a = Candidate::new(values.clone(), fail.clone());
            let mut b = a.clone();
            let direct = original.solve(request, settings(), Some(&mut a), || false);
            let mut stage =
                PressureStageV1::select(ReplacementContract::PressureV1, &mut adapted).unwrap();
            let result = stage.solve(request, settings(), Some(&mut b), || false);
            match (direct, result) {
                (Ok(direct), Ok(result)) => {
                    same_report(&direct, result.report());
                    assert_eq!(&result.report().rejection, rejection);
                    assert_eq!(bits(original.pressure()), bits(result.pressure()));
                    if rejection.is_none() {
                        assert_eq!(bits(result.pressure()), bits(&pressure));
                        assert_eq!(result.report().accepted.true_residual_l2, 0.0);
                        assert_eq!(result.report().selected, "c1a-classical-test-not-ai");
                    } else {
                        assert_eq!(result.report().selected, method.id());
                    }
                }
                (Err(a), Err(b)) => assert_eq!(a, b),
                _ => panic!("adapter changed original outcome"),
            }
            assert_eq!(*a.seen.borrow(), *b.seen.borrow());
            assert_eq!(a.predictions, b.predictions);
            for observation in b.seen.into_inner() {
                assert_eq!(
                    observation,
                    Observation {
                        grid: g.clone(),
                        density: 1.25f64.to_bits(),
                        dt: 0.01f64.to_bits(),
                        rhs: bits(&rhs),
                    }
                );
            }
        }
    }
}

#[test]
fn c1a_pressure_invalid_inputs_and_host_cancellation_preserve_original_errors() {
    let g = grid();
    let (_, rhs) = fixture(&g);
    let other = GridGeometry::new([3, 3, 2], [0.25, 0.3, 0.4], [0.0; 3]).unwrap();
    let short = vec![0.0; 2];
    let nan = vec![f64::NAN; g.cell_len()];
    let incompatible = vec![1.0; g.cell_len()];
    let overflow = vec![f64::MAX; g.cell_len()];
    let base = PressureRequest {
        grid: &g,
        density: 1.25,
        dt: 0.01,
        rhs: &rhs,
    };
    let mut bad_settings = settings();
    bad_settings.relative_residual = f64::NAN;
    let cases = [
        (
            PressureRequest {
                grid: &other,
                ..base
            },
            settings(),
            false,
        ),
        (
            PressureRequest {
                rhs: &short,
                ..base
            },
            settings(),
            false,
        ),
        (PressureRequest { rhs: &nan, ..base }, settings(), false),
        (
            PressureRequest {
                rhs: &incompatible,
                ..base
            },
            settings(),
            false,
        ),
        (
            PressureRequest {
                rhs: &overflow,
                ..base
            },
            settings(),
            false,
        ),
        (
            PressureRequest {
                density: 0.0,
                ..base
            },
            settings(),
            false,
        ),
        (
            PressureRequest {
                density: f64::NAN,
                ..base
            },
            settings(),
            false,
        ),
        (PressureRequest { dt: 0.0, ..base }, settings(), false),
        (PressureRequest { dt: -1.0, ..base }, settings(), false),
        (
            PressureRequest {
                dt: f64::NAN,
                ..base
            },
            settings(),
            false,
        ),
        (base, bad_settings, false),
        (base, settings(), true),
    ];
    for (request, settings, cancelled) in cases {
        let mut original =
            ProviderWorkspace::new(&g, 100000, PressureImplementation::JacobiPcgV1).unwrap();
        let mut adapted =
            ProviderWorkspace::new(&g, 100000, PressureImplementation::JacobiPcgV1).unwrap();
        let mut a = Candidate::new(vec![], None);
        let mut b = a.clone();
        let direct = original
            .solve(request, settings, Some(&mut a), || cancelled)
            .unwrap_err();
        let mut stage =
            PressureStageV1::select(ReplacementContract::PressureV1, &mut adapted).unwrap();
        let error = match stage.solve(request, settings, Some(&mut b), || cancelled) {
            Err(error) => error,
            Ok(_) => panic!("invalid input qualified"),
        };
        assert_eq!(direct, error);
        assert!(b.seen.borrow().is_empty());
        assert_eq!(b.predictions, 0);
    }
}

#[test]
fn c1a_unavailable_scopes_preserve_seeded_workspace_and_v1_iteration_semantics() {
    let g = grid();
    let (pressure, rhs) = fixture(&g);
    let request = PressureRequest {
        grid: &g,
        density: 1.25,
        dt: 0.01,
        rhs: &rhs,
    };
    let mut workspace =
        ProviderWorkspace::new(&g, 100000, PressureImplementation::JacobiPcgV1).unwrap();
    let mut candidate = Candidate::new(pressure.clone(), None);
    let mut zero_iterations = settings();
    zero_iterations.max_iterations = 0;
    let direct = workspace
        .solve(request, zero_iterations, Some(&mut candidate), || false)
        .unwrap();
    let seeded = bits(workspace.pressure());
    for requested in [
        ReplacementContract::SealedMacProjectionV1,
        ReplacementContract::WholeSealedMacV1,
        ReplacementContract::WholeParticlePreviewV1,
    ] {
        assert!(
            matches!(PressureStageV1::select(requested, &mut workspace), Err(error) if error.requested == requested)
        );
        assert_eq!(bits(workspace.pressure()), seeded);
    }
    let mut stage =
        PressureStageV1::select(ReplacementContract::PressureV1, &mut workspace).unwrap();
    let result = stage
        .solve(request, zero_iterations, Some(&mut candidate), || false)
        .unwrap();
    assert_eq!(bits(result.pressure()), bits(&pressure));
    same_report(&direct, result.report());
    assert_eq!(result.into_report().selected, "c1a-classical-test-not-ai");
}
