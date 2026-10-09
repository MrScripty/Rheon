//! Bounded synthetic algebra/source checks. NEVER calls the physical provider solve.
#[path = "../examples/viscous_boundary_wrench.rs"]
#[allow(dead_code)]
mod fixture;
#[path = "../examples/flat_wall_ritz/sha256.rs"]
mod retained_sha256;
use rheon::*;

fn authenticate_retained_records(n: usize, text: &[u8]) -> std::io::Result<()> {
    // Fixed reviewed-source pins; never supplied by the candidate archive.
    // These bind exact historical bytes, not their physical accuracy.
    let expected = match n {
        6 => "4b15fd627f461945f0f3090a5a4cb24da81d9017505f3e83514d60c3fe83f7c1",
        9 => "e66a04f2703f90b553db220d57f9fd9013565441195965d3145db7e964c3aa80",
        12 => "f0f5fb2d447e6c102c67a57da187e0306877b3a4237acb7d5d06c35c0cbfb598",
        _ => return Err(std::io::Error::other("unsupported retained acquisition")),
    };
    if text.len() > 1 << 20 || retained_sha256::hex(retained_sha256::digest(text)?) != expected {
        return Err(std::io::Error::other("retained archive identity mismatch"));
    }
    Ok(())
}

fn read_retained_records(n: usize, path: &std::path::Path) -> std::io::Result<String> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > 1 << 20 {
        return Err(std::io::Error::other("retained input cap"));
    }
    let mut text = String::new();
    file.take((1 << 20) + 1).read_to_string(&mut text)?;
    authenticate_retained_records(n, text.as_bytes())?;
    Ok(text)
}

fn external_observation_path(
    output: &std::path::Path,
    manifest: &std::path::Path,
) -> std::io::Result<std::path::PathBuf> {
    let absolute = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };
    let name = absolute
        .file_name()
        .ok_or_else(|| std::io::Error::other("output filename required"))?;
    let parent = absolute
        .parent()
        .ok_or_else(|| std::io::Error::other("output parent required"))?
        .canonicalize()?;
    let path = parent.join(name);
    if path.starts_with(manifest.canonicalize()?) || path.exists() {
        return Err(std::io::Error::other("fresh external CSV output required"));
    }
    Ok(path)
}
fn evidence(id: u64) -> ObstacleEvidenceRef {
    ObstacleEvidenceRef::new(id, 0, [id as u8; 32]).unwrap()
}
fn geometry(n: usize) -> StaticObstacleGeometry {
    fixture::geometry(
        [n as u64; 3],
        [3. / n as f64; 3],
        [0.; 3],
        [n / 3; 3],
        [2 * n / 3; 3],
    )
    .unwrap()
}
fn inputs() -> ObstaclePhysicalInputs {
    ObstaclePhysicalInputs {
        units: ObstacleStateUnits::Si,
        model: ObstacleStateModel::TransientStokes,
        material: ObstacleStateMaterial {
            density: 1.,
            dynamic_viscosity: 1.,
        },
        problem: evidence(1),
        boundary_evidence: evidence(2),
        boundary: ObstacleStateBoundary::StationaryNoSlipSolidSealedFreeSlipOuter,
        initial: ObstacleInitialData::KnownRest(evidence(3)),
        initial_time: 0.,
        forcing: ObstacleForcingHistory {
            kind: ObstacleForcingKind::DeclaredExternalForcing,
            evidence: evidence(4),
            start_time: 0.,
            end_time: 0.,
        },
    }
}
fn zero_force() -> FlatWallPolynomialForce {
    FlatWallPolynomialForce::new(Vec::new(), evidence(4)).unwrap()
}
fn buffers(g: &StaticObstacleGeometry) -> [Vec<f64>; 3] {
    std::array::from_fn(|a| vec![0.; g.grid().face_len([Axis::X, Axis::Y, Axis::Z][a])])
}
fn supplied<'g>(g: &'g StaticObstacleGeometry, v: &[Vec<f64>; 3]) -> ObstacleFlowState<'g> {
    let mut i = inputs();
    i.initial = ObstacleInitialData::Supplied(evidence(5));
    ObstacleFlowState::new(
        g,
        i,
        ObstacleStateFrame {
            time: 0.,
            generation: 0,
            origin: ObstacleStateOrigin::InitialData,
            errors: ObstacleVelocityErrors::unknown(),
        },
        [&v[0], &v[1], &v[2]],
        MAX_OBSTACLE_STATE_BYTES,
    )
    .unwrap()
}
fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 2e-12 * (1. + a.abs().max(b.abs())),
        "{a} != {b}"
    );
}
#[test]
fn all_basis_roster_support_stored_area_divergence_and_actual_allocations() {
    for (n, nq, rows, blocks) in [(6, 2, 176, 100), (9, 12, 681, 381), (12, 36, 1680, 936)] {
        let g = geometry(n);
        let provider = FlatWallRitzProvider::new(
            &g,
            zero_force(),
            inputs(),
            FLAT_WALL_RITZ_ENVELOPE,
            |_, _| false,
        )
        .unwrap();
        let plan = provider.plan();
        assert_eq!(plan.columns().len(), nq);
        assert_eq!(plan.sites().len(), rows);
        assert_eq!(plan.expected_blocks(), blocks);
        assert_eq!(provider.coverage().selected_rows, rows);
        assert!(provider.coverage().zero_corner_directed_sectors > 0);
        assert!(provider.coverage().outer_even_reflection);
        assert!(matches!(
            provider.solved_velocity(),
            Err(FlatWallRitzError::NotSolved)
        ));
        assert!(matches!(
            provider.acquire_initial_state(evidence(6)),
            Err(FlatWallRitzError::NotSolved)
        ));
        for i in 0..nq {
            let mut q = vec![0.; nq];
            q[i] = 1.;
            let mut v = buffers(&g);
            let [x, y, z] = &mut v;
            plan.apply_flux_curl(&q, [x, y, z]).unwrap();
            for t in plan.columns()[i].terms {
                assert_eq!(t.coefficient.abs(), 1. / g.open_areas(t.component)[t.face]);
            }
            let state = supplied(&g, &v);
            assert!(!state.pressure_available());
            assert_eq!(
                state.qualification(),
                ObstacleStateQualification::Unqualified
            );
            let (observed, bound) = flat_wall_divergence(&g, state.velocity()).unwrap();
            assert!(observed <= bound);
            assert!(bound < 2e-13);
            if n != 9 {
                assert_eq!(observed, 0.);
            }
            flat_wall_owned_traction(&state, FlatWallNormalTraction::FirstRowP1, |_, _| false)
                .unwrap();
        }
        let allocation = provider.allocation();
        assert!(allocation.assembly_peak_managed_bytes < FLAT_WALL_RITZ_ENVELOPE);
        println!(
            "allocation n={n} retained={} assembly={} acquisition={} rows={rows} blocks={blocks}",
            allocation.retained_managed_bytes,
            allocation.assembly_peak_managed_bytes,
            allocation.acquisition_peak_managed_bytes
        );
        assert!(
            FlatWallRitzProvider::new(
                &g,
                zero_force(),
                inputs(),
                allocation.assembly_peak_managed_bytes,
                |_, _| false
            )
            .is_ok()
        );
        assert!(
            FlatWallRitzProvider::new(
                &g,
                zero_force(),
                inputs(),
                allocation.assembly_peak_managed_bytes - 1,
                |_, _| false
            )
            .is_err()
        );
    }
}
#[test]
fn matrix_matches_actual_immutable_stress_action_for_synthetic_q() {
    for n in [6, 9] {
        let g = geometry(n);
        let provider = FlatWallRitzProvider::new(
            &g,
            zero_force(),
            inputs(),
            FLAT_WALL_RITZ_ENVELOPE,
            |_, _| false,
        )
        .unwrap();
        let plan = provider.plan();
        let nq = plan.columns().len();
        let q: Vec<f64> = (0..nq).map(|i| (i as f64 - 3.) / 16.).collect();
        let mut v = buffers(&g);
        let [x, y, z] = &mut v;
        plan.apply_flux_curl(&q, [x, y, z]).unwrap();
        let state = supplied(&g, &v);
        let grad = ObstacleVelocityGradient::new(
            &state,
            plan.sites(),
            MAX_OBSTACLE_STATE_BYTES,
            |_, _| false,
        )
        .unwrap();
        let stress =
            ObstacleViscousStress::new(&grad, MAX_OBSTACLE_STATE_BYTES, |_, _| false).unwrap();
        let mut scratch = vec![0.; plan.sites().len()];
        let mut f = buffers(&g);
        let [x, y, z] = &mut f;
        let w = stress
            .diagnose(&mut scratch, [x, y, z], |_, _| false)
            .unwrap();
        close(w.force_work, -w.dissipation);
        assert_eq!(provider.rhs(), vec![0.; nq]);
        for i in 0..nq {
            let actual: f64 = plan.columns()[i]
                .terms
                .iter()
                .map(|t| t.coefficient * f[t.component as usize][t.face])
                .sum();
            let expected: f64 = (0..nq).map(|j| provider.matrix()[i * nq + j] * q[j]).sum();
            close(actual, -expected);
            for j in 0..nq {
                assert_eq!(provider.matrix()[i * nq + j], provider.matrix()[j * nq + i]);
            }
        }
    }
}
#[test]
fn predicted_coarse_p1_and_normal_only_p2_torque_failures_are_preserved() {
    for n in [6, 12] {
        let g = geometry(n);
        let plan = FlatWallRitzPlan::new(&g, FLAT_WALL_RITZ_ENVELOPE).unwrap();
        let nq = plan.columns().len();
        for i in 0..nq {
            let mut q = vec![0.; nq];
            q[i] = 1. / 16.;
            let mut v = buffers(&g);
            let [x, y, z] = &mut v;
            plan.apply_flux_curl(&q, [x, y, z]).unwrap();
            let s = supplied(&g, &v);
            let p1 = flat_wall_owned_traction(&s, FlatWallNormalTraction::FirstRowP1, |_, _| false)
                .unwrap();
            close(p1.torque[2], (0.5 - 3. / n as f64) * p1.force[0]);
            for a in 0..3 {
                assert!(p1.arithmetic_force[a].contains(p1.force[a]));
                assert!(p1.arithmetic_torque[a].contains(p1.torque[a]));
            }
            if n == 6 {
                assert_eq!(p1.torque[2], 0.);
                let p2 =
                    flat_wall_owned_traction(&s, FlatWallNormalTraction::TwoPlaneP2, |_, _| false)
                        .unwrap();
                assert_eq!(p2.force[0], p1.force[0]);
                close(p2.torque[2], -0.5 * p2.force[0]);
                assert!(p2.force[0] < 0.);
                assert!(p2.torque[2] > 0.);
            }
        }
    }
}
#[test]
fn cell_average_tangential_option_is_owned_bounded_and_preserves_default() {
    for n in [6, 9, 12] {
        let g = geometry(n);
        let plan = FlatWallRitzPlan::new(&g, FLAT_WALL_RITZ_ENVELOPE).unwrap();
        let q: Vec<f64> = (0..plan.columns().len())
            .map(|i| (i as f64 + 1.) / 64.)
            .collect();
        let mut v = buffers(&g);
        let [x, y, z] = &mut v;
        plan.apply_flux_curl(&q, [x, y, z]).unwrap();
        let s = supplied(&g, &v);
        for normal in [
            FlatWallNormalTraction::FirstRowP1,
            FlatWallNormalTraction::TwoPlaneP2,
        ] {
            let old = flat_wall_owned_traction(&s, normal, |_, _| false).unwrap();
            let same = flat_wall_owned_traction_reconstructed(
                &s,
                FlatWallTangentialTraction::FirstCenterP1,
                normal,
                |_, _| false,
            )
            .unwrap();
            assert_eq!(old, same);
            let avg = flat_wall_owned_traction_reconstructed(
                &s,
                FlatWallTangentialTraction::TwoCellAverageP2,
                normal,
                |_, _| false,
            )
            .unwrap();
            assert_eq!(&old.force[1..], &avg.force[1..]);
            assert_eq!(old.torque[0], avg.torque[0]);
            assert_eq!(old.geometric_wall_area, avg.geometric_wall_area);
            assert_eq!(old.tangential_basis_area, avg.tangential_basis_area);
            assert_eq!(
                old.energy_sector_effective_area,
                avg.energy_sector_effective_area
            );
            for a in 0..3 {
                assert!(avg.arithmetic_force[a].contains(avg.force[a]));
                assert!(avg.arithmetic_torque[a].contains(avg.torque[a]));
            }
            assert!(
                flat_wall_owned_traction_reconstructed(
                    &s,
                    FlatWallTangentialTraction::TwoCellAverageP2,
                    normal,
                    |_, _| true
                )
                .is_err()
            );
        }
        assert_eq!(s.qualification(), ObstacleStateQualification::Unqualified);
        assert!(!s.pressure_available());
        assert_eq!(s.velocity(), [&v[0][..], &v[1][..], &v[2][..]]);
    }
}
#[test]
fn retained_record_identity_rejects_untrusted_and_altered_bytes() {
    for n in [6, 9, 12] {
        assert!(authenticate_retained_records(n, b"{\"kind\":\"header\"}\n").is_err());
        assert!(authenticate_retained_records(n, b"abc").is_err());
    }
    assert!(authenticate_retained_records(3, b"").is_err());
    let Ok(directory) = std::env::var("RHEON_RETAINED_FLAT_WALL") else {
        return;
    };
    for n in [6, 9, 12] {
        let text = read_retained_records(
            n,
            &std::path::Path::new(&directory).join(format!("n{n}/records.jsonl")),
        )
        .unwrap();
        let lines: Vec<_> = text.lines().collect();
        let velocity: Vec<_> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.starts_with("{\"kind\":\"velocity\""))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(
            velocity.len(),
            match n {
                6 => 8,
                9 => 36,
                12 => 96,
                _ => unreachable!(),
            }
        );
        // Exact original sparse bytes pass; the many omitted faces remain zero.
        authenticate_retained_records(n, text.as_bytes()).unwrap();
        let first = velocity[0];
        let missing = lines
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != first)
            .map(|(_, line)| *line)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let modified = text.replacen(
            lines[first],
            "{\"kind\":\"velocity\",\"component\":0,\"face\":0,\"value\":1.0}",
            1,
        );
        let duplicate = text.clone() + lines[first] + "\n";
        let mut reordered = lines.clone();
        reordered.swap(velocity[0], velocity[1]);
        let reordered = reordered.join("\n") + "\n";
        let header_only = lines[0].to_owned() + "\n";
        let zero_added =
            text.clone() + "{\"kind\":\"velocity\",\"component\":2,\"face\":0,\"value\":0.0}\n";
        let claimed = retained_sha256::hex(retained_sha256::digest(missing.as_bytes()).unwrap());
        let self_reported =
            missing.replacen('{', &format!("{{\"records_sha256\":\"{claimed}\","), 1);
        for bad in [
            missing,
            modified,
            duplicate,
            reordered,
            header_only,
            zero_added,
            self_reported,
        ] {
            assert!(authenticate_retained_records(n, bad.as_bytes()).is_err());
        }
    }
}

#[test]
fn observation_output_guard_resolves_relative_and_symlink_paths() {
    let cwd = std::env::current_dir().unwrap();
    assert!(
        external_observation_path(std::path::Path::new("forbidden-observation.csv"), &cwd).is_err()
    );
    let temp = std::env::temp_dir().join(format!("rheon-observation-guard-{}", std::process::id()));
    std::fs::create_dir(&temp).unwrap();
    let repo = temp.join("repo");
    std::fs::create_dir(&repo).unwrap();
    assert!(external_observation_path(&repo.join("forbidden.csv"), &repo).is_err());
    assert!(external_observation_path(&temp.join("missing/output.csv"), &repo).is_err());
    assert_eq!(
        external_observation_path(&temp.join("outside.csv"), &repo).unwrap(),
        temp.canonicalize().unwrap().join("outside.csv")
    );
    #[cfg(unix)]
    {
        let alias = temp.join("alias");
        std::os::unix::fs::symlink(&repo, &alias).unwrap();
        assert!(external_observation_path(&alias.join("forbidden.csv"), &repo).is_err());
        assert!(external_observation_path(&repo.join("forbidden.csv"), &alias).is_err());
        std::fs::remove_file(alias).unwrap();
    }
    assert!(!repo.join("forbidden.csv").exists());
    assert!(!temp.join("outside.csv").exists());
    assert!(!temp.join("missing").exists());
    std::fs::remove_dir(repo).unwrap();
    std::fs::remove_dir(temp).unwrap();
}

#[test]
fn optional_retained_field_observation_reads_velocities_without_solving() {
    // The ordinary test suite needs no archived campaign. Qualification sets
    // both paths explicitly and hashes every retained input before/afterward.
    let Ok(directory) = std::env::var("RHEON_RETAINED_FLAT_WALL") else {
        return;
    };
    let output = std::env::var("RHEON_FLAT_WALL_OBSERVATION_OUTPUT")
        .expect("retained test requires a fresh external CSV output");
    use std::io::Write;
    let path = external_observation_path(
        std::path::Path::new(&output),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    )
    .unwrap();
    // Preflight all three identities before creating an output. Each actual
    // reconstruction authenticates its own read again, before owning state.
    for n in [6, 9, 12] {
        read_retained_records(
            n,
            &std::path::Path::new(&directory).join(format!("n{n}/records.jsonl")),
        )
        .unwrap();
    }
    let mut csv = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    fn number(line: &str, key: &str) -> f64 {
        line.split(&format!("\"{key}\":"))
            .nth(1)
            .unwrap()
            .split([',', '}'])
            .next()
            .unwrap()
            .parse()
            .unwrap()
    }
    for n in [6, 9, 12] {
        let text = read_retained_records(
            n,
            &std::path::Path::new(&directory).join(format!("n{n}/records.jsonl")),
        )
        .unwrap();
        let g = geometry(n);
        let mut v = buffers(&g);
        let header = text.lines().next().unwrap();
        assert!(header.contains("\"mode\":\"numerical_reduced_ritz_solve\""));
        assert!(header.contains("\"head\":\"3f7d1ce7b39d1f63cba8385e2b0ebb46953d00da\""));
        assert_eq!(number(header, "n"), n as f64);
        let mut observed = std::collections::BTreeSet::new();
        for line in text
            .lines()
            .filter(|line| line.starts_with("{\"kind\":\"velocity\""))
        {
            assert!(line.len() <= 8192);
            let a = number(line, "component") as usize;
            let face = number(line, "face") as usize;
            assert!(a < 3 && face < v[a].len() && observed.insert((a, face)));
            let value = number(line, "value");
            assert!(value.is_finite());
            v[a][face] = value;
        }
        assert!(!observed.is_empty());
        let s = supplied(&g, &v);
        for normal in [
            FlatWallNormalTraction::FirstRowP1,
            FlatWallNormalTraction::TwoPlaneP2,
        ] {
            let w = flat_wall_owned_traction_reconstructed(
                &s,
                FlatWallTangentialTraction::TwoCellAverageP2,
                normal,
                |_, _| false,
            )
            .unwrap();
            write!(csv, "{n},{normal:?}").unwrap();
            for value in w.force.into_iter().chain(w.torque) {
                write!(csv, ",{value:?}").unwrap();
            }
            for interval in w.arithmetic_force.into_iter().chain(w.arithmetic_torque) {
                write!(csv, ",{:?},{:?}", interval.lower, interval.upper).unwrap();
            }
            writeln!(csv).unwrap();
        }
        assert_eq!(s.qualification(), ObstacleStateQualification::Unqualified);
    }
    assert!(csv.metadata().unwrap().len() <= 65536);
}
#[test]
fn analytic_polynomial_dual_integration_and_stored_volume_are_distinct() {
    let g = geometry(6);
    let force = FlatWallPolynomialForce::new(
        vec![FlatWallForceTerm {
            component: Axis::X,
            coefficient: 1.,
            powers: [1, 1, 0],
        }],
        evidence(4),
    )
    .unwrap();
    let f = g.grid().face_index(Axis::X, [3, 1, 2]).unwrap();
    let value = force.integrate_face(&g, Axis::X, f).unwrap();
    assert_eq!(value.force, 3. / 64.);
    assert!(value.arithmetic_interval.contains(3. / 64.));
    assert_eq!(value.clipped_geometric_volume, 1. / 8.);
    assert_eq!(value.stored_face_volume, 1. / 8.);
    let face = g.grid().face_index(Axis::X, [2, 1, 2]).unwrap();
    let clipped = force.integrate_face(&g, Axis::X, face).unwrap();
    assert_eq!(clipped.clipped_geometric_volume, 1. / 16.);
    assert_eq!(clipped.stored_face_volume, 1. / 8.);
    assert_eq!(force.integrate_face(&g, Axis::Z, 0).unwrap().force, 0.);
    assert!(
        force
            .integrate_face(&g, Axis::X, g.grid().face_len(Axis::X))
            .is_err()
    );
}
#[test]
fn unsupported_geometry_force_provenance_support_and_shapes_refuse() {
    let wrong = fixture::geometry([6; 3], [1.; 3], [0.; 3], [2; 3], [4; 3]).unwrap();
    assert!(FlatWallRitzPlan::new(&wrong, FLAT_WALL_RITZ_ENVELOPE).is_err());
    for coefficient in [f64::NAN, f64::INFINITY, f64::MIN_POSITIVE / 2.] {
        assert!(
            FlatWallPolynomialForce::new(
                vec![FlatWallForceTerm {
                    component: Axis::X,
                    coefficient,
                    powers: [0; 3]
                }],
                evidence(4)
            )
            .is_err()
        );
    }
    assert!(
        FlatWallPolynomialForce::new(
            vec![FlatWallForceTerm {
                component: Axis::X,
                coefficient: 1.,
                powers: [17, 0, 0]
            }],
            evidence(4)
        )
        .is_err()
    );
    let g = geometry(6);
    let mut bad = inputs();
    bad.forcing.evidence = evidence(8);
    assert!(
        FlatWallRitzProvider::new(&g, zero_force(), bad, FLAT_WALL_RITZ_ENVELOPE, |_, _| false)
            .is_err()
    );
    for limit in [0, FLAT_WALL_RITZ_ENVELOPE + 1] {
        assert!(FlatWallRitzPlan::new(&g, limit).is_err());
    }
    let plan = FlatWallRitzPlan::new(&g, FLAT_WALL_RITZ_ENVELOPE).unwrap();
    let mut v = buffers(&g);
    let [x, y, z] = &mut v;
    assert!(plan.apply_flux_curl(&[], [x, y, z]).is_err());
    let outside = g.grid().face_index(Axis::X, [1, 0, 0]).unwrap();
    v[0][outside] = 1.;
    let s = supplied(&g, &v);
    assert!(matches!(
        flat_wall_owned_traction(&s, FlatWallNormalTraction::FirstRowP1, |_, _| false),
        Err(FlatWallRitzError::CoverageMismatch)
    ));
}
#[test]
fn coverage_and_traction_cancellation_leave_owned_snapshot_unchanged() {
    let g = geometry(6);
    let plan = FlatWallRitzPlan::new(&g, FLAT_WALL_RITZ_ENVELOPE).unwrap();
    let v = buffers(&g);
    let s = supplied(&g, &v);
    let mut before = Vec::new();
    s.write_checkpoint(&mut before).unwrap();
    assert!(plan.verify_coverage(&s, |_, _| true).is_err());
    assert!(flat_wall_owned_traction(&s, FlatWallNormalTraction::TwoPlaneP2, |_, _| true).is_err());
    let mut after = Vec::new();
    s.write_checkpoint(&mut after).unwrap();
    assert_eq!(before, after);
    assert!(
        FlatWallRitzProvider::new(
            &g,
            zero_force(),
            inputs(),
            FLAT_WALL_RITZ_ENVELOPE,
            |_, _| true
        )
        .is_err()
    );
}
