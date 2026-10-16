//! Cold configured/native comparison using actual Session/SQLite producers.
//! Runtime observations and package pins are authored metadata, not IPC/model
//! evidence. Mutated schema/face fixtures establish reader/arithmetic contracts
//! only; they do not establish physical acceptance of the authored field bytes.
#![cfg(not(target_arch = "wasm32"))]

use rheon::{
    recording::{
        comparison::*,
        diagnostic::NoCandidateReason,
        provider::{
            self, RecordedConfiguredSession, RecordedPressureSelection, RecordedProviderRejection,
            comparison::{self as facts, RhsRelation},
            runtime::{RuntimeObservation, RuntimeOutcome, RuntimeRequestStamp},
        },
        *,
    },
    solver_provider::{
        ProviderRejection,
        configured::{ContextPressureProvider, ContextPressureRequest, ContextProviderIdentity},
    },
    *,
};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

const CAP: usize = 4 * 1024 * 1024;
const DT: f64 = 1.0 / 16.0;
const RHO: f64 = 2.0;
const H: f64 = 0.25;
const FORCE: f64 = 0.125;
const SOURCE: &str = "configured-comparison-authored-producer";
const PHASE: &str = "RHEON_CONFIGURED_COMPARISON_PHASE";
const ROOT: &str = "RHEON_CONFIGURED_COMPARISON_ROOT";
const PROCESS_TEST: &str = "producer_exits_before_model_absent_configured_comparison_reader";

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rheon-configured-comparison-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn grid() -> GridGeometry {
    GridGeometry::new([2, 1, 1], [H; 3], [0.0; 3]).unwrap()
}
fn config(adaptive: bool) -> SimulationConfig {
    SimulationConfig {
        density: RHO,
        memory_limit: CAP,
        pressure: PressureSettings {
            relative_residual: 1e-12,
            absolute_residual: 1e-12,
            divergence_limit: 1e-9,
            max_iterations: 100,
        },
        actual_divergence_limit: 1e-5,
        max_courant: if adaptive { 1.0 / 128.0 } else { 0.5 },
    }
}
fn identity() -> ContextProviderIdentity {
    ContextProviderIdentity {
        id: "authored-two-cell-equation".into(),
        version: "1".into(),
        manifest_sha256: [31; 32],
        checkpoint_sha256: Some([32; 32]),
        training_run_id: None,
        runtime: "in-process equation; explicitly authored runtime observations; no IPC".into(),
        source_revision: "authored-fixture-only".into(),
    }
}
#[derive(Clone, Copy)]
enum Behavior {
    Candidate,
    QualityFallback,
    RuntimeFailure,
    InvalidOutput,
    SupportDecline,
}
struct Candidate {
    binding: Arc<ContextProviderIdentity>,
    behavior: Behavior,
    observation: Option<RuntimeObservation>,
}
impl ContextPressureProvider for Candidate {
    fn identity(&self) -> Arc<ContextProviderIdentity> {
        Arc::clone(&self.binding)
    }
    fn supports(&self, _: &ContextPressureRequest<'_>) -> Result<(), ProviderRejection> {
        if matches!(self.behavior, Behavior::SupportDecline) {
            Err(ProviderRejection::Unsupported("authored support refusal"))
        } else {
            Ok(())
        }
    }
    fn predict(
        &mut self,
        request: &ContextPressureRequest<'_>,
        out: &mut [f64],
        _: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        let stamp = RuntimeRequestStamp::capture(request, &self.binding).unwrap();
        self.observation = Some(RuntimeObservation {
            request: stamp,
            outcome: match self.behavior {
                Behavior::RuntimeFailure | Behavior::InvalidOutput => RuntimeOutcome::NoCandidate {
                    reason: if matches!(self.behavior, Behavior::InvalidOutput) {
                        NoCandidateReason::InvalidOutput
                    } else {
                        NoCandidateReason::RuntimeFailure
                    },
                    detail: "authored metadata fixture, not a runtime/payload observation".into(),
                    dispatched: matches!(self.behavior, Behavior::InvalidOutput),
                    invalid_payload_sha256: matches!(self.behavior, Behavior::InvalidOutput)
                        .then_some([19; 32]),
                },
                _ => RuntimeOutcome::CandidateReturned,
            },
        });
        if matches!(
            self.behavior,
            Behavior::RuntimeFailure | Behavior::InvalidOutput
        ) {
            return Err(ProviderRejection::Execution(
                "authored runtime/output failure",
            ));
        }
        // Independent two-cell equation: A*p=(-h*p1/rho,h*p1/rho).
        let c = &request.context;
        assert_eq!(c.rhs()[0], -c.rhs()[1]);
        out[0] = 0.0;
        out[1] = c.rhs()[1] * c.density() / H;
        if matches!(self.behavior, Behavior::QualityFallback) {
            out[1] += 100.0;
        }
        Ok(())
    }
}
struct Capture {
    run: RecordingRun,
    rows: Vec<RecordedStep>,
}
fn configured(id: u64, force: f64, behavior: Behavior, runtime: bool, adaptive: bool) -> Capture {
    configured_on_grid(id, force, behavior, runtime, adaptive, grid())
}
fn configured_on_grid(
    id: u64,
    force: f64,
    behavior: Behavior,
    runtime: bool,
    adaptive: bool,
    geometry: GridGeometry,
) -> Capture {
    configured_intervals(id, force, behavior, runtime, adaptive, geometry, 3)
}
fn configured_intervals(
    id: u64,
    force: f64,
    behavior: Behavior,
    runtime: bool,
    adaptive: bool,
    geometry: GridGeometry,
    intervals: usize,
) -> Capture {
    assert_ne!(
        std::env::var(PHASE).ok().as_deref(),
        Some("inspect"),
        "cold reader constructed Session/provider"
    );
    let mut s = SimulationSession::new(
        SessionIdentity {
            run_id: id,
            source_revision: SOURCE.into(),
        },
        geometry.clone(),
        LiquidTransportConfig {
            carrier: config(adaptive),
            represented_density: RHO,
            volume_stamp: VolumeStamp {
                id: 901,
                version: 0,
            },
            carrier_id: 902,
        },
        PressureImplementation::JacobiPcgV1,
        vec![1.0; 2],
        CAP,
        intervals.max(8),
    )
    .unwrap();
    let run = provider::configured_run(
        &s.snapshot(),
        "actual Session; authored provider/runtime metadata",
    )
    .unwrap();
    let mut rows = vec![provider::configured_publication(&s.snapshot(), 0, CAP).unwrap()];
    let mut workspace =
        ProviderStepWorkspace::new(&geometry, RHO, CAP, PressureImplementation::JacobiPcgV1)
            .unwrap();
    let mut candidate = Candidate {
        binding: Arc::new(identity()),
        behavior,
        observation: None,
    };
    let forces = [BodyForce {
        value: [force, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    for interval in 0..intervals {
        candidate.observation = None;
        let requested_dt = if adaptive && interval == 1 { 2.0 } else { DT };
        s.step_with_configured_pressure(
            LiquidStepInputs {
                requested_dt,
                smoke_source: None,
                forces: &forces,
                inlet: LiquidInlet::new(
                    VolumeStamp {
                        id: 903,
                        version: 0,
                    },
                    [[0.0; 2]; 3],
                )
                .unwrap(),
                source: None,
                volume: LiquidVolumeSettings {
                    max_outward_courant: 1.0,
                    actual_divergence_limit: 1e-5,
                },
            },
            &mut workspace,
            None,
            (interval > 0).then_some(&mut candidate as &mut dyn ContextPressureProvider),
            None,
            |_| false,
        )
        .unwrap();
        rows.push(
            provider::configured_publication_with_runtime(
                &s.snapshot(),
                rows.len(),
                CAP,
                if runtime {
                    candidate.observation.as_ref()
                } else {
                    None
                },
            )
            .unwrap(),
        );
    }
    Capture { run, rows }
}
fn native(id: u64) -> Capture {
    assert_ne!(
        std::env::var(PHASE).ok().as_deref(),
        Some("inspect"),
        "cold reader constructed Simulation"
    );
    let mut s = Simulation::new(grid(), config(false)).unwrap();
    let run = carrier_run(
        SessionIdentity {
            run_id: id,
            source_revision: SOURCE.into(),
        },
        &s,
        "actual native force producer",
    )
    .unwrap();
    let mut rows = vec![carrier_publication(&s, 0, CAP).unwrap()];
    for _ in 0..3 {
        // This existing success-qualified path owns carrier publication stamps.
        // It is an actual rest/no-force native source, not a relabelled Session.
        s.step(DT, None, |_| false).unwrap();
        rows.push(carrier_publication(&s, rows.len(), CAP).unwrap());
    }
    Capture { run, rows }
}
fn write(root: &Path, name: &str, capture: Capture) {
    let last_generation = capture.rows.last().map(|row| row.generation);
    let mut writer =
        StepRecorder::create(&root.join(name), capture.run, RecordingLimits::default()).unwrap();
    for row in &capture.rows {
        writer.append(row).unwrap();
    }
    writer
        .finish(
            "completed",
            "generic terminal; comparison has no typed terminal custody",
            last_generation,
        )
        .unwrap();
}
fn reader(root: &Path, name: &str) -> RecordingReader {
    RecordingReader::open(&root.join(name), RecordingLimits::default()).unwrap()
}
fn selected(root: &Path, name: &str, ordinal: usize) -> Selection {
    Selection::read(&reader(root, name), ordinal).unwrap()
}
fn recipe(field: FieldRole) -> Recipe {
    Recipe {
        field,
        time_tolerance: 0.0,
        pressure: PressurePolicy::Canonical,
    }
}
fn all(root: &Path, name: &str) -> Vec<Selection> {
    let r = reader(root, name);
    (0..r.count().unwrap())
        .map(|i| Selection::read(&r, i).unwrap())
        .collect()
}
fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 2e-13 * b.abs().max(1.0),
        "actual {a:?}, expected {b:?}"
    );
}
fn field_mut<'a>(row: &'a mut RecordedStep, name: &str) -> &'a mut RecordedField {
    row.fields.iter_mut().find(|f| f.name == name).unwrap()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn real_native_configured_and_configured_pairs_keep_profiles_and_nonempty_interval_matches() {
    let d = Directory::new();
    write(&d.0, "native", native(1));
    write(
        &d.0,
        "configured",
        configured(2, 0.0, Behavior::Candidate, true, false),
    );
    write(
        &d.0,
        "rcp2",
        configured(3, 0.0, Behavior::Candidate, false, false),
    );
    for candidate in ["configured", "rcp2"] {
        let a = all(&d.0, "native");
        let b = all(&d.0, candidate);
        assert!(a[2].run().solver.starts_with("rheon-carrier-full-box-v1/"));
        assert!(b[2].run().solver.starts_with(provider::PROFILE));
        let coverage = align(&a[1..], &b[1..], recipe(FieldRole::IntervalPressure)).unwrap();
        assert_eq!(coverage.pairs.len(), 3);
        assert!(coverage.reference_unmatched.is_empty());
        assert!(coverage.candidate_unmatched.is_empty());
        for pair in coverage.pairs {
            assert_eq!(pair.outcome.unwrap().metrics.l2, 0.0);
        }
        assert_eq!(
            facts::rhs_relation(&a[2], &b[2]).unwrap(),
            RhsRelation::Unavailable
        );
    }
    let a = selected(&d.0, "configured", 2);
    let b = selected(&d.0, "rcp2", 2);
    assert_eq!(
        compare(&a, &b, recipe(FieldRole::IntervalPressure))
            .unwrap()
            .outcome
            .unwrap()
            .metrics
            .l2,
        0.0
    );
    assert!(facts::inspect(&a).unwrap().runtime.is_some());
    assert!(facts::inspect(&b).unwrap().runtime.is_none());
    assert_eq!(
        facts::rhs_relation(&a, &b).unwrap(),
        RhsRelation::EqualHashes
    );
}

#[test]
fn independently_closed_form_pressure_metrics_do_not_equate_time_and_rhs() {
    let d = Directory::new();
    write(
        &d.0,
        "a",
        configured(10, FORCE, Behavior::Candidate, true, false),
    );
    write(
        &d.0,
        "b",
        configured(11, 2.0 * FORCE, Behavior::Candidate, true, false),
    );
    let a = selected(&d.0, "a", 2);
    let b = selected(&d.0, "b", 2);
    let p = RHO * FORCE * H;
    assert_eq!(
        native_samples(&a, FieldRole::IntervalPressure).unwrap(),
        vec![0.0, p]
    );
    assert_eq!(
        native_samples(&b, FieldRole::IntervalPressure).unwrap(),
        vec![0.0, 2.0 * p]
    );
    let result = compare(&a, &b, recipe(FieldRole::IntervalPressure))
        .unwrap()
        .outcome
        .unwrap();
    assert_eq!(result.units, "Pa");
    assert_eq!(result.metrics.support_count, 2);
    close(result.metrics.l2, p);
    close(result.metrics.rmse, p / 2.0f64.sqrt());
    close(result.metrics.linf, p);
    assert_eq!(result.metrics.relative_l2, Some(1.0));
    assert_eq!(
        facts::rhs_relation(&a, &b).unwrap(),
        RhsRelation::DifferentHashes
    );
    assert_eq!(
        a.publication().time.to_bits(),
        b.publication().time.to_bits()
    );
    let values = compared_values(&a, &b, recipe(FieldRole::IntervalPressure)).unwrap();
    assert_eq!(values.reference, vec![0.0, p]);
    assert_eq!(values.candidate, vec![0.0, 2.0 * p]);
    assert_eq!(values.comparison.outcome.unwrap().metrics, result.metrics);
}

#[test]
fn adaptive_interval_coverage_retains_nonempty_matches_and_unmatched_publications() {
    let d = Directory::new();
    write(
        &d.0,
        "regular",
        configured(20, FORCE, Behavior::Candidate, true, false),
    );
    write(
        &d.0,
        "adaptive",
        configured(21, FORCE, Behavior::Candidate, true, true),
    );
    let a = all(&d.0, "regular");
    let b = all(&d.0, "adaptive");
    assert!(b[2].publication().dt.unwrap() < 2.0);
    assert_ne!(b[2].publication().dt.unwrap().to_bits(), DT.to_bits());
    let coverage = align(&a[1..], &b[1..], recipe(FieldRole::IntervalPressure)).unwrap();
    assert!(!coverage.pairs.is_empty());
    assert!(!coverage.reference_unmatched.is_empty());
    assert!(!coverage.candidate_unmatched.is_empty());
    assert!(coverage.pairs.iter().all(|p| p.outcome.is_ok()));
    assert_eq!(
        compare(&a[2], &b[2], recipe(FieldRole::IntervalPressure))
            .unwrap()
            .outcome,
        Err(Unavailable::PhysicalTime)
    );
    let observed = facts::inspect(&b[2]).unwrap().interval.unwrap();
    let row = RecordedConfiguredSession::decode(b[2].run(), b[2].publication()).unwrap();
    assert_eq!(
        observed.start.to_bits(),
        row.interval_pressure.unwrap().0.to_bits()
    );
    assert_eq!(observed.end.to_bits(), b[2].publication().time.to_bits());
}

#[test]
fn actual_selection_fallback_and_authored_runtime_outcomes_stay_separate() {
    let d = Directory::new();
    for (index, behavior) in [
        Behavior::Candidate,
        Behavior::QualityFallback,
        Behavior::RuntimeFailure,
        Behavior::InvalidOutput,
        Behavior::SupportDecline,
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("outcome-{index}");
        write(
            &d.0,
            &name,
            configured(30 + index as u64, FORCE, behavior, true, false),
        );
        let selection = selected(&d.0, &name, 2);
        let actual = facts::inspect(&selection).unwrap();
        let receipt = actual.receipt.unwrap();
        assert_eq!(actual.identity, *selection.identity());
        assert_eq!(receipt.binding.unwrap(), identity());
        match behavior {
            Behavior::Candidate => {
                assert_eq!(receipt.selected, RecordedPressureSelection::Configured);
                assert_eq!(
                    actual.runtime.unwrap().outcome,
                    RuntimeOutcome::CandidateReturned
                );
            }
            Behavior::QualityFallback => {
                assert!(matches!(
                    receipt.selected,
                    RecordedPressureSelection::Native(_)
                ));
                assert_eq!(
                    receipt.rejection,
                    Some(RecordedProviderRejection::QualityGate)
                );
                assert_eq!(
                    actual.runtime.unwrap().outcome,
                    RuntimeOutcome::CandidateReturned
                );
            }
            Behavior::RuntimeFailure | Behavior::InvalidOutput => {
                assert!(matches!(
                    receipt.selected,
                    RecordedPressureSelection::Native(_)
                ));
                assert!(matches!(
                    receipt.rejection,
                    Some(RecordedProviderRejection::Execution(_))
                ));
                assert!(
                    matches!(actual.runtime.unwrap().outcome,RuntimeOutcome::NoCandidate{reason,..} if reason==if matches!(behavior,Behavior::InvalidOutput){NoCandidateReason::InvalidOutput}else{NoCandidateReason::RuntimeFailure})
                );
            }
            Behavior::SupportDecline => {
                assert!(!receipt.prediction_called);
                assert!(receipt.attempted.is_some());
                assert!(actual.runtime.is_none());
            }
        }
    }
}

#[test]
fn aggregate_counts_pressure_call_durations_and_order_have_bounded_scope() {
    let d = Directory::new();
    write(
        &d.0,
        "a",
        configured(50, FORCE, Behavior::Candidate, true, false),
    );
    write(
        &d.0,
        "b",
        configured(51, FORCE, Behavior::QualityFallback, true, false),
    );
    let selections = all(&d.0, "a");
    let aggregate = facts::summarize(&selections).unwrap();
    assert_eq!(
        aggregate.selections,
        selections
            .iter()
            .map(|s| s.identity().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        (
            aggregate.constructor_count,
            aggregate.accepted_count,
            aggregate.attempted_count,
            aggregate.prediction_called_count
        ),
        (1, 3, 2, 2)
    );
    assert_eq!(
        (
            aggregate.configured_selected_count,
            aggregate.native_selected_count,
            aggregate.rejected_count,
            aggregate.runtime_observed_count
        ),
        (2, 1, 0, 2)
    );
    let mut expected = [Duration::ZERO; 4];
    for selection in &selections {
        if let Some(receipt) =
            RecordedConfiguredSession::decode(selection.run(), selection.publication())
                .unwrap()
                .receipt
        {
            for (sum, value) in expected.iter_mut().zip([
                receipt.total,
                receipt.inference,
                receipt.qualification,
                receipt.fallback,
            ]) {
                *sum = sum.checked_add(value).unwrap();
            }
        }
    }
    // These are exact sums of retained pressure-stage call durations, not step
    // wall time, isolated model compute, throughput or runtime shutdown timing.
    assert_eq!(
        [
            aggregate.durations.total,
            aggregate.durations.inference,
            aggregate.durations.qualification,
            aggregate.durations.fallback
        ],
        expected
    );
    let fallback = facts::summarize(&all(&d.0, "b")).unwrap();
    assert_eq!(
        (
            fallback.native_selected_count,
            fallback.configured_selected_count,
            fallback.rejected_count,
            fallback.runtime_observed_count
        ),
        (3, 0, 2, 2)
    );
    let gaps = facts::summarize(&[selected(&d.0, "a", 0), selected(&d.0, "a", 3)]).unwrap();
    assert_eq!((gaps.constructor_count, gaps.accepted_count), (1, 1));
    assert!(matches!(
        facts::summarize(&[selected(&d.0, "a", 2), selected(&d.0, "a", 1)]),
        Err(RecordingError::Invalid(
            "configured comparison selection order/run"
        ))
    ));
    assert!(matches!(
        facts::summarize(&[selected(&d.0, "a", 1), selected(&d.0, "b", 2)]),
        Err(RecordingError::Invalid(
            "configured comparison selection order/run"
        ))
    ));
    let empty = facts::summarize(&[]).unwrap();
    assert!(empty.selections.is_empty());
    assert_eq!((empty.constructor_count, empty.accepted_count), (0, 0));
    assert_eq!(empty.durations.total, Duration::ZERO);
    assert!(matches!(
        facts::summarize(&[selected(&d.0, "a", 1), selected(&d.0, "a", 1)]),
        Err(RecordingError::Invalid(
            "configured comparison selection order/run"
        ))
    ));
    // Caller provenance is deliberately outside the core row source binding,
    // but inside Selection::run_digest. Equal declaration IDs are insufficient.
    let mut declared = configured(50, FORCE, Behavior::Candidate, true, false);
    declared
        .run
        .provenance
        .push_str("; different caller declaration");
    write(&d.0, "same-id-other-provenance", declared);
    let original = selected(&d.0, "a", 1);
    let changed = selected(&d.0, "same-id-other-provenance", 2);
    facts::inspect(&changed).unwrap();
    assert_eq!(original.run().identity, changed.run().identity);
    assert_ne!(original.run_digest(), changed.run_digest());
    assert!(matches!(
        facts::summarize(&[original, changed]),
        Err(RecordingError::Invalid(
            "configured comparison selection order/run"
        ))
    ));
    write(
        &d.0,
        "at-bound",
        configured_intervals(
            52,
            FORCE,
            Behavior::Candidate,
            true,
            false,
            grid(),
            facts::MAX_SELECTIONS - 1,
        ),
    );
    let at_bound = all(&d.0, "at-bound");
    assert_eq!(at_bound.len(), facts::MAX_SELECTIONS);
    let boundary = facts::summarize(&at_bound).unwrap();
    assert_eq!(boundary.constructor_count, 1);
    assert_eq!(boundary.accepted_count, facts::MAX_SELECTIONS - 1);
    assert_eq!(boundary.runtime_observed_count, facts::MAX_SELECTIONS - 2);
    let oversized: Vec<_> = (0..=facts::MAX_SELECTIONS)
        .map(|_| selected(&d.0, "a", 1))
        .collect();
    assert!(matches!(
        facts::summarize(&oversized),
        Err(RecordingError::Limit(
            "configured comparison selection count"
        ))
    ));
}

fn replace_control_bytes(row: &mut RecordedStep, needle: &[u8], replacement: &[u8], last: bool) {
    assert_eq!(needle.len(), replacement.len());
    let (prefix, encoded) = row.controls.split_once(':').unwrap();
    let mut bytes: Vec<_> = encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect();
    let positions: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == needle).then_some(i))
        .collect();
    assert!(
        !positions.is_empty(),
        "intended metadata mutation did not target a retained byte sequence"
    );
    let position = if last {
        *positions.last().unwrap()
    } else {
        positions[0]
    };
    bytes[position..position + needle.len()].copy_from_slice(replacement);
    row.controls = format!("{prefix}:{}", hex(&bytes));
}
#[test]
fn strict_configured_schema_metadata_and_runtime_association_precede_field_sampling() {
    let d = Directory::new();
    write(
        &d.0,
        "good",
        configured(60, FORCE, Behavior::Candidate, true, false),
    );
    let good = selected(&d.0, "good", 2);
    let hash = facts::inspect(&good)
        .unwrap()
        .receipt
        .unwrap()
        .context
        .unwrap()
        .rhs_sha256;
    for case in 0..12 {
        let mut capture = configured(61 + case, FORCE, Behavior::Candidate, true, false);
        let row = &mut capture.rows[2];
        match case {
            0 => field_mut(row, "interval_pressure").units = "kPa".into(),
            1 => field_mut(row, "interval_pressure").stage = "tentative".into(),
            2 => field_mut(row, "velocity_x").association = "cell_center".into(),
            3 => field_mut(row, "interval_pressure").evaluation_time = None,
            4 => {
                field_mut(row, "interval_pressure").data[..8].copy_from_slice(&1.0f64.to_le_bytes())
            }
            5 => row.controls.push_str("00"),
            6 => row.controls = row.controls.replacen("RCP3:", "RCP9:", 1),
            7 => {
                let mut changed = hash;
                changed[0] ^= 1;
                replace_control_bytes(row, &hash, &changed, true);
            }
            8 => replace_control_bytes(row, b"m3/s2", b"wrong", false),
            9 => capture.run.configuration = "RSC1:00".into(),
            10 => capture.run.solver = "rheon-configured-session-v9/jacobi-pcg-v1".into(),
            _ => row.fields.retain(|field| field.name != "fraction"),
        }
        let name = format!("invalid-{case}");
        write(&d.0, &name, capture);
        let invalid = selected(&d.0, &name, 2);
        let expected = if case == 10 {
            Unavailable::UnsupportedProfile
        } else {
            Unavailable::FieldSchema
        };
        assert_eq!(
            compare(&good, &invalid, recipe(FieldRole::IntervalPressure))
                .unwrap()
                .outcome,
            Err(expected),
            "case {case}"
        );
        assert_eq!(
            native_samples(&invalid, FieldRole::Tracer),
            Err(expected),
            "case {case}"
        );
        assert!(facts::inspect(&invalid).is_err(), "case {case}");
        assert!(facts::rhs_relation(&good, &invalid).is_err(), "case {case}");
        if case == 4 {
            let diagnostic = Recipe {
                pressure: PressurePolicy::RemoveMeanOffsetDiagnostic,
                ..recipe(FieldRole::IntervalPressure)
            };
            assert_eq!(
                compare(&good, &invalid, diagnostic).unwrap().outcome,
                Err(Unavailable::FieldSchema)
            );
        }
    }
}

#[test]
fn stage_geometry_interval_start_and_legacy_constructor_facts_are_explicit() {
    let d = Directory::new();
    write(
        &d.0,
        "good",
        configured(80, FORCE, Behavior::Candidate, false, false),
    );
    let constructor = selected(&d.0, "good", 0);
    let accepted = selected(&d.0, "good", 1);
    assert_eq!(
        compare(&constructor, &accepted, recipe(FieldRole::Tracer))
            .unwrap()
            .outcome,
        Err(Unavailable::PublicationStage)
    );
    let initial = facts::inspect(&constructor).unwrap();
    assert!(initial.receipt.is_none());
    assert!(initial.interval.is_none());
    assert!(initial.runtime.is_none());
    assert_eq!(
        facts::rhs_relation(&constructor, &accepted).unwrap(),
        RhsRelation::Unavailable
    );
    let mut missing = configured(81, FORCE, Behavior::Candidate, false, false);
    field_mut(&mut missing.rows[2], "interval_pressure").evaluation_time = None;
    write(&d.0, "missing-start", missing);
    assert_eq!(
        native_samples(
            &selected(&d.0, "missing-start", 2),
            FieldRole::IntervalPressure
        ),
        Err(Unavailable::FieldSchema)
    );
    let changed = configured_on_grid(
        82,
        FORCE,
        Behavior::Candidate,
        false,
        false,
        GridGeometry::new([2, 1, 1], [H; 3], [1.0, 0.0, 0.0]).unwrap(),
    );
    write(&d.0, "geometry", changed);
    facts::inspect(&selected(&d.0, "geometry", 1)).unwrap();
    assert_eq!(
        compare(
            &accepted,
            &selected(&d.0, "geometry", 1),
            recipe(FieldRole::Tracer)
        )
        .unwrap()
        .outcome,
        Err(Unavailable::Geometry)
    );
    write(&d.0, "native", native(83));
    assert!(facts::inspect(&selected(&d.0, "native", 1)).is_err());
}

fn authored_faces(row: &mut RecordedStep, scale: f32) {
    // x: derivative 1; y: derivative 2; z: derivative -1. With h=.25,
    // every cell has divergence 2*scale. These are authored arithmetic inputs.
    for (name, values) in [
        ("velocity_x", vec![0.0, 0.25, 0.5]),
        ("velocity_y", vec![0.0, 0.0, 0.5, 0.5]),
        ("velocity_z", vec![0.0, 0.0, -0.25, -0.25]),
    ] {
        field_mut(row, name).data = values
            .into_iter()
            .flat_map(|v: f32| (v * scale).to_le_bytes())
            .collect();
    }
}
#[test]
fn authored_native_face_oracle_checks_shared_divergence_and_face_metrics() {
    let d = Directory::new();
    let mut a = configured(90, FORCE, Behavior::Candidate, false, false);
    authored_faces(&mut a.rows[2], 1.0);
    write(&d.0, "a", a);
    let mut b = configured(91, FORCE, Behavior::Candidate, false, false);
    authored_faces(&mut b.rows[2], 2.0);
    write(&d.0, "b", b);
    let a = selected(&d.0, "a", 2);
    let b = selected(&d.0, "b", 2);
    assert_eq!(
        native_samples(&a, FieldRole::VelocityDivergence).unwrap(),
        vec![2.0; 2]
    );
    assert_eq!(
        native_samples(&b, FieldRole::VelocityDivergence).unwrap(),
        vec![4.0; 2]
    );
    let result = compare(&a, &b, recipe(FieldRole::VelocityDivergence))
        .unwrap()
        .outcome
        .unwrap();
    assert_eq!(result.units, "1/s");
    assert_eq!(result.metrics.support_count, 2);
    close(result.metrics.l2, 8.0f64.sqrt());
    close(result.metrics.rmse, 2.0);
    close(result.metrics.linf, 2.0);
    assert_eq!(result.metrics.relative_l2, Some(1.0));
    let x = compare(&a, &b, recipe(FieldRole::VelocityX))
        .unwrap()
        .outcome
        .unwrap();
    assert_eq!(x.units, "m/s");
    assert_eq!(x.metrics.support_count, 3);
    close(x.metrics.l2, (5.0 / 16.0f64).sqrt());
    close(x.metrics.rmse, (5.0 / 48.0f64).sqrt());
    close(x.metrics.linf, 0.5);
    let values = compared_values(&a, &b, recipe(FieldRole::VelocityX)).unwrap();
    assert_eq!(values.reference, vec![0.0, 0.25, 0.5]);
    assert_eq!(values.sampling.association, SampleAssociation::XFace);
    assert_eq!(values.sampling.dimensions, [3, 1, 1]);
    assert_eq!(values.comparison.outcome.unwrap().metrics, x.metrics);
}

#[test]
fn damaged_sqlite_field_hash_is_refused_before_owned_selection_or_facts() {
    let d = Directory::new();
    write(
        &d.0,
        "archive",
        configured(100, FORCE, Behavior::Candidate, true, false),
    );
    let original = selected(&d.0, "archive", 2).digest();
    let db = rusqlite::Connection::open(d.0.join("archive")).unwrap();
    db.execute("UPDATE fields SET data=zeroblob(length(data)) WHERE ordinal=2 AND name='interval_pressure'",[]).unwrap();
    drop(db);
    let reader = reader(&d.0, "archive");
    assert!(matches!(
        Selection::read(&reader, 2),
        Err(RecordingError::Integrity(_))
    ));
    assert_ne!(original, [0; 32]);
}

const COLD_CASES: [&str; 9] = [
    "native",
    "rest",
    "candidate",
    "fallback",
    "failure",
    "invalid",
    "rcp2",
    "double",
    "adaptive",
];
#[test]
fn producer_exits_before_model_absent_configured_comparison_reader() {
    match std::env::var(PHASE).ok().as_deref() {
        Some("produce") => {
            let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
            let mut ledger = String::new();
            for (index, name) in COLD_CASES.into_iter().enumerate() {
                let id = 300 + index as u64;
                let capture = if name == "native" {
                    native(id)
                } else {
                    configured(
                        id,
                        if name == "double" {
                            2.0 * FORCE
                        } else if name == "rest" {
                            0.0
                        } else {
                            FORCE
                        },
                        match name {
                            "fallback" => Behavior::QualityFallback,
                            "failure" => Behavior::RuntimeFailure,
                            "invalid" => Behavior::InvalidOutput,
                            _ => Behavior::Candidate,
                        },
                        name != "rcp2",
                        name == "adaptive",
                    )
                };
                // These expected byte strings come from the producer's capture,
                // before SQLite append or any cold selection/reduction.
                let pressure = hex(&capture.rows[2].field("interval_pressure").unwrap().data);
                let velocity = hex(&capture.rows[2].field("velocity_x").unwrap().data);
                write(&root, name, capture);
                let file_sha: [u8; 32] =
                    Sha256::digest(std::fs::read(root.join(name)).unwrap()).into();
                let digest = selected(&root, name, 2).digest();
                ledger.push_str(&format!(
                    "{name} {} {pressure} {velocity} {}\n",
                    hex(&file_sha),
                    hex(&digest)
                ));
            }
            std::fs::write(root.join("source-captures.txt"), ledger).unwrap();
            assert!(!root.join("absent-model.pt").exists());
            println!("producer closed nine native/configured comparison archives");
        }
        Some("inspect") => {
            assert_eq!(std::env::var_os("PATH").unwrap(), "");
            let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
            assert!(!root.join("absent-model.pt").exists());
            let ledger = std::fs::read_to_string(root.join("source-captures.txt")).unwrap();
            assert_eq!(ledger.lines().count(), COLD_CASES.len());
            for (index, (line, name)) in ledger.lines().zip(COLD_CASES).enumerate() {
                let parts: Vec<_> = line.split_whitespace().collect();
                assert_eq!(parts.len(), 5);
                assert_eq!(parts[0], name);
                let sha: [u8; 32] = Sha256::digest(std::fs::read(root.join(name)).unwrap()).into();
                assert_eq!(hex(&sha), parts[1]);
                let r = reader(&root, name);
                assert_eq!(r.count().unwrap(), 4);
                assert_eq!(r.run().identity.run_id, 300 + index as u64);
                assert_eq!(r.run().identity.source_revision, SOURCE);
                assert_eq!(r.status().unwrap().outcome, "completed");
                assert_eq!(r.status().unwrap().accepted_generation, Some(3));
                let selection = Selection::read(&r, 2).unwrap();
                assert_eq!(
                    hex(&selection
                        .publication()
                        .field("interval_pressure")
                        .unwrap()
                        .data),
                    parts[2]
                );
                assert_eq!(
                    hex(&selection.publication().field("velocity_x").unwrap().data),
                    parts[3]
                );
                assert_eq!(hex(&selection.digest()), parts[4]);
                if name == "native" {
                    assert!(facts::inspect(&selection).is_err());
                    continue;
                }
                let f = facts::inspect(&selection).unwrap();
                let receipt = f.receipt.unwrap();
                assert_eq!(receipt.binding.unwrap(), identity());
                assert_eq!(f.runtime.is_some(), name != "rcp2");
                match name {
                    "rest" | "candidate" | "double" | "rcp2" | "adaptive" => {
                        assert_eq!(receipt.selected, RecordedPressureSelection::Configured)
                    }
                    _ => assert!(matches!(
                        receipt.selected,
                        RecordedPressureSelection::Native(_)
                    )),
                }
                if name == "fallback" {
                    assert_eq!(
                        receipt.rejection,
                        Some(RecordedProviderRejection::QualityGate)
                    );
                    assert_eq!(
                        f.runtime.as_ref().unwrap().outcome,
                        RuntimeOutcome::CandidateReturned
                    );
                }
                if name == "failure" {
                    assert!(matches!(
                        f.runtime.as_ref().unwrap().outcome,
                        RuntimeOutcome::NoCandidate {
                            reason: NoCandidateReason::RuntimeFailure,
                            dispatched: false,
                            ..
                        }
                    ));
                }
                if name == "invalid" {
                    assert!(matches!(
                        f.runtime.as_ref().unwrap().outcome,
                        RuntimeOutcome::NoCandidate {
                            reason: NoCandidateReason::InvalidOutput,
                            dispatched: true,
                            invalid_payload_sha256: Some(hash),
                            ..
                        } if hash == [19; 32]
                    ));
                }
                let aggregate = facts::summarize(&all(&root, name)).unwrap();
                assert_eq!(
                    (aggregate.constructor_count, aggregate.accepted_count),
                    (1, 3)
                );
                assert_eq!(
                    aggregate.runtime_observed_count,
                    if name == "rcp2" { 0 } else { 2 }
                );
            }
            let native = all(&root, "native");
            let rest = all(&root, "rest");
            let candidate = all(&root, "candidate");
            let doubled = all(&root, "double");
            let adaptive = all(&root, "adaptive");
            let mixed = align(
                &native[1..],
                &rest[1..],
                recipe(FieldRole::IntervalPressure),
            )
            .unwrap();
            assert_eq!(mixed.pairs.len(), 3);
            assert!(
                mixed
                    .pairs
                    .iter()
                    .all(|p| p.outcome.as_ref().unwrap().metrics.l2 == 0.0)
            );
            assert_eq!(
                facts::rhs_relation(&native[2], &rest[2]).unwrap(),
                RhsRelation::Unavailable
            );
            assert_eq!(
                facts::rhs_relation(&candidate[2], &selected(&root, "fallback", 2)).unwrap(),
                RhsRelation::EqualHashes
            );
            assert_eq!(
                facts::rhs_relation(&candidate[2], &doubled[2]).unwrap(),
                RhsRelation::DifferentHashes
            );
            let metric = compare(
                &candidate[2],
                &doubled[2],
                recipe(FieldRole::IntervalPressure),
            )
            .unwrap()
            .outcome
            .unwrap();
            let p = RHO * FORCE * H;
            close(metric.metrics.l2, p);
            close(metric.metrics.rmse, p / 2.0f64.sqrt());
            close(metric.metrics.linf, p);
            assert_eq!(metric.metrics.relative_l2, Some(1.0));
            let coverage = align(
                &candidate[1..],
                &adaptive[1..],
                recipe(FieldRole::IntervalPressure),
            )
            .unwrap();
            assert!(!coverage.pairs.is_empty());
            assert!(!coverage.reference_unmatched.is_empty());
            assert!(!coverage.candidate_unmatched.is_empty());
            println!("PATH-empty cold reader verified nine comparison archives without model");
        }
        None => {
            let d = Directory::new();
            for (phase, marker) in [
                (
                    "produce",
                    "producer closed nine native/configured comparison archives",
                ),
                (
                    "inspect",
                    "PATH-empty cold reader verified nine comparison archives without model",
                ),
            ] {
                // output() reaps the real producer OS before reader creation.
                let child = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", PROCESS_TEST, "--nocapture"])
                    .env(PHASE, phase)
                    .env(ROOT, &d.0)
                    .env("PATH", "")
                    .output()
                    .unwrap();
                let stdout = String::from_utf8_lossy(&child.stdout);
                assert!(
                    child.status.success(),
                    "{phase}: {stdout}\n{}",
                    String::from_utf8_lossy(&child.stderr)
                );
                assert!(stdout.contains(marker));
                assert!(
                    stdout.contains("1 passed; 0 failed;"),
                    "zero-test child: {stdout}"
                );
            }
        }
        Some(other) => panic!("unknown comparison test phase {other}"),
    }
}
