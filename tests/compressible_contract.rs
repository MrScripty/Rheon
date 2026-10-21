use rheon::{
    Axis, CompressibleCause, CompressibleError, CompressibleSimulation, CompressibleStage,
    ConservativeState, GridGeometry, IdealGas,
};

fn gas() -> IdealGas {
    IdealGas::new(1.4, 287.0).unwrap()
}
fn setup(shape: [u64; 3]) -> CompressibleSimulation {
    let grid = GridGeometry::new(shape, [0.17, 0.23, 0.31], [-1.2, 0.4, 2.3]).unwrap();
    CompressibleSimulation::new(
        grid,
        gas(),
        0.5,
        1 << 20,
        gas().conservative(1.7, [0.7, -0.4, 0.2], 2.1).unwrap(),
    )
    .unwrap()
}
fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 96.0 * f64::EPSILON * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}
fn stamp(
    s: &CompressibleSimulation,
) -> (
    Vec<[u64; 5]>,
    u64,
    u64,
    Option<rheon::CompressibleStepReport>,
    rheon::CompressibleDiagnostics,
) {
    let v = s.view();
    (
        v.conservative.iter().map(|u| u.map(f64::to_bits)).collect(),
        v.time.to_bits(),
        v.generation,
        v.last_report.cloned(),
        v.diagnostics,
    )
}

#[test]
fn si_roundtrip_and_all_axis_pressure_work_and_orientation() {
    let eos = gas();
    let rho = 1.7;
    let velocity = [13.0, -7.0, 2.0];
    let pressure = 101325.0;
    let u = eos.conservative(rho, velocity, pressure).unwrap();
    let p = eos.primitive(u).unwrap();
    close(p.density, rho);
    for d in 0..3 {
        close(p.velocity[d], velocity[d]);
        close(u[d + 1], rho * velocity[d]);
    }
    close(p.pressure, pressure);
    close(p.temperature, pressure / (rho * 287.0));
    close(p.sound_speed, (1.4 * pressure / rho).sqrt());
    close(p.internal_energy_density, pressure / 0.4);
    close(eos.cv(), 287.0 / 0.4);
    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        let f = eos.physical_flux(u, axis, true).unwrap();
        let reverse = eos.physical_flux(u, axis, false).unwrap();
        close(f[0], rho * velocity[d]);
        for j in 0..3 {
            close(
                f[j + 1],
                rho * velocity[j] * velocity[d] + if j == d { pressure } else { 0.0 },
            );
        }
        close(
            f[4],
            (pressure / 0.4 + 0.5 * rho * velocity.iter().map(|v| v * v).sum::<f64>() + pressure)
                * velocity[d],
        );
        for j in 0..5 {
            assert_eq!(reverse[j], -f[j]);
        }
    }
}

#[test]
fn uniform_asymmetric_and_degenerate_periodic_grids() {
    for shape in [[3, 4, 2], [1, 1, 1], [1, 2, 4], [2, 1, 2]] {
        let mut s = setup(shape);
        let before = s.view().conservative.to_vec();
        let totals = s.view().diagnostics.totals;
        let bytes = s.allocated_bytes();
        let r = s.step(0.01, |_| false).unwrap();
        for (a, b) in s.view().conservative.iter().zip(&before) {
            for d in 0..5 {
                close(a[d], b[d]);
            }
        }
        for (d, total) in totals.iter().enumerate() {
            close(r.diagnostics.totals[d], *total);
        }
        let p = gas().primitive(before[0]).unwrap();
        let rate = 2.0
            * (0..3)
                .map(|d| (p.velocity[d].abs() + p.sound_speed) / s.view().geometry.spacing()[d])
                .sum::<f64>();
        close(r.incident_acoustic_rate, rate);
        close(r.cfl_bound, 0.5 / rate);
        close(r.accepted_dt, 0.01f64.min(r.cfl_bound));
        assert_eq!(r.attempts, 1);
        assert_eq!(r.flux_fallbacks, 0);
        assert_eq!(bytes, s.allocated_bytes());
        assert_eq!(bytes, shape.iter().product::<u64>() as usize * 128);
    }
}

#[test]
fn nonuniform_wraps_preserve_five_budgets_every_step() {
    let mut s = setup([3, 2, 4]);
    s.initialize_with(0.0, |p, _| {
        gas()
            .conservative(
                1.0 + 0.01 * (p[0] + 2 * p[1] + 3 * p[2]) as f64,
                [0.2 * p[1] as f64, -0.13 * p[2] as f64, 0.07 * p[0] as f64],
                1.0 + 0.02 * p[2] as f64,
            )
            .unwrap()
    })
    .unwrap();
    let initial = s.view().diagnostics.totals;
    for _ in 0..12 {
        let r = s.step(0.007, |_| false).unwrap();
        for (d, total) in initial.iter().enumerate() {
            assert!((r.diagnostics.totals[d] - total).abs() / total.abs().max(1.0) <= 2e-13);
        }
        assert!(
            r.diagnostics.min_density > 0.0
                && r.diagnostics.min_pressure > 0.0
                && r.diagnostics.min_temperature > 0.0
        );
    }
}

#[test]
fn eos_and_cfl_parameters_reject_invalid_values() {
    for gamma in [0.0, -1.0, 1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            IdealGas::new(gamma, 287.0),
            Err(CompressibleError::InvalidParameter { name: "gamma" })
        );
    }
    for r in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            IdealGas::new(1.4, r),
            Err(CompressibleError::InvalidParameter {
                name: "gas_constant"
            })
        );
    }
    assert_eq!(
        IdealGas::new(1.0 + f64::EPSILON, f64::MAX),
        Err(CompressibleError::InvalidParameter { name: "cv" })
    );
    for cfl in [0.0, -1.0, 0.500001, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            CompressibleSimulation::new(
                GridGeometry::new([2; 3], [1.0; 3], [0.0; 3]).unwrap(),
                gas(),
                cfl,
                4096,
                [1.0, 0.0, 0.0, 0.0, 2.5]
            ),
            Err(CompressibleError::InvalidParameter { name: "cfl" })
        ));
    }
}

#[test]
fn invalid_initialization_and_preflight_preserve_accepted_bits_and_report() {
    let mut s = setup([2, 3, 2]);
    s.step(0.001, |_| false).unwrap();
    let before = stamp(&s);
    for (component, value, cause) in [
        (0, 0.0, CompressibleCause::NonpositiveDensity),
        (0, -1.0, CompressibleCause::NonpositiveDensity),
        (2, f64::NAN, CompressibleCause::Nonfinite),
        (4, 0.0, CompressibleCause::NonpositiveInternalEnergy),
        (4, f64::INFINITY, CompressibleCause::Nonfinite),
    ] {
        let mut state = s.view().conservative.to_vec();
        state[7][component] = value;
        let e = s.initialize(&state, 0.0).unwrap_err();
        assert!(
            matches!(e, CompressibleError::InvalidState(f) if f.stage==CompressibleStage::Initialization && f.cell==Some(7) && f.component==Some(component) && f.cause==cause),
            "{e:?}"
        );
        assert_eq!(stamp(&s), before);
    }
    assert_eq!(
        s.initialize(&[], 0.0),
        Err(CompressibleError::StateLength {
            expected: 12,
            actual: 0
        })
    );
    for time in [-1.0, f64::NAN, f64::INFINITY] {
        let state = s.view().conservative.to_vec();
        assert_eq!(
            s.initialize(&state, time),
            Err(CompressibleError::InvalidParameter { name: "time" })
        );
    }
    for dt in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            s.step(dt, |_| false),
            Err(CompressibleError::InvalidParameter {
                name: "requested_dt"
            })
        );
        assert_eq!(stamp(&s), before);
    }
}

#[test]
fn cancellation_before_during_and_at_publication_rolls_back() {
    for stage in [
        CompressibleStage::Preflight,
        CompressibleStage::Cfl,
        CompressibleStage::Candidate,
        CompressibleStage::Diagnostics,
        CompressibleStage::Publication,
    ] {
        let mut s = setup([3, 2, 4]);
        s.step(0.001, |_| false).unwrap();
        let before = stamp(&s);
        let e = s
            .step(0.001, |c| {
                c.stage == stage && (c.cell == Some(5) || c.cell.is_none())
            })
            .unwrap_err();
        assert!(matches!(e, CompressibleError::Cancelled(c) if c.stage==stage));
        assert_eq!(stamp(&s), before);
        s.step(0.001, |_| false).unwrap();
    }
}

#[test]
fn clock_nonadvance_and_capacity_fail_before_publication() {
    let mut s = setup([2; 3]);
    let state = s.view().conservative.to_vec();
    s.initialize(&state, 1e20).unwrap();
    let before = stamp(&s);
    assert!(
        matches!(s.step(0.001, |_|false), Err(CompressibleError::NonadvancingTime{time,dt}) if time==1e20 && dt>0.0)
    );
    assert_eq!(stamp(&s), before);
    let grid = GridGeometry::new([2; 3], [1.0; 3], [0.0; 3]).unwrap();
    assert!(matches!(
        CompressibleSimulation::new(grid, gas(), 0.5, 1023, [1.0, 0.0, 0.0, 0.0, 2.5]),
        Err(CompressibleError::BufferLimit {
            required: 1024,
            limit: 1023
        })
    ));
    let huge = GridGeometry::new([1 << 30, 1 << 28, 1], [1.0; 3], [0.0; 3]).unwrap();
    assert!(matches!(
        CompressibleSimulation::new(huge, gas(), 0.5, usize::MAX, [1.0, 0.0, 0.0, 0.0, 2.5]),
        Err(CompressibleError::CapacityOverflow)
    ));
}

#[test]
fn eos_rejects_overflow_underflow_and_lost_internal_energy() {
    let states: [ConservativeState; 4] = [
        [1.0, 1e308, 0.0, 0.0, 1e308],
        [1.0, 1e8, 0.0, 0.0, 5e15],
        [1.0, 0.0, 0.0, 0.0, f64::from_bits(1)],
        [1e308, 0.0, 0.0, 0.0, 1.0],
    ];
    for u in states {
        assert!(matches!(
            gas().primitive(u),
            Err(CompressibleError::InvalidState(_))
        ));
    }
}

#[test]
fn dt_independent_flux_arithmetic_is_terminal_and_preserves_state() {
    // Valid EOS and finite budgets; physical energy flux overflows. A retry
    // cannot fix this raw flux. No forced test hook bypasses the public path.
    let eos = IdealGas::new(1.4, 1.0).unwrap();
    let u = eos.conservative(1e100, [1e100, 0.0, 0.0], 1e300).unwrap();
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let mut s = CompressibleSimulation::new(grid, eos, 0.5, 4096, u).unwrap();
    let before = stamp(&s);
    let e = s.step(1e-110, |_| false).unwrap_err();
    assert!(
        matches!(e,CompressibleError::Arithmetic(cause) if cause.stage==CompressibleStage::Candidate && cause.component==Some(4)),
        "{e:?}"
    );
    assert_eq!(stamp(&s), before);
}

fn conditioned(state: &[ConservativeState; 6]) -> CompressibleSimulation {
    let eos = IdealGas::new(1.4, 1.0).unwrap();
    let grid = GridGeometry::new([3, 2, 1], [1.0, 3.0, 2.0], [0.0; 3]).unwrap();
    let mut sim = CompressibleSimulation::new(grid, eos, 0.5, 4096, state[0]).unwrap();
    sim.initialize(state, 0.0).unwrap();
    sim
}

#[test]
fn real_conditioning_failure_retries_from_same_state_and_can_accept() {
    // Independently found by reviewer: high-Mach, barely resolved internal
    // energy in unequal-density co-moving cells. This is retry evidence only.
    let state = [
        [
            2594.6168488743115,
            4072407956.611138,
            1221722386.9833415,
            -407240795.6611138,
            3515539727858377.0,
        ],
        [
            1.7708419914437827e-5,
            27.79443530934273,
            8.33833059280282,
            -2.779443530934273,
            23993775.32517533,
        ],
        [
            0.026798622168641574,
            42062.05713692962,
            12618.617141078887,
            -4206.2057136929625,
            36310417442.405716,
        ],
        [
            60.208757578003926,
            94501283.88887683,
            28350385.166663047,
            -9450128.388887683,
            81579012069662.25,
        ],
        [
            0.1774450220547531,
            278510.6864585157,
            83553.20593755471,
            -27851.06864585157,
            240426645196.12436,
        ],
        [
            76.9044476606645,
            120706178.52032854,
            36211853.55609856,
            -12070617.852032855,
            104200603305790.14,
        ],
    ];
    let mut s = conditioned(&state);
    let before = stamp(&s);
    let e = s.step(1.0, |c| c.attempt == 2).unwrap_err();
    assert!(
        matches!(e, CompressibleError::Cancelled(c) if c.stage == CompressibleStage::Candidate && c.attempt == 2)
    );
    assert_eq!(stamp(&s), before);
    let r = s.step(1.0, |_| false).unwrap();
    assert!(r.attempts > 1 && r.attempts <= 9);
    assert_eq!(r.rejections, r.attempts - 1);
    assert!(
        matches!(r.last_rejection, Some(f) if f.stage==CompressibleStage::Candidate && f.cause==CompressibleCause::NonpositiveInternalEnergy)
    );
    close(
        r.accepted_dt,
        r.cfl_bound * 0.5f64.powi(i32::from(r.rejections)),
    );
    let mut fresh = conditioned(&state);
    // The same accepted dt without failed attempts must produce identical bits.
    fresh.step(r.accepted_dt, |_| false).unwrap();
    assert_eq!(stamp(&s).0, stamp(&fresh).0);
    assert_eq!(s.view().time, fresh.view().time);
}

#[test]
fn real_candidate_exhaustion_preserves_accepted_state_and_time() {
    let state = [
        [
            2.1881052467123334e-5,
            3719.369292165712,
            1115.8107876497136,
            -371.9369292165712,
            347722732887.7291,
        ],
        [
            84.60491325467503,
            14381251395.406584,
            4314375418.621975,
            -1438125139.5406585,
            1.3444989310659308e18,
        ],
        [
            10.700930653001766,
            1818957882.7696164,
            545687364.8308849,
            -181895788.27696162,
            1.70053833411105e17,
        ],
        [
            0.009180220853414868,
            1560465.6854963202,
            468139.7056488961,
            -156046.56854963204,
            145887474492309.4,
        ],
        [
            5.951968289994049e-5,
            10117.231846598794,
            3035.1695539796383,
            -1011.7231846598794,
            945857007092.0929,
        ],
        [
            384.25287576302486,
            65315795421.02587,
            19594738626.307762,
            -6531579542.102587,
            6.10635435754494e18,
        ],
    ];
    let mut s = conditioned(&state);
    let before = stamp(&s);
    let e = s.step(1.0, |_| false).unwrap_err();
    assert!(
        matches!(e, CompressibleError::RetryExhausted { attempts: 9, last, .. } if last.stage==CompressibleStage::Candidate && last.cause==CompressibleCause::NonpositiveInternalEnergy),
        "{e:?}"
    );
    assert_eq!(stamp(&s), before);
}

#[test]
fn nonuniform_cfl_uses_both_actual_face_endpoints() {
    let eos = gas();
    let grid = GridGeometry::new([3, 2, 1], [0.4, 0.7, 1.1], [-0.2, 0.3, 1.0]).unwrap();
    let mut s = CompressibleSimulation::new(
        grid,
        eos,
        0.5,
        4096,
        eos.conservative(1.0, [0.0; 3], 1.0).unwrap(),
    )
    .unwrap();
    s.initialize_with(0.0, |p, _| {
        eos.conservative(
            1.0 + 0.1 * p[0] as f64,
            [0.3 * p[0] as f64, -0.2 * p[1] as f64, 0.17],
            1.0 + 0.2 * p[1] as f64,
        )
        .unwrap()
    })
    .unwrap();
    let v = s.view();
    let mut rates = [0.0; 6];
    // Independent cell-centered sum of the negative AND positive neighbors,
    // rather than native's unique-face paired accumulation traversal.
    for z in 0..1 {
        for y in 0..2 {
            for x in 0..3 {
                let p = [x, y, z];
                let i = v.geometry.cell_index(p).unwrap();
                let here = eos.primitive(v.conservative[i]).unwrap();
                for d in 0..3 {
                    for offset in [1, v.geometry.counts()[d] - 1] {
                        let mut q = p;
                        q[d] = (q[d] + offset) % v.geometry.counts()[d];
                        let other = eos
                            .primitive(v.conservative[v.geometry.cell_index(q).unwrap()])
                            .unwrap();
                        rates[i] += (here.velocity[d].abs() + here.sound_speed)
                            .max(other.velocity[d].abs() + other.sound_speed)
                            / v.geometry.spacing()[d];
                    }
                }
            }
        }
    }
    let control = (0..6)
        .max_by(|&a, &b| rates[a].total_cmp(&rates[b]))
        .unwrap();
    let r = s.step(1.0, |_| false).unwrap();
    close(r.incident_acoustic_rate, rates[control]);
    assert_eq!(r.controlling_cell, control);
    close(r.cfl_bound, 0.5 / rates[control]);
}

#[test]
fn nonuniform_constructor_publishes_initial_generation_zero_only_after_validation() {
    let grid = GridGeometry::new([3, 2, 1], [0.2, 0.3, 0.4], [0.25, -0.5, 1.0]).unwrap();
    let s = CompressibleSimulation::from_fn(grid.clone(), gas(), 0.5, 4096, |p, _| {
        gas()
            .conservative(1.0 + 0.1 * p[0] as f64, [0.1; 3], 1.0)
            .unwrap()
    })
    .unwrap();
    assert_eq!(s.view().generation, 0);
    assert_eq!(s.view().time, 0.0);
    assert_eq!(s.view().conservative[5][0], 1.2);
    assert!(s.view().last_report.is_none());
    let e = CompressibleSimulation::from_fn(grid, gas(), 0.5, 4096, |p, _| {
        if p == [2, 1, 0] {
            [0.0; 5]
        } else {
            [1.0, 0.0, 0.0, 0.0, 2.5]
        }
    });
    assert!(
        matches!(e, Err(CompressibleError::InvalidState(c)) if c.stage == CompressibleStage::Initialization && c.cell == Some(5) && c.cause == CompressibleCause::NonpositiveDensity)
    );
}
