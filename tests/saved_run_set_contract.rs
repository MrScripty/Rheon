#![cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/bin/desktop/run_sets.rs"]
#[allow(dead_code)] // Configured owner contracts are exercised by configured_run_sets_contract.
mod run_sets;

use rheon::{SessionIdentity, recording::*};
use run_sets::*;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn directory() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "rheon-saved-sets-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).unwrap();
    path
}
fn spec(method: NativePressureMethod) -> NativeCarrierSpec {
    NativeCarrierSpec {
        counts_xyz: [3, 4, 5],
        spacing_m: [0.2, 0.25, 0.3],
        origin_m: [-2.0, 3.0, 5.0],
        density_kg_m3: 1.0,
        pressure_method: method,
        tolerances: NativeTolerances {
            relative_residual: 1e-10,
            absolute_residual_m3_s2: 1e-12,
            predicted_divergence_per_s: 1e-8,
            actual_divergence_per_s: 1e-5,
            max_courant: 1.0,
            max_pressure_iterations: 2000,
        },
        requested_dt_s: 0.003,
        requested_steps: 3,
        source: Some(SourceSpec {
            lower_m: [-1.9, 3.1, 5.1],
            upper_m: [-1.6, 3.7, 6.1],
            tracer_rate_per_s: 2.0,
            vertical_acceleration_m_s2: 1.0,
            active_steps: 3,
        }),
        simulation_memory_limit_bytes: 64 * 1024 * 1024,
        recording: ArchiveBudget {
            max_database_bytes: 8 * 1024 * 1024,
            max_record_bytes: 1024 * 1024,
            max_records: 16,
        },
    }
}
fn attempt(method: NativePressureMethod) -> RunAttempt {
    let method = method.implementation().id();
    RunAttempt {
        attempt_id: format!("attempt-{method}"),
        archive_relative_path: format!("{method}.sqlite"),
        expected_run_id: 77,
        expected_source_revision: "actual-native-test-source".into(),
        finalized: None,
    }
}
fn member(method: NativePressureMethod) -> RunSetMember {
    RunSetMember {
        member_id: method.implementation().id().into(),
        spec: FrozenSpec::new(spec(method)).unwrap(),
        differences_from_baseline: if method == NativePressureMethod::JacobiPcgV1 {
            vec![]
        } else {
            vec![SpecDifference::PressureMethod]
        },
        attempt: Some(attempt(method)),
    }
}
fn set() -> RunSetRevision {
    RunSetRevision {
        schema_version: 1,
        set_id: "two-real-native-variants".into(),
        parent_sha256: None,
        baseline_member: "jacobi-pcg-v1".into(),
        members: vec![
            member(NativePressureMethod::JacobiPcgV1),
            member(NativePressureMethod::SymmetricGaussSeidelPcgV1),
        ],
    }
}
/// This test host drives existing numerical and recording owners directly. It
/// is not a second application worker, provider runtime or GUI acceptance test.
fn produce(root: &Path, member: &RunSetMember, finish: bool) -> Vec<RecordedStep> {
    produce_with_metadata(root, member, finish, false)
}
fn produce_with_metadata(
    root: &Path,
    member: &RunSetMember,
    finish: bool,
    historical_metadata: bool,
) -> Vec<RecordedStep> {
    let a = member.attempt.as_ref().unwrap();
    let s = &member.spec.spec;
    let mut simulation = s.create_simulation().unwrap();
    let mut run = bound_native_run(
        &member.spec,
        &simulation,
        SessionIdentity {
            run_id: a.expected_run_id,
            source_revision: a.expected_source_revision.clone(),
        },
        "native contract host; no model loaded",
    )
    .unwrap();
    if historical_metadata {
        run.solver = format!(
            "rheon-carrier-full-box-v1/archived-build/{}",
            s.pressure_method.implementation().id()
        );
        run.configuration = "opaque historical producer Debug rendering, never executed".into();
    }
    let limits = s.recording.limits();
    let mut writer =
        StepRecorder::create(&root.join(&a.archive_relative_path), run, limits).unwrap();
    let mut publications =
        vec![carrier_publication(&simulation, 0, limits.max_record_bytes).unwrap()];
    writer.append(&publications[0]).unwrap();
    for interval in 0..s.requested_steps {
        simulation
            .step(s.requested_dt_s, s.source_at(interval), |_| false)
            .unwrap();
        let publication =
            carrier_publication(&simulation, interval + 1, limits.max_record_bytes).unwrap();
        writer.append(&publication).unwrap();
        publications.push(publication);
    }
    if finish {
        writer
            .finish(
                "completed",
                "all requested native intervals",
                Some(simulation.state().generation),
            )
            .unwrap();
    }
    publications
}
fn ready(root: &Path, member: &RunSetMember) -> RecordingReader {
    match open_member(root, member) {
        MemberState::Ready(reader) => *reader,
        _ => panic!("not a pinned ready member"),
    }
}

#[test]
fn two_real_native_variants_close_and_cold_reopen_exact_bits() {
    let root = directory();
    let mut revision = set();
    for member in &mut revision.members {
        let expected = produce(&root, member, true);
        member
            .attempt
            .as_mut()
            .unwrap()
            .pin_closed(&root, &member.spec)
            .unwrap();
        let reader = ready(&root, member);
        assert_eq!(reader.count().unwrap(), 4);
        assert_eq!(reader.status().unwrap().outcome, "completed");
        for publication in expected {
            let actual = reader.select(publication.ordinal).unwrap();
            assert_eq!(actual.generation, publication.generation);
            assert_eq!(actual.time.to_bits(), publication.time.to_bits());
            assert_eq!(
                actual.dt.map(f64::to_bits),
                publication.dt.map(f64::to_bits)
            );
            assert_eq!(actual.fields, publication.fields);
        }
    }
    revision.members[0]
        .spec
        .save_new(&root.join("spec.json"))
        .unwrap();
    assert_eq!(
        FrozenSpec::open(&root.join("spec.json")).unwrap(),
        revision.members[0].spec
    );
    revision.save_new(&root.join("set.json")).unwrap();
    assert_eq!(
        RunSetRevision::open(&root.join("set.json")).unwrap(),
        revision
    );
    // Run a distinct reader process after both native producers and SQLite
    // writers have been dropped; this process never constructs Simulation.
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "cold_reader_child", "--nocapture"])
        .env("RHEON_SAVED_SET_CHILD", &root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("fresh-process exact native bytes verified")
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn cold_reader_child() {
    let Some(root) = std::env::var_os("RHEON_SAVED_SET_CHILD") else {
        return;
    };
    let root = PathBuf::from(root);
    let revision = RunSetRevision::open(&root.join("set.json")).unwrap();
    for member in &revision.members {
        let reader = ready(&root, member);
        let publication = reader.select(1).unwrap();
        let native = RecordedCarrier::decode(reader.run(), &publication).unwrap();
        assert_eq!(publication.time.to_bits(), 0.003f64.to_bits());
        assert_eq!(publication.fields.len(), 5);
        assert_eq!(native.state().generation, 1);
        // Independently read the existing SQLite BLOB using its public wire
        // contract; exact native bytes must agree with the selected decoder.
        let path = root.join(&member.attempt.as_ref().unwrap().archive_relative_path);
        let connection = rusqlite::Connection::open(path).unwrap();
        let raw: Vec<u8> = connection
            .query_row(
                "SELECT data FROM fields WHERE ordinal=1 AND name='velocity_y'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(publication.field("velocity_y").unwrap().data, raw);
        let native_bits: Vec<u8> = native
            .state()
            .y
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        assert_eq!(native_bits, raw);
    }
    println!("fresh-process exact native bytes verified");
}
#[test]
fn closed_producer_child() {
    let Some(root) = std::env::var_os("RHEON_SAVED_SET_PRODUCER") else {
        return;
    };
    let root = PathBuf::from(root);
    let mut revision = set();
    for member in &mut revision.members {
        produce(&root, member, true);
        member
            .attempt
            .as_mut()
            .unwrap()
            .pin_closed(&root, &member.spec)
            .unwrap();
    }
    revision.save_new(&root.join("set.json")).unwrap();
    println!("two native archives finalized by producer process");
}
#[test]
fn producer_process_exits_before_fresh_reader_starts() {
    let root = directory();
    let producer = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "closed_producer_child", "--nocapture"])
        .env("RHEON_SAVED_SET_PRODUCER", &root)
        .output()
        .unwrap();
    assert!(
        producer.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&producer.stdout),
        String::from_utf8_lossy(&producer.stderr)
    );
    assert!(
        String::from_utf8_lossy(&producer.stdout)
            .contains("two native archives finalized by producer process")
    );
    let reader = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "cold_reader_child", "--nocapture"])
        .env("RHEON_SAVED_SET_CHILD", &root)
        .output()
        .unwrap();
    assert!(
        reader.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&reader.stdout),
        String::from_utf8_lossy(&reader.stderr)
    );
    assert!(
        String::from_utf8_lossy(&reader.stdout)
            .contains("fresh-process exact native bytes verified")
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn full_intent_binding_rejects_changed_source_time_and_controls_without_parsing_debug() {
    let root = directory();
    let mut m = member(NativePressureMethod::JacobiPcgV1);
    produce_with_metadata(&root, &m, true, true);
    m.attempt
        .as_mut()
        .unwrap()
        .pin_closed(&root, &m.spec)
        .unwrap();
    assert!(ready(&root, &m).run().configuration.contains("historical"));
    for change in 0..3 {
        let mut altered = m.clone();
        match change {
            0 => altered.spec.spec.source.as_mut().unwrap().tracer_rate_per_s = 4.0,
            1 => altered.spec.spec.requested_dt_s = 0.002,
            _ => altered.spec.spec.tolerances.relative_residual = 1e-8,
        }
        altered.spec = FrozenSpec::new(altered.spec.spec).unwrap();
        assert!(matches!(
            open_member(&root, &altered),
            MemberState::Invalid(RunSetError::Integrity("archive/spec binding"))
        ))
    }
    let mut simulation = m.spec.spec.create_simulation().unwrap();
    simulation
        .step(m.spec.spec.requested_dt_s, None, |_| false)
        .unwrap();
    assert!(matches!(
        bound_native_run(
            &m.spec,
            &simulation,
            SessionIdentity {
                run_id: 77,
                source_revision: "test".into()
            },
            "test"
        ),
        Err(RunSetError::Invalid("native constructor/spec binding"))
    ));
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn immutable_revisions_refuse_overwrite_and_keep_old_members() {
    let root = directory();
    let first = set();
    let first_hash = first.sha256().unwrap();
    first.save_new(&root.join("first.json")).unwrap();
    let old_bytes = std::fs::read(root.join("first.json")).unwrap();
    let mut second = first.clone();
    second.parent_sha256 = Some(first_hash);
    second.members[1].spec.spec.requested_dt_s = 0.001;
    second.members[1].spec = FrozenSpec::new(second.members[1].spec.spec.clone()).unwrap();
    second.members[1]
        .differences_from_baseline
        .push(SpecDifference::TimePolicy);
    assert!(second.save_new(&root.join("first.json")).is_err());
    assert_eq!(std::fs::read(root.join("first.json")).unwrap(), old_bytes);
    second.save_new(&root.join("second.json")).unwrap();
    assert_eq!(
        RunSetRevision::open(&root.join("first.json")).unwrap(),
        first
    );
    assert_ne!(second.sha256().unwrap(), first_hash);
    assert!(matches!(
        open_member(&root, &second.members[1]),
        MemberState::Missing
    ));
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn not_run_missing_incomplete_and_unpinned_are_distinct() {
    let root = directory();
    let mut member = member(NativePressureMethod::JacobiPcgV1);
    let reference = member.attempt.take();
    assert!(matches!(open_member(&root, &member), MemberState::NotRun));
    member.attempt = reference;
    assert!(matches!(open_member(&root, &member), MemberState::Missing));
    produce(&root, &member, false);
    match open_member(&root, &member) {
        MemberState::Incomplete {
            recorded_count,
            status,
        } => {
            assert_eq!(recorded_count, 4);
            assert_eq!(status.outcome, "active");
        }
        _ => panic!("active archive was not incomplete"),
    }
    assert!(matches!(
        member
            .attempt
            .as_mut()
            .unwrap()
            .pin_closed(&root, &member.spec),
        Err(RunSetError::Invalid("archive is not terminal"))
    ));
    let other_root = directory();
    produce(&other_root, &member, true);
    match open_member(&other_root, &member) {
        MemberState::Unpinned {
            recorded_count,
            status,
        } => {
            assert_eq!(recorded_count, 4);
            assert_eq!(status.outcome, "completed");
        }
        _ => panic!("unbound terminal archive silently promoted"),
    }
    std::fs::remove_dir_all(root).unwrap();
    std::fs::remove_dir_all(other_root).unwrap();
}
#[test]
fn corrupt_changed_and_unsupported_members_never_substitute_live_data() {
    let root = directory();
    let mut m = member(NativePressureMethod::JacobiPcgV1);
    produce(&root, &m, true);
    m.attempt
        .as_mut()
        .unwrap()
        .pin_closed(&root, &m.spec)
        .unwrap();
    let path = root.join(&m.attempt.as_ref().unwrap().archive_relative_path);
    let original = std::fs::read(&path).unwrap();
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute(
        "UPDATE fields SET data=zeroblob(length(data)) WHERE ordinal=1 AND name='velocity_y'",
        [],
    )
    .unwrap();
    drop(c);
    assert!(matches!(
        open_member(&root, &m),
        MemberState::Invalid(RunSetError::Integrity("finalized archive changed"))
    ));
    std::fs::write(&path, &original).unwrap();
    let c = rusqlite::Connection::open(&path).unwrap();
    c.pragma_update(None, "user_version", 99).unwrap();
    drop(c);
    assert!(matches!(
        open_member(&root, &m),
        MemberState::Unsupported("archive version")
    ));
    std::fs::write(&path, &original).unwrap();
    m.attempt.as_mut().unwrap().expected_run_id += 1;
    assert!(matches!(
        open_member(&root, &m),
        MemberState::Invalid(RunSetError::Integrity("archive/spec binding"))
    ));
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn bounded_strict_manifests_reject_versions_keys_duplicate_keys_and_mutation() {
    let root = directory();
    let path = root.join("spec.json");
    let frozen = FrozenSpec::new(spec(NativePressureMethod::JacobiPcgV1)).unwrap();
    frozen.save_new(&path).unwrap();
    let original = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        original.replacen("\"schema_version\":1", "\"schema_version\":99", 1),
    )
    .unwrap();
    assert!(matches!(
        FrozenSpec::open(&path),
        Err(RunSetError::Unsupported("manifest version"))
    ));
    std::fs::write(
        &path,
        original.replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"surprise\":true",
            1,
        ),
    )
    .unwrap();
    assert!(matches!(
        FrozenSpec::open(&path),
        Err(RunSetError::Codec(_))
    ));
    std::fs::write(
        &path,
        original.replacen(
            "\"requested_steps\":3",
            "\"requested_steps\":3,\"requested_steps\":4",
            1,
        ),
    )
    .unwrap();
    assert!(matches!(
        FrozenSpec::open(&path),
        Err(RunSetError::Codec(_))
    ));
    std::fs::write(
        &path,
        original.replacen("\"requested_steps\":3", "\"requested_steps\":4", 1),
    )
    .unwrap();
    assert!(matches!(
        FrozenSpec::open(&path),
        Err(RunSetError::Integrity("spec digest"))
    ));
    std::fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(matches!(
        FrozenSpec::open(&path),
        Err(RunSetError::Invalid("manifest byte limit"))
    ));
    let mut revision = set();
    revision.members[1].differences_from_baseline.clear();
    assert!(matches!(
        revision.validate(),
        Err(RunSetError::Invalid("declared differences"))
    ));
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn spec_and_archive_paths_reject_invalid_boundaries() {
    let root = directory();
    let mut s = spec(NativePressureMethod::JacobiPcgV1);
    s.source.as_mut().unwrap().upper_m[0] = 1e100;
    assert!(matches!(
        s.validate(),
        Err(RunSetError::Invalid("spec source"))
    ));
    for relative in [
        "../outside.sqlite",
        "/tmp/elsewhere.sqlite",
        "nested/../run.sqlite",
        "a//b.sqlite",
        "a\\b.sqlite",
        "C:run.sqlite",
    ] {
        let mut m = member(NativePressureMethod::JacobiPcgV1);
        m.attempt.as_mut().unwrap().archive_relative_path = relative.into();
        assert!(matches!(
            open_member(&root, &m),
            MemberState::Invalid(RunSetError::Invalid("archive relative path"))
        ))
    }
    #[cfg(unix)]
    {
        let m = member(NativePressureMethod::JacobiPcgV1);
        let elsewhere = directory();
        produce(&elsewhere, &m, true);
        std::os::unix::fs::symlink(
            elsewhere.join(&m.attempt.as_ref().unwrap().archive_relative_path),
            root.join(&m.attempt.as_ref().unwrap().archive_relative_path),
        )
        .unwrap();
        assert!(matches!(
            open_member(&root, &m),
            MemberState::Invalid(RunSetError::Invalid("symlink archive component"))
        ));
        std::fs::remove_dir_all(elsewhere).unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_binding_checks_actual_budgets_fields_horizons_and_unrecorded_tail() {
    for case in 0..5 {
        let root = directory();
        let mut m = member(NativePressureMethod::JacobiPcgV1);
        let mut publications = produce(&root, &m, true);
        let path = root.join(&m.attempt.as_ref().unwrap().archive_relative_path);
        let reader = RecordingReader::open(&path, m.spec.spec.recording.limits()).unwrap();
        let run = reader.run().clone();
        drop(reader);
        std::fs::rename(&path, root.join("original.sqlite")).unwrap();
        let mut limits = m.spec.spec.recording.limits();
        let mut outcome = "completed";
        let mut accepted = Some(3);
        match case {
            0 => limits.max_records = 8,
            1 => {
                publications[0].fields.pop();
            }
            2 => {
                outcome = "failed";
                accepted = Some(4);
            }
            3 => {
                publications.truncate(2);
                outcome = "failed";
            }
            _ => {
                publications.clear();
                outcome = "failed";
                accepted = None;
            }
        }
        let mut writer = StepRecorder::create(&path, run, limits).unwrap();
        for publication in &publications {
            writer.append(publication).unwrap();
        }
        writer
            .finish(
                outcome,
                "declared failed accepted tail where applicable",
                accepted,
            )
            .unwrap();
        drop(writer);
        let result = m.attempt.as_mut().unwrap().pin_closed(&root, &m.spec);
        match case {
            0 => assert!(matches!(
                result,
                Err(RunSetError::Integrity("archive recording budget"))
            )),
            1 => assert!(matches!(
                result,
                Err(RunSetError::Recording(RecordingError::Invalid(
                    "carrier field inventory"
                )))
            )),
            2 => assert!(matches!(
                result,
                Err(RunSetError::Integrity("native publication horizon"))
            )),
            3 => {
                result.unwrap();
                let reader = ready(&root, &m);
                assert_eq!(reader.count().unwrap(), 2);
                assert_eq!(reader.status().unwrap().accepted_generation, Some(3));
                assert_eq!(reader.status().unwrap().outcome, "failed");
                assert!(matches!(reader.select(2), Err(RecordingError::Missing(2))));
            }
            _ => {
                result.unwrap();
                let reader = ready(&root, &m);
                assert_eq!(reader.count().unwrap(), 0);
                assert_eq!(reader.status().unwrap().outcome, "failed");
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn concurrent_manifest_publication_has_one_winner_and_no_overwrite() {
    let root = directory();
    let target = root.join("one.json");
    let first = FrozenSpec::new(spec(NativePressureMethod::JacobiPcgV1)).unwrap();
    let second = FrozenSpec::new(spec(NativePressureMethod::SymmetricGaussSeidelPcgV1)).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let tasks = [first.clone(), second.clone()]
        .into_iter()
        .map(|frozen| {
            let barrier = barrier.clone();
            let target = target.clone();
            std::thread::spawn(move || {
                barrier.wait();
                frozen.save_new(&target)
            })
        })
        .collect::<Vec<_>>();
    let results = tasks
        .into_iter()
        .map(|task| task.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|r|matches!(r,Err(RunSetError::Io(e)) if e.kind()==std::io::ErrorKind::AlreadyExists)).count(),1);
    let saved = FrozenSpec::open(&target).unwrap();
    assert!(saved == first || saved == second);
    std::fs::remove_dir_all(root).unwrap();
}

fn fresh_native_writer(root: &Path, m: &RunSetMember) -> (rheon::Simulation, StepRecorder) {
    let a = m.attempt.as_ref().unwrap();
    let simulation = m.spec.spec.create_simulation().unwrap();
    let run = bound_native_run(
        &m.spec,
        &simulation,
        SessionIdentity {
            run_id: a.expected_run_id,
            source_revision: a.expected_source_revision.clone(),
        },
        "real native acceptance host",
    )
    .unwrap();
    let limits = m.spec.spec.recording.limits();
    let mut writer =
        StepRecorder::create(&root.join(&a.archive_relative_path), run, limits).unwrap();
    writer
        .append(&carrier_publication(&simulation, 0, limits.max_record_bytes).unwrap())
        .unwrap();
    (simulation, writer)
}

#[test]
fn accepted_native_capacity_failure_cold_reopens_honest_unrecorded_tail() {
    let root = directory();
    let mut m = member(NativePressureMethod::JacobiPcgV1);
    m.spec.spec.recording.max_records = 2;
    m.spec = FrozenSpec::new(m.spec.spec).unwrap();
    let (mut simulation, mut writer) = fresh_native_writer(&root, &m);
    let s = &m.spec.spec;
    for interval in 0..2 {
        simulation
            .step(s.requested_dt_s, s.source_at(interval), |_| false)
            .unwrap();
        let publication =
            carrier_publication(&simulation, interval + 1, s.recording.max_record_bytes).unwrap();
        let result = writer.append(&publication);
        if interval == 0 {
            result.unwrap();
        } else {
            assert!(matches!(result, Err(RecordingError::Limit("record count"))));
            assert_eq!(
                simulation.state().generation,
                2,
                "recording cannot roll physics back"
            );
            assert!(matches!(
                writer.append(&publication),
                Err(RecordingError::Closed)
            ));
        }
    }
    assert!(matches!(
        writer.finish(
            "completed",
            "cannot conceal accepted unrecorded tail",
            Some(2)
        ),
        Err(RecordingError::Invalid(
            "completion must cover accepted tail"
        ))
    ));
    writer
        .finish(
            "failed",
            "accepted generation2 is unrecorded after capacity",
            Some(2),
        )
        .unwrap();
    drop(writer);
    drop(simulation);
    m.attempt
        .as_mut()
        .unwrap()
        .pin_closed(&root, &m.spec)
        .unwrap();
    let revision = RunSetRevision {
        schema_version: 1,
        set_id: "capacity-unrecorded-tail".into(),
        parent_sha256: None,
        baseline_member: m.member_id.clone(),
        members: vec![m],
    };
    revision.save_new(&root.join("failed-set.json")).unwrap();
    let reader = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "failed_tail_reader_child", "--nocapture"])
        .env("RHEON_FAILED_TAIL_READER", &root)
        .output()
        .unwrap();
    assert!(
        reader.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&reader.stdout),
        String::from_utf8_lossy(&reader.stderr)
    );
    assert!(
        String::from_utf8_lossy(&reader.stdout)
            .contains("accepted unrecorded native tail retained")
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn failed_tail_reader_child() {
    let Some(root) = std::env::var_os("RHEON_FAILED_TAIL_READER") else {
        return;
    };
    let root = PathBuf::from(root);
    let revision = RunSetRevision::open(&root.join("failed-set.json")).unwrap();
    let reader = ready(&root, &revision.members[0]);
    assert_eq!(reader.status().unwrap().outcome, "failed");
    assert_eq!(reader.status().unwrap().accepted_generation, Some(2));
    assert_eq!(reader.count().unwrap(), 2);
    let selected = reader.select(1).unwrap();
    assert_eq!(selected.generation, 1);
    RecordedCarrier::decode(reader.run(), &selected).unwrap();
    assert!(
        selected.field("temperature").is_none(),
        "unknown fields cannot become zeros"
    );
    assert!(matches!(reader.select(2), Err(RecordingError::Missing(2))));
    println!("accepted unrecorded native tail retained");
}
#[test]
fn cold_selected_member_is_stable_while_another_native_track_advances() {
    let root = directory();
    let mut first = member(NativePressureMethod::JacobiPcgV1);
    produce(&root, &first, true);
    first
        .attempt
        .as_mut()
        .unwrap()
        .pin_closed(&root, &first.spec)
        .unwrap();
    let first_reader = ready(&root, &first);
    let selected = first_reader.select(1).unwrap();
    let mut second = member(NativePressureMethod::SymmetricGaussSeidelPcgV1);
    let (mut simulation, mut writer) = fresh_native_writer(&root, &second);
    let original = RunSetRevision {
        schema_version: 1,
        set_id: "independent-native-tracks".into(),
        parent_sha256: None,
        baseline_member: first.member_id.clone(),
        members: vec![first.clone(), second.clone()],
    };
    original.save_new(&root.join("set-active.json")).unwrap();
    for interval in 0..second.spec.spec.requested_steps {
        let s = &second.spec.spec;
        simulation
            .step(s.requested_dt_s, s.source_at(interval), |_| false)
            .unwrap();
        writer
            .append(
                &carrier_publication(&simulation, interval + 1, s.recording.max_record_bytes)
                    .unwrap(),
            )
            .unwrap();
        let stable = first_reader.select(1).unwrap();
        assert_eq!(stable.generation, selected.generation);
        assert_eq!(stable.time.to_bits(), selected.time.to_bits());
        assert_eq!(stable.fields, selected.fields);
        let reopened = RunSetRevision::open(&root.join("set-active.json")).unwrap();
        match open_member(&root, &reopened.members[1]) {
            MemberState::Incomplete {
                recorded_count,
                status,
            } => {
                assert_eq!(recorded_count, interval + 2);
                assert_eq!(status.outcome, "active");
            }
            _ => panic!("advancing native track silently became a completed selection"),
        }
    }
    writer
        .finish("completed", "three native intervals", Some(3))
        .unwrap();
    drop(writer);
    drop(simulation);
    second
        .attempt
        .as_mut()
        .unwrap()
        .pin_closed(&root, &second.spec)
        .unwrap();
    let mut closed = original.clone();
    closed.parent_sha256 = Some(original.sha256().unwrap());
    closed.members[1] = second;
    closed.save_new(&root.join("set-closed.json")).unwrap();
    let old = RunSetRevision::open(&root.join("set-active.json")).unwrap();
    assert!(
        matches!(
            open_member(&root, &old.members[1]),
            MemberState::Unpinned { .. }
        ),
        "old revision cannot acquire a new final artifact binding"
    );
    let completed = RunSetRevision::open(&root.join("set-closed.json")).unwrap();
    assert_eq!(
        ready(&root, &completed.members[1])
            .select(1)
            .unwrap()
            .generation,
        1
    );
    assert_eq!(first_reader.select(1).unwrap().fields, selected.fields);
    drop(first_reader);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn real_native_terminal_busy_stays_incomplete_without_overwrite_recovery() {
    let root = directory();
    let mut m = member(NativePressureMethod::JacobiPcgV1);
    let (mut simulation, mut writer) = fresh_native_writer(&root, &m);
    simulation
        .step(m.spec.spec.requested_dt_s, m.spec.spec.source_at(0), |_| {
            false
        })
        .unwrap();
    writer
        .append(
            &carrier_publication(&simulation, 1, m.spec.spec.recording.max_record_bytes).unwrap(),
        )
        .unwrap();
    let path = root.join(&m.attempt.as_ref().unwrap().archive_relative_path);
    let lock = rusqlite::Connection::open(&path).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(
        matches!(writer.finish("cancelled","real writer collision",Some(1)),Err(RecordingError::Sqlite(error)) if error.sqlite_error_code()==Some(rusqlite::ErrorCode::DatabaseBusy))
    );
    lock.execute_batch("ROLLBACK").unwrap();
    drop(lock);
    assert!(matches!(
        writer.finish("cancelled", "cannot retry uncertain terminal", Some(1)),
        Err(RecordingError::Closed)
    ));
    drop(writer);
    drop(simulation);
    let exact = std::fs::read(&path).unwrap();
    match open_member(&root, &m) {
        MemberState::Incomplete {
            recorded_count,
            status,
        } => {
            assert_eq!(recorded_count, 2);
            assert_eq!(status.outcome, "active");
        }
        _ => panic!("failed terminal status inferred completion from valid tail"),
    }
    assert!(matches!(
        m.attempt.as_mut().unwrap().pin_closed(&root, &m.spec),
        Err(RunSetError::Invalid("archive is not terminal"))
    ));
    assert_eq!(
        std::fs::read(&path).unwrap(),
        exact,
        "reconciliation cannot overwrite or delete native evidence"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn capture_actual_constructor_feeds_existing_native_producer_without_debug_configuration() {
    for method in [
        NativePressureMethod::JacobiPcgV1,
        NativePressureMethod::SymmetricGaussSeidelPcgV1,
    ] {
        let expected = spec(method);
        let mut simulation = expected.create_simulation().unwrap();
        let captured = FrozenSpec::capture_constructor(
            &simulation,
            expected.requested_dt_s,
            expected.requested_steps,
            expected.source.clone(),
            expected.recording.limits(),
        )
        .unwrap();
        assert_eq!(captured.spec, expected);
        let run = bound_native_run(
            &captured,
            &simulation,
            SessionIdentity {
                run_id: 9,
                source_revision: "existing-native-owner".into(),
            },
            "captured actual constructor",
        )
        .unwrap();
        assert_eq!(run.counts, expected.counts_xyz);
        assert_eq!(
            run.solver.rsplit('/').next(),
            Some(method.implementation().id())
        );
        simulation
            .step(expected.requested_dt_s, expected.source_at(0), |_| false)
            .unwrap();
        assert!(matches!(
            FrozenSpec::capture_constructor(
                &simulation,
                expected.requested_dt_s,
                expected.requested_steps,
                expected.source.clone(),
                expected.recording.limits()
            ),
            Err(RunSetError::Invalid("native intent requires constructor"))
        ));
    }
}
