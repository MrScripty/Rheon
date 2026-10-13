#[allow(dead_code)]
#[path = "../experiments/plate_coupling.rs"]
mod plate_coupling;
use plate_coupling::{Error, Interval, Plate, Workspace};
use rheon::{
    Axis, GridGeometry, ObstacleShearWall, StaticObstacleGeometry, StaticObstacleShear,
    SurfaceSettings, SurfaceStamp, TriangleSurface,
};
fn mesh(lo: [f64; 3], hi: [f64; 3]) -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 42, version: 7 },
        (0..8)
            .map(|c| std::array::from_fn(|d| if c & (1 << d) == 0 { lo[d] } else { hi[d] }))
            .collect(),
        vec![
            [0, 2, 3],
            [0, 3, 1],
            [4, 5, 7],
            [4, 7, 6],
            [0, 1, 5],
            [0, 5, 4],
            [2, 6, 7],
            [2, 7, 3],
            [0, 4, 6],
            [0, 6, 2],
            [1, 3, 7],
            [1, 7, 5],
        ],
        SurfaceSettings::default(),
    )
    .unwrap()
}

fn owner(layers: u64, axis: Axis, wall: f64) -> StaticObstacleGeometry {
    let d = match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    };
    let mut counts = [1; 3];
    counts[d] = layers;
    let mut spacing = [1.0; 3];
    spacing[d] = 1.0 / layers as f64;
    let mut lo = [0.0; 3];
    lo[d] = -1.0;
    let mut hi = [1.0; 3];
    hi[d] = wall;
    StaticObstacleGeometry::new(
        GridGeometry::new(counts, spacing, [0.0; 3]).unwrap(),
        mesh(lo, hi),
        1_000_000,
        |_, _| false,
    )
    .unwrap()
}
fn contains(x: Interval, value: f64) {
    assert!(x.lo <= value && value <= x.hi, "{x:?} misses {value}");
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 2e-13, "{a} != {b}");
}
fn dynamic() -> Plate {
    Plate::Dynamic { mass_kg: 1.0 }
}
#[test]
fn prescribed_couette_impulse_and_actuator_work_match_hand_case() {
    let g = owner(1, Axis::Y, 0.0);
    let op = StaticObstacleShear::new(
        &g,
        Axis::Y,
        1.0,
        1.0,
        ObstacleShearWall::NoSlip,
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    let mut w = Workspace::new(&op, Axis::X, [Plate::Prescribed; 2], 1_000_000).unwrap();
    let mut v = [0.0];
    let mut plates = [0.0, 1.0];
    let r = w.step(&mut v, &mut plates, 0.25, |_, _| false).unwrap();
    assert_eq!(v, [0.25]);
    assert_eq!(plates, [0.0, 1.0]);
    contains(r.fluid_impulse_n_s[0], -1.0 / 8.0);
    contains(r.fluid_impulse_n_s[1], 3.0 / 8.0);
    contains(r.kinetic_after_j, 1.0 / 32.0);
    contains(r.increment_energy_j, 1.0 / 32.0);
    contains(r.dissipation_j, 5.0 / 16.0);
    contains(r.prescribed_work_j, 3.0 / 8.0);
}
#[test]
fn dynamic_plate_recoil_and_mixed_boundary_work_are_coupled() {
    let g = owner(1, Axis::Y, 0.0);
    let op = StaticObstacleShear::new(
        &g,
        Axis::Y,
        1.0,
        1.0,
        ObstacleShearWall::NoSlip,
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    for (modes, initial, target) in [
        (
            [dynamic(), Plate::Prescribed],
            [1.0, 0.0],
            [8.0 / 11.0, 2.0 / 11.0, 0.0],
        ),
        ([dynamic(); 2], [1.0, -1.0], [2.0 / 3.0, 0.0, -2.0 / 3.0]),
    ] {
        let mut w = Workspace::new(&op, Axis::X, modes, 1_000_000).unwrap();
        let mut v = [0.0];
        let mut plates = initial;
        let r = w.step(&mut v, &mut plates, 0.25, |_, _| false).unwrap();
        close(plates[0], target[0]);
        close(v[0], target[1]);
        close(plates[1], target[2]);
        assert!(r.kinetic_change_j.hi < 0.0);
        for end in 0..2 {
            if modes[end] == dynamic() {
                close(
                    plates[end] - initial[end],
                    -0.5 * (r.fluid_impulse_n_s[end].lo + r.fluid_impulse_n_s[end].hi),
                );
            }
        }
    }
}
#[test]
fn common_translation_and_free_slip_decoupling() {
    let g = owner(3, Axis::Y, 0.0);
    let op = StaticObstacleShear::new(
        &g,
        Axis::Y,
        1.0,
        1.0,
        ObstacleShearWall::Navier { beta: 0.0 },
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    let mut w = Workspace::new(&op, Axis::Z, [dynamic(); 2], 1_000_000).unwrap();
    let mut v = [2.0; 3];
    let mut plates = [2.0; 2];
    let r = w.step(&mut v, &mut plates, 0.125, |_, _| false).unwrap();
    assert_eq!(v, [2.0; 3]);
    assert_eq!(plates, [2.0; 2]);
    assert_eq!(r.kinetic_change_j.lo, 0.0);
    plates[0] = -0.7;
    let r = w.step(&mut v, &mut plates, 0.125, |_, _| false).unwrap();
    assert_eq!(plates[0], -0.7);
    contains(r.fluid_impulse_n_s[0], 0.0);
}
#[test]
fn steady_couette_all_axes_and_cut_first_layer() {
    for (normal, tangent) in [(Axis::X, Axis::Y), (Axis::Y, Axis::Z), (Axis::Z, Axis::X)] {
        let g = owner(4, normal, 0.125);
        let op = StaticObstacleShear::new(
            &g,
            normal,
            2.0,
            0.5,
            ObstacleShearWall::NoSlip,
            1_000_000,
            |_, _| false,
        )
        .unwrap();
        let mut w = Workspace::new(&op, tangent, [Plate::Prescribed; 2], 1_000_000).unwrap();
        let mut v: Vec<_> = op
            .layer_centers()
            .iter()
            .map(|y| (y - 0.125) / 0.875)
            .collect();
        let initial = v.clone();
        let mut plates = [0.0, 1.0];
        let r = w.step(&mut v, &mut plates, 0.125, |_, _| false).unwrap();
        for (a, b) in v.iter().zip(initial) {
            close(*a, b);
        }
        let j = 0.125 * 0.5 / 0.875;
        assert!((r.fluid_impulse_n_s[0].lo + j).abs() < 1e-13);
        assert!((r.fluid_impulse_n_s[1].hi - j).abs() < 1e-13);
    }
}
#[test]
fn cancellation_and_invalid_input_preserve_all_publication() {
    let g = owner(3, Axis::Y, 0.0);
    let op = StaticObstacleShear::new(
        &g,
        Axis::Y,
        1.0,
        1.0,
        ObstacleShearWall::NoSlip,
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    let mut w = Workspace::new(&op, Axis::X, [dynamic(); 2], 1_000_000).unwrap();
    let mut v = [0.0; 3];
    let mut plates = [1.0, -0.5];
    w.step(&mut v, &mut plates, 0.125, |_, _| false).unwrap();
    let before = v;
    let wall_before = plates;
    let last = *w.last_report().unwrap();
    let mut checkpoints = Vec::new();
    w.step(&mut v, &mut plates, 0.125, |s, i| {
        checkpoints.push((s, i));
        false
    })
    .unwrap();
    let expected = (v, plates, *w.last_report().unwrap());
    for (stage, index) in checkpoints {
        let mut w = Workspace::new(&op, Axis::X, [dynamic(); 2], 1_000_000).unwrap();
        let mut v = [0.0; 3];
        let mut plates = [1.0, -0.5];
        w.step(&mut v, &mut plates, 0.125, |_, _| false).unwrap();
        assert!(matches!(
            w.step(&mut v, &mut plates, 0.125, |s, i| s == stage && i == index),
            Err(Error::Cancelled { .. })
        ));
        assert_eq!(
            (v, plates, *w.last_report().unwrap()),
            (before, wall_before, last)
        );
        w.step(&mut v, &mut plates, 0.125, |_, _| false).unwrap();
        assert_eq!((v, plates, *w.last_report().unwrap()), expected);
    }
    for dt in [
        0.0,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::from_bits(1),
        f64::MAX,
    ] {
        let b = (v, plates, *w.last_report().unwrap());
        assert!(w.step(&mut v, &mut plates, dt, |_, _| false).is_err());
        assert_eq!((v, plates, *w.last_report().unwrap()), b);
    }
    for bad in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        for target in 0..2 {
            let mut input = v;
            let mut input_walls = plates;
            if target == 0 {
                input[1] = bad;
            } else {
                input_walls[0] = bad;
            }
            let field_bits = input.map(f64::to_bits);
            let wall_bits = input_walls.map(f64::to_bits);
            let report = *w.last_report().unwrap();
            assert!(
                w.step(&mut input, &mut input_walls, 0.125, |_, _| false)
                    .is_err()
            );
            assert_eq!(input.map(f64::to_bits), field_bits);
            assert_eq!(input_walls.map(f64::to_bits), wall_bits);
            assert_eq!(*w.last_report().unwrap(), report);
        }
    }
    assert!(matches!(
        w.step(&mut [], &mut plates, 0.1, |_, _| false),
        Err(Error::ShapeMismatch)
    ));
}
#[test]
fn constructor_and_payload_admission() {
    let g = owner(1, Axis::Y, 0.0);
    let op = StaticObstacleShear::new(
        &g,
        Axis::Y,
        1.0,
        1.0,
        ObstacleShearWall::NoSlip,
        1_000_000,
        |_, _| false,
    )
    .unwrap();
    assert!(matches!(
        Workspace::new(&op, Axis::Y, [dynamic(); 2], 1_000_000),
        Err(Error::InvalidParameter)
    ));
    for mass in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            Workspace::new(
                &op,
                Axis::X,
                [Plate::Dynamic { mass_kg: mass }; 2],
                1_000_000
            )
            .is_err()
        );
    }
    let w = Workspace::new(&op, Axis::X, [dynamic(); 2], 1_000_000).unwrap();
    let cap = w.combined_bytes();
    assert!(matches!(
        Workspace::new(&op, Axis::X, [dynamic(); 2], cap - 1),
        Err(Error::BufferLimit { .. })
    ));
    Workspace::new(&op, Axis::X, [dynamic(); 2], cap).unwrap();
}
#[test]
fn transient_couette_sine_refines_against_continuum() {
    let mut errors = Vec::new();
    for n in [8_u64, 16, 32] {
        let g = owner(n, Axis::Y, 0.0);
        let op = StaticObstacleShear::new(
            &g,
            Axis::Y,
            1.0,
            1.0,
            ObstacleShearWall::NoSlip,
            1_000_000,
            |_, _| false,
        )
        .unwrap();
        let mut w = Workspace::new(&op, Axis::X, [Plate::Prescribed; 2], 1_000_000).unwrap();
        let pi = std::f64::consts::PI;
        let mut v: Vec<_> = op
            .layer_centers()
            .iter()
            .map(|y| y + 0.2 * (pi * y).sin())
            .collect();
        let mut walls = [0.0, 1.0];
        let steps = n * n;
        let dt = 0.125 / steps as f64;
        for _ in 0..steps {
            w.step(&mut v, &mut walls, dt, |_, _| false).unwrap();
        }
        let error = (v
            .iter()
            .zip(op.layer_centers())
            .map(|(v, y)| (v - (y + 0.2 * (pi * y).sin() * (-pi * pi * 0.125).exp())).powi(2))
            .sum::<f64>()
            / n as f64)
            .sqrt();
        eprintln!("Couette N={n}, dt={dt}, L2={error}");
        errors.push(error);
    }
    assert!(
        errors[0] / errors[1] > 3.0 && errors[1] / errors[2] > 3.0,
        "{errors:?}"
    );
    assert!(errors[2] < 1e-4);
}
