use super::*;
use crate::solver_provider::{PressureRequest, ProviderReport, ProviderWorkspace};
use crate::{Axis, PressureReport};
use std::mem::size_of;

const CAP: usize = 1024 * 1024;
const CANDIDATE_ID: ProjectionCandidateIdentity = ProjectionCandidateIdentity {
    algorithm: "classical-supplied-projection",
    version: "1",
    source_revision: "c1b1-direct-existing-dispatcher",
};

struct Fixture {
    grid: GridGeometry,
    source: SessionIdentity,
    velocity: [Vec<f32>; 3],
}
impl Fixture {
    fn nonzero() -> Self {
        let grid = GridGeometry::new([4, 3, 2], [0.25, 0.3, 0.4], [0.0; 3]).unwrap();
        let mut velocity = std::array::from_fn(|d| vec![0.0; grid.face_len(Axis::ALL[d])]);
        for axis in Axis::ALL {
            let d = axis.index();
            let shape = grid.face_counts(axis);
            for k in 0..shape[2] {
                for j in 0..shape[1] {
                    for i in 0..shape[0] {
                        let coordinate = [i, j, k];
                        if coordinate[d] != 0 && coordinate[d] != grid.counts()[d] {
                            let index = grid.face_index(axis, coordinate).unwrap();
                            velocity[d][index] = (0.001 * (0.3 * index as f64).sin()) as f32;
                        }
                    }
                }
            }
        }
        Self::with_fields(grid, velocity)
    }
    fn zero(value: f32) -> Self {
        let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
        let velocity = std::array::from_fn(|d| vec![value; grid.face_len(Axis::ALL[d])]);
        Self::with_fields(grid, velocity)
    }
    fn with_fields(grid: GridGeometry, velocity: [Vec<f32>; 3]) -> Self {
        Self {
            grid,
            source: SessionIdentity {
                run_id: 101,
                source_revision: "c1b1-frozen-reference".into(),
            },
            velocity,
        }
    }
    fn input(&self) -> ProjectionInput<'_> {
        ProjectionInput {
            grid: &self.grid,
            density: 1.25,
            interval_start: 0.0,
            dt: 0.01,
            source: &self.source,
            generation: 0,
            velocity: slices(&self.velocity),
            settings: PressureSettings {
                relative_residual: 1e-10,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 1000,
            },
            actual_divergence_limit: 1e-5,
            domain: ProjectionDomain::SealedStationaryBox,
        }
    }
}

fn slices(values: &[Vec<f32>; 3]) -> [&[f32]; 3] {
    std::array::from_fn(|d| values[d].as_slice())
}
fn bits(values: [&[f32]; 3]) -> [Vec<u32>; 3] {
    std::array::from_fn(|d| values[d].iter().map(|v| v.to_bits()).collect())
}
fn pressure_bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|v| v.to_bits()).collect()
}
fn diagnostics(report: &PressureReport) -> [u64; 3] {
    [
        report.true_residual_l2.to_bits(),
        report.true_residual_max.to_bits(),
        report.predicted_divergence_max.to_bits(),
    ]
}

struct DirectProjection {
    pressure: Vec<f64>,
    velocity: [Vec<f32>; 3],
    report: ProviderReport,
    actual_divergence_max: f64,
}
fn direct(input: &ProjectionInput<'_>, method: PressureImplementation) -> DirectProjection {
    let operator = PressureOperator::new(input.grid, input.density).unwrap();
    let mut rhs = vec![0.0; input.grid.cell_len()];
    operator
        .build_rhs(input.velocity, input.dt, &mut rhs)
        .unwrap();
    let mut workspace = ProviderWorkspace::new(input.grid, CAP, method).unwrap();
    let report = workspace
        .solve(
            PressureRequest {
                grid: input.grid,
                density: input.density,
                dt: input.dt,
                rhs: &rhs,
            },
            input.settings,
            None,
            || false,
        )
        .unwrap();
    let pressure = workspace.pressure().to_vec();
    let mut velocity = input.velocity.map(<[f32]>::to_vec);
    let [x, y, z] = &mut velocity;
    operator
        .correct_candidate_in_place(&pressure, input.dt, [x, y, z])
        .unwrap();
    operator.divergence(slices(&velocity), &mut rhs).unwrap();
    let actual_divergence_max = rhs.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
    DirectProjection {
        pressure,
        velocity,
        report,
        actual_divergence_max,
    }
}
fn supplied<'a>(
    request: &ProjectionRequest<'_>,
    direct: &'a DirectProjection,
) -> SuppliedProjection<'a> {
    SuppliedProjection {
        input_id: request.input_id(),
        representation: request.representation(),
        identity: CANDIDATE_ID,
        pressure: &direct.pressure,
        velocity: slices(&direct.velocity),
    }
}
fn assert_native(result: &QualifiedProjection<'_>, direct: &DirectProjection) {
    assert_eq!(
        pressure_bits(result.pressure()),
        pressure_bits(&direct.pressure)
    );
    assert_eq!(bits(result.velocity()), bits(slices(&direct.velocity)));
    assert_eq!(
        diagnostics(&result.report().pressure.accepted),
        diagnostics(&direct.report.accepted)
    );
    assert_eq!(
        result.report().pressure.accepted.iterations,
        direct.report.accepted.iterations
    );
    assert_eq!(
        result.report().actual_divergence_max.to_bits(),
        direct.actual_divergence_max.to_bits()
    );
}

#[test]
fn c1b1_native_and_supplied_match_existing_kernel_for_100_independent_requests() {
    let fixture = Fixture::nonzero();
    let original = bits(slices(&fixture.velocity));
    let operator = PressureOperator::new(&fixture.grid, 1.25).unwrap();
    let mut rhs = vec![0.0; fixture.grid.cell_len()];
    operator
        .build_rhs(slices(&fixture.velocity), 0.01, &mut rhs)
        .unwrap();
    assert!(rhs.iter().any(|value| *value != 0.0));
    for method in PressureImplementation::ALL {
        let oracle = direct(&fixture.input(), method);
        assert!(oracle.report.accepted.iterations > 0);
        let candidate_pressure = pressure_bits(&oracle.pressure);
        let candidate_velocity = bits(slices(&oracle.velocity));
        let mut workspace = ProjectionWorkspace::new(&fixture.grid, CAP, method).unwrap();
        let allocated = workspace.allocated_bytes();
        for generation in 0..100 {
            let mut input = fixture.input();
            input.generation = generation;
            input.interval_start = generation as f64 * 0.01;
            let request = ProjectionRequest::new(input).unwrap();
            {
                let result = workspace.qualify(&request, None, |_| false).unwrap();
                assert_native(&result, &oracle);
                assert_eq!(result.report().input_id, request.input_id());
                assert_eq!(
                    result.report().selection,
                    ProjectionSelection::NativeBaseline
                );
                assert_eq!(result.report().supplied_identity, None);
                assert_eq!(result.report().rejection, None);
                assert!(result.report().discarded_candidate_pressure.is_none());
                assert_eq!(result.report().pressure.selected, method.id());
                assert!(result.report().pressure.attempted.is_none());
            }
            let candidate = supplied(&request, &oracle);
            let result = workspace
                .qualify(&request, Some(&candidate), |_| false)
                .unwrap();
            assert_eq!(pressure_bits(result.pressure()), candidate_pressure);
            assert_eq!(bits(result.velocity()), candidate_velocity);
            assert_eq!(
                result.report().selection,
                ProjectionSelection::SuppliedCandidate
            );
            assert_eq!(result.report().input_id, request.input_id());
            assert_eq!(result.report().supplied_identity, Some(CANDIDATE_ID));
            assert_eq!(result.report().rejection, None);
            assert!(result.report().discarded_candidate_pressure.is_none());
            assert_eq!(result.report().pressure.accepted.iterations, 0);
            assert_eq!(
                diagnostics(&result.report().pressure.accepted),
                diagnostics(&oracle.report.accepted)
            );
            assert_eq!(
                result.report().actual_divergence_max.to_bits(),
                oracle.actual_divergence_max.to_bits()
            );
            let bridge = result.report().pressure.attempted.as_ref().unwrap();
            assert_eq!(result.report().pressure.selected, bridge.id);
            assert_eq!(bridge.id, "supplied-projection-pressure-witness-v1");
            assert!(bridge.provenance.is_none());
            assert_eq!(workspace.allocated_bytes(), allocated);
        }
        assert_eq!(pressure_bits(&oracle.pressure), candidate_pressure);
        assert_eq!(bits(slices(&oracle.velocity)), candidate_velocity);
    }
    assert_eq!(bits(slices(&fixture.velocity)), original);
}

#[test]
fn c1b1_divergence_free_wrong_dynamics_fail_the_exact_momentum_witness() {
    let fixture = Fixture::zero(0.0);
    let request = ProjectionRequest::new(fixture.input()).unwrap();
    for method in PressureImplementation::ALL {
        let oracle = direct(&fixture.input(), method);
        let mut wrong = oracle.velocity.clone();
        for (axis, coordinate, value) in [
            (Axis::X, [2, 1, 1], 0.125),
            (Axis::Y, [2, 2, 1], 0.125),
            (Axis::X, [2, 2, 1], -0.125),
            (Axis::Y, [1, 2, 1], -0.125),
        ] {
            wrong[axis.index()][fixture.grid.face_index(axis, coordinate).unwrap()] = value;
        }
        let mut divergence = vec![f64::NAN; fixture.grid.cell_len()];
        let operator = PressureOperator::new(&fixture.grid, 1.25).unwrap();
        operator
            .divergence(slices(&wrong), &mut divergence)
            .unwrap();
        assert!(divergence.iter().all(|value| *value == 0.0));
        assert_eq!(oracle.report.accepted.true_residual_l2, 0.0);
        let mut candidate = supplied(&request, &oracle);
        candidate.velocity = slices(&wrong);
        let original = bits(candidate.velocity);
        let mut workspace = ProjectionWorkspace::new(&fixture.grid, CAP, method).unwrap();
        let result = workspace
            .qualify(&request, Some(&candidate), |_| false)
            .unwrap();
        assert_native(&result, &oracle);
        assert_eq!(
            result.report().selection,
            ProjectionSelection::NativeFallback
        );
        assert_eq!(
            result.report().rejection,
            Some(ProjectionRejection::MomentumWitness {
                axis: Axis::X,
                index: fixture.grid.face_index(Axis::X, [2, 1, 1]).unwrap(),
            })
        );
        assert!(result.report().pressure.attempted.is_none());
        assert_eq!(bits(candidate.velocity), original);
    }
}

#[test]
fn c1b1_signed_zero_uses_in_place_correction_and_preserves_wall_bits() {
    let fixture = Fixture::zero(-0.0);
    let input_bits = bits(slices(&fixture.velocity));
    let request = ProjectionRequest::new(fixture.input()).unwrap();
    for method in PressureImplementation::ALL {
        let mut oracle = direct(&fixture.input(), method);
        oracle.pressure[0] = -0.0;
        oracle.velocity = corrected(&fixture.input(), &oracle.pressure);
        assert_eq!(oracle.velocity[0][0].to_bits(), (-0.0_f32).to_bits());
        let mut workspace = ProjectionWorkspace::new(&fixture.grid, CAP, method).unwrap();
        let good = supplied(&request, &oracle);
        {
            let result = workspace.qualify(&request, Some(&good), |_| false).unwrap();
            assert_eq!(
                result.report().selection,
                ProjectionSelection::SuppliedCandidate
            );
            assert_eq!(
                pressure_bits(result.pressure()),
                pressure_bits(&oracle.pressure)
            );
            assert_eq!(bits(result.velocity()), bits(slices(&oracle.velocity)));
        }
        let mut wrong = oracle.velocity.clone();
        wrong[0][0] = 0.0;
        let mut bad = supplied(&request, &oracle);
        bad.velocity = slices(&wrong);
        let result = workspace.qualify(&request, Some(&bad), |_| false).unwrap();
        assert_eq!(
            result.report().selection,
            ProjectionSelection::NativeFallback
        );
        assert_eq!(
            result.report().rejection,
            Some(ProjectionRejection::MomentumWitness {
                axis: Axis::X,
                index: 0
            })
        );
        assert_eq!(result.velocity()[0][0].to_bits(), (-0.0_f32).to_bits());
    }
    assert_eq!(bits(slices(&fixture.velocity)), input_bits);
}

fn corrected(input: &ProjectionInput<'_>, pressure: &[f64]) -> [Vec<f32>; 3] {
    let mut velocity = input.velocity.map(<[f32]>::to_vec);
    let [x, y, z] = &mut velocity;
    PressureOperator::new(input.grid, input.density)
        .unwrap()
        .correct_candidate_in_place(pressure, input.dt, [x, y, z])
        .unwrap();
    velocity
}

fn changed_binding(input: ProjectionInput<'_>, original: &SuppliedProjection<'_>) {
    let mut workspace =
        ProjectionWorkspace::new(input.grid, CAP, PressureImplementation::JacobiPcgV1).unwrap();
    let request = ProjectionRequest::new(input).unwrap();
    assert_ne!(request.input_id(), original.input_id);
    assert_eq!(
        workspace
            .qualify(&request, Some(original), |_| panic!(
                "header refusal called checkpoint"
            ))
            .err()
            .unwrap(),
        ProjectionError::Refused(ProjectionRefusal::InputBinding)
    );
}

#[test]
fn c1b1_input_identity_binds_every_actual_input_fact_and_exact_face_bits() {
    let fixture = Fixture::nonzero();
    let original = ProjectionRequest::new(fixture.input()).unwrap();
    let oracle = direct(&fixture.input(), PressureImplementation::JacobiPcgV1);
    let candidate = supplied(&original, &oracle);
    for field in 0..9 {
        let mut input = fixture.input();
        match field {
            0 => input.dt *= 2.0,
            1 => input.interval_start = 0.125,
            2 => input.generation = 1,
            3 => input.density *= 2.0,
            4 => input.settings.relative_residual *= 2.0,
            5 => input.settings.absolute_residual *= 2.0,
            6 => input.settings.divergence_limit *= 2.0,
            7 => input.settings.max_iterations += 1,
            8 => input.actual_divergence_limit *= 2.0,
            _ => unreachable!(),
        }
        changed_binding(input, &candidate);
    }
    for source in [
        SessionIdentity {
            run_id: 102,
            source_revision: fixture.source.source_revision.clone(),
        },
        SessionIdentity {
            run_id: 101,
            source_revision: "another-declared-revision".into(),
        },
    ] {
        let mut input = fixture.input();
        input.source = &source;
        changed_binding(input, &candidate);
    }
    for grid in [
        GridGeometry::new([4, 3, 2], [0.25, 0.3, 0.4], [0.1, 0.0, 0.0]).unwrap(),
        GridGeometry::new([4, 3, 2], [0.26, 0.3, 0.4], [0.0; 3]).unwrap(),
    ] {
        let mut input = fixture.input();
        input.grid = &grid;
        changed_binding(input, &candidate);
    }
    let mut velocity = fixture.velocity.clone();
    let index = fixture.grid.face_index(Axis::X, [1, 0, 0]).unwrap();
    velocity[0][index] = f32::from_bits(velocity[0][index].to_bits() + 1);
    let mut input = fixture.input();
    input.velocity = slices(&velocity);
    changed_binding(input, &candidate);
    // Signed zero is a distinct actual input even though native validation accepts both.
    let mut velocity = fixture.velocity.clone();
    velocity[0][0] = -0.0;
    let mut input = fixture.input();
    input.velocity = slices(&velocity);
    changed_binding(input, &candidate);
    let grid = GridGeometry::new([5, 3, 2], [0.25, 0.3, 0.4], [0.0; 3]).unwrap();
    let velocity = std::array::from_fn(|d| vec![0.0; grid.face_len(Axis::ALL[d])]);
    let changed = Fixture::with_fields(grid, velocity);
    changed_binding(changed.input(), &candidate);
    assert_eq!(
        ProjectionRequest::new(fixture.input()).unwrap().input_id(),
        original.input_id()
    );
}

#[test]
fn c1b1_header_representation_identity_and_domain_refuse_before_callbacks() {
    let fixture = Fixture::nonzero();
    let request = ProjectionRequest::new(fixture.input()).unwrap();
    let oracle = direct(&fixture.input(), PressureImplementation::JacobiPcgV1);
    let mut workspace =
        ProjectionWorkspace::new(&fixture.grid, CAP, PressureImplementation::JacobiPcgV1).unwrap();
    for field in 0..18 {
        let mut candidate = supplied(&request, &oracle);
        match field {
            0 => candidate.representation.version += 1,
            1 => candidate.representation.operator = "unsupported-operator",
            _ => {
                let field = field - 2;
                let metadata = if field / 4 == 0 {
                    &mut candidate.representation.pressure
                } else {
                    &mut candidate.representation.velocity[field / 4 - 1]
                };
                match field % 4 {
                    0 => metadata.shape_xyz[0] += 1,
                    1 => metadata.layout = "unsupported-layout",
                    2 => metadata.scalar = "unsupported-scalar",
                    3 => metadata.units = "unsupported-units",
                    _ => unreachable!(),
                }
            }
        }
        assert_eq!(
            workspace
                .qualify(&request, Some(&candidate), |_| panic!(
                    "representation refusal called checkpoint"
                ))
                .err()
                .unwrap(),
            ProjectionError::Refused(ProjectionRefusal::Representation)
        );
    }
    for identity in [
        ProjectionCandidateIdentity {
            algorithm: "",
            ..CANDIDATE_ID
        },
        ProjectionCandidateIdentity {
            version: "bad\nversion",
            ..CANDIDATE_ID
        },
        ProjectionCandidateIdentity {
            source_revision: "",
            ..CANDIDATE_ID
        },
        ProjectionCandidateIdentity {
            algorithm: concat!(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "a"
            ),
            ..CANDIDATE_ID
        },
    ] {
        let mut candidate = supplied(&request, &oracle);
        candidate.identity = identity;
        assert_eq!(
            workspace
                .qualify(&request, Some(&candidate), |_| panic!(
                    "identity refusal called checkpoint"
                ))
                .err()
                .unwrap(),
            ProjectionError::Refused(ProjectionRefusal::InvalidIdentity)
        );
    }
    for domain in [
        ProjectionDomain::FreeSurface,
        ProjectionDomain::EmbeddedObstacle,
        ProjectionDomain::MovingWall,
    ] {
        let mut input = fixture.input();
        input.domain = domain;
        assert_eq!(
            ProjectionRequest::new(input).err().unwrap(),
            ProjectionError::Refused(ProjectionRefusal::UnsupportedDomain(domain))
        );
    }
    let other = Fixture::zero(0.0);
    let other_request = ProjectionRequest::new(other.input()).unwrap();
    assert_eq!(
        workspace
            .qualify(&other_request, None, |_| panic!(
                "grid refusal called checkpoint"
            ))
            .err()
            .unwrap(),
        ProjectionError::Refused(ProjectionRefusal::WorkspaceGrid)
    );
    let result = workspace.qualify(&request, None, |_| false).unwrap();
    assert_native(&result, &oracle);
}

#[test]
fn c1b1_candidate_numerical_failures_retain_typed_reasons_and_clean_native_fallback() {
    let fixture = Fixture::nonzero();
    let request = ProjectionRequest::new(fixture.input()).unwrap();
    let input_before = bits(slices(&fixture.velocity));
    let interior = fixture.grid.face_index(Axis::X, [1, 0, 0]).unwrap();
    for method in PressureImplementation::ALL {
        let oracle = direct(&fixture.input(), method);
        let mut workspace = ProjectionWorkspace::new(&fixture.grid, CAP, method).unwrap();
        for case in 0..8 {
            let mut pressure = oracle.pressure.clone();
            let mut velocity = oracle.velocity.clone();
            let reason = match case {
                0 => {
                    pressure.pop();
                    ProjectionRejection::Field(OperatorError::LengthMismatch)
                }
                1 => {
                    pressure[1] = f64::NAN;
                    ProjectionRejection::Field(OperatorError::NonFiniteInput)
                }
                2 => {
                    velocity[0].pop();
                    ProjectionRejection::Field(OperatorError::LengthMismatch)
                }
                3 => {
                    velocity[0][interior] = f32::NAN;
                    ProjectionRejection::Field(OperatorError::NonFiniteInput)
                }
                4 => {
                    velocity[0][0] = 1.0;
                    ProjectionRejection::Field(OperatorError::NonZeroWallVelocity {
                        axis: Axis::X,
                        index: 0,
                    })
                }
                5 => {
                    pressure.fill(0.0);
                    velocity = corrected(&fixture.input(), &pressure);
                    ProjectionRejection::Pressure(ProviderRejection::QualityGate)
                }
                6 => {
                    pressure.fill(1.0);
                    velocity = corrected(&fixture.input(), &pressure);
                    ProjectionRejection::Pressure(ProviderRejection::NonZeroGauge)
                }
                7 => {
                    velocity[0][interior] = f32::from_bits(velocity[0][interior].to_bits() ^ 1);
                    ProjectionRejection::MomentumWitness {
                        axis: Axis::X,
                        index: interior,
                    }
                }
                _ => unreachable!(),
            };
            let candidate = SuppliedProjection {
                pressure: &pressure,
                velocity: slices(&velocity),
                ..supplied(&request, &oracle)
            };
            let candidate_pressure = pressure_bits(candidate.pressure);
            let candidate_velocity = bits(candidate.velocity);
            {
                let result = workspace
                    .qualify(&request, Some(&candidate), |_| false)
                    .unwrap();
                assert_native(&result, &oracle);
                assert_eq!(
                    result.report().selection,
                    ProjectionSelection::NativeFallback,
                    "case {case}"
                );
                assert_eq!(result.report().supplied_identity, Some(CANDIDATE_ID));
                assert_eq!(
                    result.report().rejection,
                    Some(reason.clone()),
                    "case {case}"
                );
                assert_eq!(result.report().pressure.selected, method.id());
                if let ProjectionRejection::Pressure(pressure_reason) = reason {
                    assert_eq!(result.report().pressure.rejection, Some(pressure_reason));
                    assert!(result.report().pressure.attempted.is_some());
                    if case == 5 {
                        assert!(result.report().pressure.candidate.unwrap().true_residual_l2 > 0.0);
                    }
                } else {
                    assert!(result.report().pressure.attempted.is_none());
                    assert_eq!(result.report().pressure.rejection, None);
                }
            }
            assert_eq!(pressure_bits(candidate.pressure), candidate_pressure);
            assert_eq!(bits(candidate.velocity), candidate_velocity);
            let good = supplied(&request, &oracle);
            let result = workspace.qualify(&request, Some(&good), |_| false).unwrap();
            assert_eq!(
                result.report().selection,
                ProjectionSelection::SuppliedCandidate
            );
            assert_eq!(result.report().rejection, None);
            assert_eq!(bits(result.velocity()), bits(slices(&oracle.velocity)));
        }
    }
    assert_eq!(bits(slices(&fixture.velocity)), input_before);
}

#[test]
fn c1b1_invalid_actual_inputs_are_typed_constructor_errors() {
    let fixture = Fixture::nonzero();
    for field in 0..15 {
        let mut input = fixture.input();
        match field {
            0 => input.dt = 0.0,
            1 => input.dt = f64::NAN,
            2 => input.interval_start = -1.0,
            3 => input.interval_start = f64::INFINITY,
            4 => {
                input.interval_start = f64::MAX;
                input.dt = f64::MAX;
            }
            5 => input.settings.relative_residual = -1.0,
            6 => input.settings.relative_residual = f64::NAN,
            7 => input.settings.absolute_residual = -1.0,
            8 => input.settings.absolute_residual = f64::INFINITY,
            9 => input.settings.divergence_limit = -1.0,
            10 => input.settings.divergence_limit = f64::NAN,
            11 => input.settings.max_iterations = 0,
            12 => input.actual_divergence_limit = -1.0,
            13 => input.actual_divergence_limit = f64::INFINITY,
            14 => input.interval_start = f64::MAX,
            _ => unreachable!(),
        }
        assert_eq!(
            ProjectionRequest::new(input).err().unwrap(),
            ProjectionError::InvalidInput,
            "field {field}"
        );
    }
    for density in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut input = fixture.input();
        input.density = density;
        assert_eq!(
            ProjectionRequest::new(input).err().unwrap(),
            ProjectionError::Operator(OperatorError::InvalidDensity)
        );
    }
    for revision in ["", "bad\nrevision"] {
        let source = SessionIdentity {
            run_id: 101,
            source_revision: revision.into(),
        };
        let mut input = fixture.input();
        input.source = &source;
        assert_eq!(
            ProjectionRequest::new(input).err().unwrap(),
            ProjectionError::Refused(ProjectionRefusal::InvalidIdentity)
        );
    }
    for case in 0..3 {
        let mut velocity = fixture.velocity.clone();
        let error = match case {
            0 => {
                velocity[0].pop();
                OperatorError::LengthMismatch
            }
            1 => {
                velocity[0][1] = f32::NAN;
                OperatorError::NonFiniteInput
            }
            2 => {
                velocity[0][0] = 1.0;
                OperatorError::NonZeroWallVelocity {
                    axis: Axis::X,
                    index: 0,
                }
            }
            _ => unreachable!(),
        };
        let mut input = fixture.input();
        input.velocity = slices(&velocity);
        assert_eq!(
            ProjectionRequest::new(input).err().unwrap(),
            ProjectionError::Operator(error)
        );
    }
}

#[test]
fn c1b1_cooperative_cancellation_returns_no_view_and_allows_clean_retry() {
    let fixture = Fixture::nonzero();
    let request = ProjectionRequest::new(fixture.input()).unwrap();
    let before = bits(slices(&fixture.velocity));
    for method in PressureImplementation::ALL {
        let oracle = direct(&fixture.input(), method);
        let candidate = supplied(&request, &oracle);
        let candidate_pressure = pressure_bits(candidate.pressure);
        let candidate_velocity = bits(candidate.velocity);
        let mut workspace = ProjectionWorkspace::new(&fixture.grid, CAP, method).unwrap();
        for with_candidate in [false, true] {
            for target in [
                ProjectionCheckpoint::BeforeQualification,
                ProjectionCheckpoint::BeforePressure,
                ProjectionCheckpoint::PressurePoll,
                ProjectionCheckpoint::AfterPressure,
                ProjectionCheckpoint::BeforeReturn,
            ] {
                let mut seen = Vec::new();
                let error = workspace
                    .qualify(&request, with_candidate.then_some(&candidate), |point| {
                        seen.push(point);
                        point == target
                    })
                    .err()
                    .unwrap();
                assert_eq!(seen.last(), Some(&target));
                if target == ProjectionCheckpoint::PressurePoll {
                    assert_eq!(
                        error,
                        ProjectionError::Pressure(PressureError::Cancelled { iterations: 0 })
                    );
                } else {
                    assert_eq!(error, ProjectionError::Cancelled);
                }
                let result = workspace.qualify(&request, None, |_| false).unwrap();
                assert_native(&result, &oracle);
                assert_eq!(
                    result.report().selection,
                    ProjectionSelection::NativeBaseline
                );
            }
        }
        // Dispatcher poll, numerical entry poll, iteration-1 poll, then
        // iteration-2 poll. This cancels inside the existing numerical solve.
        assert!(oracle.report.accepted.iterations > 1);
        let mut polls = 0;
        let error = workspace
            .qualify(&request, None, |point| {
                if point == ProjectionCheckpoint::PressurePoll {
                    polls += 1;
                }
                point == ProjectionCheckpoint::PressurePoll && polls == 4
            })
            .err()
            .unwrap();
        assert_eq!(polls, 4);
        assert_eq!(
            error,
            ProjectionError::Pressure(PressureError::Cancelled { iterations: 1 })
        );
        let result = workspace
            .qualify(&request, Some(&candidate), |_| false)
            .unwrap();
        assert_eq!(
            result.report().selection,
            ProjectionSelection::SuppliedCandidate
        );
        assert_eq!(bits(result.velocity()), candidate_velocity);
        assert_eq!(pressure_bits(candidate.pressure), candidate_pressure);
        assert_eq!(bits(candidate.velocity), candidate_velocity);
    }
    assert_eq!(bits(slices(&fixture.velocity)), before);
}

#[test]
fn c1b1_native_iteration_failure_and_actual_divergence_failure_are_not_qualified() {
    let fixture = Fixture::nonzero();
    let original_bits = bits(slices(&fixture.velocity));
    for method in PressureImplementation::ALL {
        let oracle = direct(&fixture.input(), method);
        assert!(oracle.report.accepted.iterations > 1);
        let mut workspace = ProjectionWorkspace::new(&fixture.grid, CAP, method).unwrap();
        let mut bounded_input = fixture.input();
        bounded_input.settings.max_iterations = 1;
        let bounded_request = ProjectionRequest::new(bounded_input).unwrap();
        let mut wrong_velocity = oracle.velocity.clone();
        wrong_velocity[0][0] = 1.0;
        let mut bad = supplied(&bounded_request, &oracle);
        bad.velocity = slices(&wrong_velocity);
        for supplied in [None, Some(&bad)] {
            let error = workspace
                .qualify(&bounded_request, supplied, |_| false)
                .err()
                .unwrap();
            match error {
                ProjectionError::Pressure(PressureError::IterationLimit {
                    iterations,
                    residual_max,
                }) => {
                    assert_eq!(iterations, 1);
                    assert!(residual_max.is_finite() && residual_max > 0.0);
                }
                other => panic!("expected native iteration exhaustion, got {other:?}"),
            }
        }
        // The same disposable workspace can still accept a qualified supplied
        // result; max_iterations bounds native solving, not candidate acceptance.
        let good = supplied(&bounded_request, &oracle);
        {
            let result = workspace
                .qualify(&bounded_request, Some(&good), |_| false)
                .unwrap();
            assert_eq!(
                result.report().selection,
                ProjectionSelection::SuppliedCandidate
            );
            assert_eq!(result.report().pressure.accepted.iterations, 0);
        }
        let mut tight_input = fixture.input();
        tight_input.settings.relative_residual = 1.0;
        tight_input.settings.absolute_residual = 1.0;
        tight_input.settings.divergence_limit = 1.0;
        tight_input.actual_divergence_limit = 0.0;
        let loose_oracle = direct(&tight_input, method);
        assert_eq!(loose_oracle.report.accepted.iterations, 0);
        assert!(loose_oracle.actual_divergence_max > 0.0);
        let tight_request = ProjectionRequest::new(tight_input).unwrap();
        let candidate = supplied(&tight_request, &loose_oracle);
        for with_candidate in [false, true] {
            let mut solves = 0;
            let error = workspace
                .qualify(
                    &tight_request,
                    with_candidate.then_some(&candidate),
                    |point| {
                        if point == ProjectionCheckpoint::AfterPressure {
                            solves += 1;
                        }
                        false
                    },
                )
                .err()
                .unwrap();
            assert_eq!(
                error,
                ProjectionError::DivergenceExceeded {
                    maximum: loose_oracle.actual_divergence_max,
                    limit: 0.0
                }
            );
            // Candidate pressure qualified; the actual-f32 gate retried native,
            // whose identical correction cannot meet the zero actual limit.
            assert_eq!(solves, if with_candidate { 2 } else { 1 });
        }
        let request = ProjectionRequest::new(fixture.input()).unwrap();
        let result = workspace.qualify(&request, None, |_| false).unwrap();
        assert_native(&result, &oracle);
        assert_eq!(result.report().rejection, None);
    }
    assert_eq!(bits(slices(&fixture.velocity)), original_bits);
}

#[test]
fn c1b1_workspace_cap_includes_all_managed_array_capacities() {
    let fixture = Fixture::nonzero();
    for method in PressureImplementation::ALL {
        let pressure = ProviderWorkspace::new(&fixture.grid, CAP, method).unwrap();
        let host_bytes = 2 * fixture.grid.cell_len() * size_of::<f64>()
            + Axis::ALL
                .into_iter()
                .map(|axis| fixture.grid.face_len(axis) * size_of::<f32>())
                .sum::<usize>();
        let workspace = ProjectionWorkspace::new(&fixture.grid, CAP, method).unwrap();
        let required = workspace.allocated_bytes();
        assert_eq!(required, pressure.allocated_bytes() + host_bytes);
        assert!(required <= CAP);
        assert_eq!(
            ProjectionWorkspace::new(&fixture.grid, required, method)
                .unwrap()
                .allocated_bytes(),
            required
        );
        let error = ProjectionWorkspace::new(&fixture.grid, required - 1, method)
            .err()
            .unwrap();
        match error {
            ProjectionError::BufferLimit { required, limit }
            | ProjectionError::Pressure(PressureError::BufferLimit { required, limit }) => {
                assert!(required > limit);
            }
            other => panic!("expected aggregate cap rejection, got {other:?}"),
        }
        assert!(matches!(
            ProjectionWorkspace::new(&fixture.grid, 0, method)
                .err()
                .unwrap(),
            ProjectionError::BufferLimit { limit: 0, .. }
        ));
    }
}

#[test]
fn c1b1_actual_divergence_rejection_retains_both_pressure_calls_and_native_fallback() {
    let fixture = Fixture::zero(0.0);
    let mut input = fixture.input();
    input.actual_divergence_limit = 0.0;
    let request = ProjectionRequest::new(input).unwrap();
    let input_before = bits(slices(&fixture.velocity));
    let mut pressure = vec![0.0; fixture.grid.cell_len()];
    pressure[1] = 1e-13;
    let velocity = corrected(&fixture.input(), &pressure);
    let mut divergence = vec![0.0; fixture.grid.cell_len()];
    PressureOperator::new(&fixture.grid, 1.25)
        .unwrap()
        .divergence(slices(&velocity), &mut divergence)
        .unwrap();
    let candidate_divergence = divergence.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
    assert!(candidate_divergence > 0.0);
    let candidate = SuppliedProjection {
        input_id: request.input_id(),
        representation: request.representation(),
        identity: CANDIDATE_ID,
        pressure: &pressure,
        velocity: slices(&velocity),
    };
    let pressure_before = pressure_bits(candidate.pressure);
    let velocity_before = bits(candidate.velocity);
    for method in PressureImplementation::ALL {
        let native = direct(&fixture.input(), method);
        let mut workspace = ProjectionWorkspace::new(&fixture.grid, CAP, method).unwrap();
        let mut calls = 0;
        let result = workspace
            .qualify(&request, Some(&candidate), |point| {
                if point == ProjectionCheckpoint::AfterPressure {
                    calls += 1;
                }
                false
            })
            .unwrap();
        assert_eq!(calls, 2);
        assert_native(&result, &native);
        assert_eq!(
            result.report().selection,
            ProjectionSelection::NativeFallback
        );
        assert_eq!(result.report().supplied_identity, Some(CANDIDATE_ID));
        assert_eq!(
            result.report().rejection,
            Some(ProjectionRejection::ActualDivergence {
                maximum: candidate_divergence,
                limit: 0.0
            })
        );
        assert_eq!(result.report().actual_divergence_max, 0.0);
        assert_eq!(result.report().pressure.selected, method.id());
        assert!(result.report().pressure.attempted.is_none());
        let discarded = result
            .report()
            .discarded_candidate_pressure
            .as_ref()
            .unwrap();
        assert_eq!(
            discarded.selected,
            "supplied-projection-pressure-witness-v1"
        );
        assert!(discarded.attempted.is_some());
        assert_eq!(discarded.rejection, None);
        assert_eq!(discarded.accepted.iterations, 0);
        assert!(discarded.accepted.true_residual_l2 > 0.0);
        assert!(discarded.accepted.true_residual_l2 < 1e-12);
        assert!(discarded.accepted.predicted_divergence_max <= 1e-8);
        assert_eq!(
            diagnostics(&discarded.accepted),
            diagnostics(&discarded.candidate.unwrap())
        );
        assert_eq!(pressure_bits(candidate.pressure), pressure_before);
        assert_eq!(bits(candidate.velocity), velocity_before);
    }
    assert_eq!(bits(slices(&fixture.velocity)), input_before);
}
