//! Structural tests: all evidence references below are explicit synthetic test declarations.
//! No test certifies physical inputs, dynamics, or a caller-declared error bound.
#[path = "../examples/viscous_boundary_wrench.rs"]
#[allow(dead_code)]
mod fixture;
use rheon::*;
use std::io::{self, Read, Write};

fn evidence(id: u64) -> ObstacleEvidenceRef {
    ObstacleEvidenceRef::new(id, 0, [id as u8; 32]).unwrap()
}
fn geometry(n: usize) -> StaticObstacleGeometry {
    fixture::geometry(
        [n as u64; 3],
        [3.0 / n as f64; 3],
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
            density: 1000.,
            dynamic_viscosity: 0.001,
        },
        problem: evidence(1),
        boundary_evidence: evidence(2),
        boundary: ObstacleStateBoundary::StationaryNoSlipSolidSealedFreeSlipOuter,
        initial: ObstacleInitialData::KnownRest(evidence(3)),
        initial_time: 0.,
        forcing: ObstacleForcingHistory {
            kind: ObstacleForcingKind::ExplicitNoForcing,
            evidence: evidence(4),
            start_time: 0.,
            end_time: 0.,
        },
    }
}
fn frame() -> ObstacleStateFrame {
    ObstacleStateFrame {
        time: 0.,
        generation: 0,
        origin: ObstacleStateOrigin::InitialData,
        errors: ObstacleVelocityErrors::unknown(),
    }
}
fn fields(g: &StaticObstacleGeometry) -> [Vec<f64>; 3] {
    std::array::from_fn(|a| vec![0.; g.grid().face_len([Axis::X, Axis::Y, Axis::Z][a])])
}
fn slices(v: &[Vec<f64>; 3]) -> [&[f64]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn active(g: &StaticObstacleGeometry, axis: Axis) -> usize {
    let a = fixture::axis(axis);
    let n = g.grid().face_counts(axis);
    (0..g.grid().face_len(axis))
        .find(|&f| {
            let c = [f % n[0], (f / n[0]) % n[1], f / (n[0] * n[1])];
            c[a] > 0 && c[a] < g.grid().counts()[a] && g.open_areas(axis)[f] > 0.
        })
        .unwrap()
}
fn checkpoint(s: &ObstacleFlowState<'_>) -> Vec<u8> {
    let mut b = Vec::new();
    s.write_checkpoint(&mut b).unwrap();
    b
}
fn assert_same(s: &ObstacleFlowState<'_>, t: &ObstacleFlowState<'_>) {
    assert!(std::ptr::eq(s.geometry(), t.geometry()));
    assert_eq!(s.physical_inputs(), t.physical_inputs());
    assert_eq!(s.frame(), t.frame());
    for a in 0..3 {
        assert_eq!(
            s.velocity()[a]
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            t.velocity()[a]
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(t.qualification(), ObstacleStateQualification::Unqualified);
    assert!(!t.pressure_available());
}
// Wire v1 is a public format, not the private encoder implementation. Metadata
// starts after the complete represented geometry, including full area/label arrays.
fn metadata_offset(g: &StaticObstacleGeometry) -> usize {
    12 + 3 * 8
        + 4 * 3 * 8
        + 2 * 8
        + 8
        + 8
        + g.surface().vertices().len() * 24
        + 8
        + g.surface().triangles().len() * 24
        + 8
        + [
            g.fluid_volumes().len(),
            g.open_areas(Axis::X).len(),
            g.open_areas(Axis::Y).len(),
            g.open_areas(Axis::Z).len(),
            g.component_labels().len(),
        ]
        .into_iter()
        .map(|n| 8 + n * 8)
        .sum::<usize>()
}

#[test]
fn owns_values_and_preserves_bits_and_unknown_qualification() {
    let g = geometry(3);
    let mut v = fields(&g);
    let mut i = inputs();
    let mut f = frame();
    i.initial = ObstacleInitialData::Supplied(evidence(3));
    i.forcing.kind = ObstacleForcingKind::DeclaredExternalForcing;
    i.forcing.end_time = 2.;
    f.time = 2.;
    f.generation = 7;
    f.origin = ObstacleStateOrigin::ExternalSnapshot(evidence(5));
    f.errors.face_linf[1] = ObstacleVelocityError::CallerDeclaredUpperBound {
        value: 0.,
        evidence: evidence(6),
    };
    f.errors.roundoff = ObstacleVelocityError::CallerDeclaredUpperBound {
        value: 0.25,
        evidence: evidence(7),
    };
    for a in [Axis::X, Axis::Y, Axis::Z] {
        v[fixture::axis(a)][active(&g, a)] = (fixture::axis(a) as f64 + 1.) / 4.;
    }
    v[0][0] = -0.;
    let s = ObstacleFlowState::new(&g, i, f, slices(&v), MAX_OBSTACLE_STATE_BYTES).unwrap();
    assert_eq!(s.velocity()[0][0].to_bits(), (-0f64).to_bits());
    let before = checkpoint(&s);
    v[0][active(&g, Axis::X)] = 99.;
    v[1].clear();
    assert_eq!(checkpoint(&s), before);
    let t =
        ObstacleFlowState::read_checkpoint(&g, &mut before.as_slice(), MAX_OBSTACLE_STATE_BYTES)
            .unwrap();
    assert_same(&s, &t);
    assert_eq!(checkpoint(&t), before);
}

#[test]
fn known_rest_at_nonzero_initial_time_and_signed_zero_metadata() {
    let g = geometry(3);
    let v = fields(&g);
    let mut i = inputs();
    let mut f = frame();
    i.initial_time = 3.;
    f.time = 3.;
    i.forcing.start_time = 3.;
    i.forcing.end_time = 3.;
    let s = ObstacleFlowState::new(&g, i, f, slices(&v), MAX_OBSTACLE_STATE_BYTES).unwrap();
    let b = checkpoint(&s);
    assert_same(
        &s,
        &ObstacleFlowState::read_checkpoint(&g, &mut b.as_slice(), MAX_OBSTACLE_STATE_BYTES)
            .unwrap(),
    );
    i.initial_time = -0.;
    f.time = 0.;
    i.forcing.start_time = 0.;
    i.forcing.end_time = -0.;
    let s = ObstacleFlowState::new(&g, i, f, slices(&v), MAX_OBSTACLE_STATE_BYTES).unwrap();
    let b = checkpoint(&s);
    let t = ObstacleFlowState::read_checkpoint(&g, &mut b.as_slice(), MAX_OBSTACLE_STATE_BYTES)
        .unwrap();
    assert_eq!(
        t.physical_inputs().initial_time.to_bits(),
        (-0f64).to_bits()
    );
    assert_eq!(
        t.physical_inputs().forcing.end_time.to_bits(),
        (-0f64).to_bits()
    );
}

#[test]
fn rejects_missing_material_chronology_and_error_claims() {
    assert!(matches!(
        ObstacleEvidenceRef::new(0, 0, [1; 32]),
        Err(ObstacleStateError::MissingEvidence)
    ));
    assert!(matches!(
        ObstacleEvidenceRef::new(1, 0, [0; 32]),
        Err(ObstacleStateError::MissingEvidence)
    ));
    let g = geometry(3);
    let v = fields(&g);
    for bad in [0., -1., f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        for density in [true, false] {
            let mut i = inputs();
            if density {
                i.material.density = bad;
            } else {
                i.material.dynamic_viscosity = bad;
            }
            assert!(matches!(
                ObstacleFlowState::new(&g, i, frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES),
                Err(ObstacleStateError::InvalidMaterial)
            ));
        }
    }
    for bad in [-1., f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        let mut i = inputs();
        i.initial_time = bad;
        assert!(matches!(
            ObstacleFlowState::new(&g, i, frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES),
            Err(ObstacleStateError::InvalidChronology)
        ));
        let mut f = frame();
        f.errors.spatial = ObstacleVelocityError::CallerDeclaredUpperBound {
            value: bad,
            evidence: evidence(6),
        };
        assert!(matches!(
            ObstacleFlowState::new(&g, inputs(), f, slices(&v), MAX_OBSTACLE_STATE_BYTES),
            Err(ObstacleStateError::InvalidErrorDeclaration)
        ));
    }
    for case in 0..5 {
        let mut i = inputs();
        let mut f = frame();
        match case {
            0 => i.forcing.start_time = 1.,
            1 => i.forcing.end_time = 1.,
            2 => f.time = 1.,
            3 => f.generation = 1,
            _ => {
                i.initial_time = 2.;
                i.forcing.start_time = 2.;
            }
        }
        assert!(matches!(
            ObstacleFlowState::new(&g, i, f, slices(&v), MAX_OBSTACLE_STATE_BYTES),
            Err(ObstacleStateError::InvalidChronology)
        ));
    }
}

#[test]
fn rejects_bad_fields_and_traces_without_changing_inputs() {
    let g = geometry(3);
    let mut v = fields(&g);
    let a = active(&g, Axis::X);
    for bad in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(1),
    ] {
        v[0][a] = bad;
        assert!(matches!(
            ObstacleFlowState::new(&g, inputs(), frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES),
            Err(ObstacleStateError::InvalidVelocity { .. })
        ));
        assert_eq!(v[0][a].to_bits(), bad.to_bits());
    }
    v[0][a] = 1.;
    assert!(matches!(
        ObstacleFlowState::new(&g, inputs(), frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES),
        Err(ObstacleStateError::InvalidRest)
    ));
    let mut i = inputs();
    i.initial = ObstacleInitialData::Supplied(evidence(3));
    v[0][a] = 0.;
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let d = fixture::axis(axis);
        v[d][0] = 1.;
        assert!(matches!(
            ObstacleFlowState::new(&g, i, frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES),
            Err(ObstacleStateError::NonstationaryTrace { .. })
        ));
        v[d][0] = 0.;
        let blocked = g.open_areas(axis).iter().position(|&x| x == 0.).unwrap();
        v[d][blocked] = 1.;
        assert!(matches!(
            ObstacleFlowState::new(&g, i, frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES),
            Err(ObstacleStateError::NonstationaryTrace { .. })
        ));
        v[d][blocked] = 0.;
    }
    v[2].pop();
    assert!(matches!(
        ObstacleFlowState::new(&g, i, frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES),
        Err(ObstacleStateError::ShapeMismatch { axis: Axis::Z })
    ));
}

#[test]
fn bounded_payload_accounts_geometry_once_at_three_sizes() {
    for n in [3, 6, 12] {
        let g = geometry(n);
        let v = fields(&g);
        let planned = ObstacleFlowState::planned_payload_bytes(&g).unwrap();
        assert!(planned < MAX_OBSTACLE_STATE_BYTES);
        assert!(
            matches!(ObstacleFlowState::new(&g,inputs(),frame(),slices(&v),planned-1),Err(ObstacleStateError::BufferLimit{required,..}) if required==planned)
        );
        let s = ObstacleFlowState::new(&g, inputs(), frame(), slices(&v), planned).unwrap();
        assert_eq!(s.combined_payload_bytes(), planned);
        assert_eq!(
            s.combined_payload_bytes(),
            s.owned_payload_bytes() + g.allocation().retained_bytes
        );
        let b = checkpoint(&s);
        assert!(matches!(
            ObstacleFlowState::read_checkpoint(&g, &mut b.as_slice(), planned - 1),
            Err(ObstacleStateError::BufferLimit { .. })
        ));
        assert_same(
            &s,
            &ObstacleFlowState::read_checkpoint(&g, &mut b.as_slice(), planned).unwrap(),
        );
    }
    let g = geometry(3);
    let v = fields(&g);
    for cap in [0, MAX_OBSTACLE_STATE_BYTES + 1, usize::MAX] {
        assert!(matches!(
            ObstacleFlowState::new(&g, inputs(), frame(), slices(&v), cap),
            Err(ObstacleStateError::InvalidBudget { .. })
        ));
    }
}

#[test]
fn codec_rejects_every_truncation_trailing_data_and_unknown_tags() {
    let g = geometry(3);
    let v = fields(&g);
    let s = ObstacleFlowState::new(&g, inputs(), frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES)
        .unwrap();
    let b = checkpoint(&s);
    let m = metadata_offset(&g);
    for end in 0..b.len() {
        assert!(
            ObstacleFlowState::read_checkpoint(&g, &mut &b[..end], MAX_OBSTACLE_STATE_BYTES)
                .is_err(),
            "accepted prefix {end}"
        );
    }
    let mut c = b.clone();
    c.push(0);
    assert!(matches!(
        ObstacleFlowState::read_checkpoint(&g, &mut c.as_slice(), MAX_OBSTACLE_STATE_BYTES),
        Err(ObstacleStateError::InvalidCheckpoint)
    ));
    for (offset, value) in [
        (0, 0),
        (8, 2),
        (m, 2),
        (m + 1, 3),
        (m + 2, 2),
        (m + 3, 1),
        (m + 4, 1),
        (m + 141, 3),
        (m + 190, 3),
        (m + 255, 3),
        (m + 256, 8),
        (m + 257, 2),
    ] {
        let mut c = b.clone();
        c[offset] = value;
        assert!(
            ObstacleFlowState::read_checkpoint(&g, &mut c.as_slice(), MAX_OBSTACLE_STATE_BYTES)
                .is_err(),
            "accepted tag at {offset}"
        );
    }
    for offset in [m + 45, m + 93, m + 142, m + 191] {
        let mut c = b.clone();
        c[offset..offset + 8].fill(0);
        assert!(matches!(
            ObstacleFlowState::read_checkpoint(&g, &mut c.as_slice(), MAX_OBSTACLE_STATE_BYTES),
            Err(ObstacleStateError::MissingEvidence)
        ));
    }
    for axis in 0..3 {
        let mut c = b.clone();
        c[m + 266 + axis * 8..m + 274 + axis * 8].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(matches!(
            ObstacleFlowState::read_checkpoint(&g, &mut c.as_slice(), MAX_OBSTACLE_STATE_BYTES),
            Err(ObstacleStateError::InvalidCheckpoint)
        ));
    }
    assert_eq!(checkpoint(&s), b);
}

#[test]
fn codec_checks_full_geometry_and_values_instead_of_stamp_only() {
    let g = geometry(3);
    let v = fields(&g);
    let s = ObstacleFlowState::new(&g, inputs(), frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES)
        .unwrap();
    let b = checkpoint(&s);
    let m = metadata_offset(&g);
    let g2 = fixture::geometry([3; 3], [1.; 3], [1.; 3], [1; 3], [2; 3]).unwrap();
    assert_eq!(g.surface().stamp(), g2.surface().stamp());
    assert!(matches!(
        ObstacleFlowState::read_checkpoint(&g2, &mut b.as_slice(), MAX_OBSTACLE_STATE_BYTES),
        Err(ObstacleStateError::GeometryMismatch)
    ));
    // Counts, origin, spacing, bounds, stamp, tolerance, vertices, triangle indices,
    // component count, volume, each area family, and component labels.
    let mut offsets = vec![
        12, 36, 60, 84, 108, 132, 148, 156, 164, 364, 372, 660, 668, 676,
    ];
    let mut p = 668 + g.fluid_volumes().len() * 8;
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        offsets.push(p);
        offsets.push(p + 8);
        p += 8 + g.open_areas(axis).len() * 8;
    }
    offsets.push(p);
    offsets.push(p + 8);
    for offset in offsets {
        let mut c = b.clone();
        c[offset] ^= 1;
        assert!(
            matches!(
                ObstacleFlowState::read_checkpoint(&g, &mut c.as_slice(), MAX_OBSTACLE_STATE_BYTES),
                Err(ObstacleStateError::GeometryMismatch)
            ),
            "geometry offset {offset}"
        );
    }
    for bad in [1., f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        let mut c = b.clone();
        c[m + 290..m + 298].copy_from_slice(&bad.to_bits().to_le_bytes());
        assert!(
            ObstacleFlowState::read_checkpoint(&g, &mut c.as_slice(), MAX_OBSTACLE_STATE_BYTES)
                .is_err()
        );
    }
}

struct FailedIo;
impl Read for FailedIo {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("injected read failure"))
    }
}
impl Write for FailedIo {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("injected write failure"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn io_errors_leave_existing_owner_unchanged() {
    let g = geometry(3);
    let v = fields(&g);
    let s = ObstacleFlowState::new(&g, inputs(), frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES)
        .unwrap();
    let b = checkpoint(&s);
    assert!(matches!(
        s.write_checkpoint(&mut FailedIo),
        Err(ObstacleStateError::Io(_))
    ));
    assert!(matches!(
        ObstacleFlowState::read_checkpoint(&g, &mut FailedIo, MAX_OBSTACLE_STATE_BYTES),
        Err(ObstacleStateError::Io(_))
    ));
    assert_eq!(checkpoint(&s), b);
}

#[test]
fn rejects_unpadded_and_non_grid_aligned_retained_boxes() {
    let g = fixture::geometry([4; 3], [1.; 3], [0.; 3], [0, 1, 1], [2; 3]).unwrap();
    let v = fields(&g);
    assert!(matches!(
        ObstacleFlowState::new(&g, inputs(), frame(), slices(&v), MAX_OBSTACLE_STATE_BYTES),
        Err(ObstacleStateError::UnsupportedGeometry)
    ));
    let good = fixture::geometry([4; 3], [1.; 3], [0.; 3], [1; 3], [2; 3]).unwrap();
    let mut vertices = good.surface().vertices().to_vec();
    for vertex in &mut vertices {
        if vertex[0] == 2. {
            vertex[0] = 2.5;
        }
    }
    let surface = TriangleSurface::new(
        good.stamp(),
        vertices,
        good.surface().triangles().to_vec(),
        SurfaceSettings::default(),
    )
    .unwrap();
    let unsupported = StaticObstacleGeometry::new(
        good.grid().clone(),
        surface,
        MAX_OBSTACLE_STATE_BYTES,
        |_, _| false,
    )
    .unwrap();
    let v = fields(&unsupported);
    assert!(matches!(
        ObstacleFlowState::new(
            &unsupported,
            inputs(),
            frame(),
            slices(&v),
            MAX_OBSTACLE_STATE_BYTES
        ),
        Err(ObstacleStateError::UnsupportedGeometry)
    ));
    assert!(matches!(
        ObstacleFlowState::read_checkpoint(&unsupported, &mut &b""[..], MAX_OBSTACLE_STATE_BYTES),
        Err(ObstacleStateError::UnsupportedGeometry)
    ));
}
