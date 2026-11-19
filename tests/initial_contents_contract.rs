use rheon::{GridGeometry, SceneObjectId, initial_contents::*};
fn grid() -> GridGeometry {
    GridGeometry::new([4; 3], [0.2, 0.3, 0.4], [-0.4, -0.6, -0.8]).unwrap()
}
fn editor(fill: ContentsMaterial) -> ContentsEditor {
    ContentsEditor::new(InitialContents::new(SceneObjectId(7), grid(), fill).unwrap())
}
fn brush(op: BrushOperation, material: ContentsMaterial) -> ContentsBrush {
    ContentsBrush {
        operation: op,
        material,
        radius: 0.01,
        axis: 2,
        layer: 1,
        depth: BrushDepth::Layer,
    }
}
fn center(e: &ContentsEditor) -> [f64; 3] {
    e.document().geometry().cell_position([1; 3]).unwrap()
}
fn dab(e: &mut ContentsEditor, b: ContentsBrush) -> bool {
    let p = center(e);
    e.begin(b).unwrap();
    e.segment(p, p).unwrap();
    e.commit().unwrap()
}
#[test]
fn operation_table_preserves_occupied_volume_and_explicit_vacuum() {
    use BrushOperation::*;
    use ContentsMaterial::*;
    for original in [Vacuum, Water, Air] {
        for selected in [Water, Air] {
            for op in [Add, Remove, Replace] {
                let mut e = editor(original);
                let index = e.document().geometry().cell_index([1; 3]).unwrap();
                let expected = match op {
                    Add if original == Vacuum => selected,
                    Remove => Vacuum,
                    Replace if original != Vacuum => selected,
                    _ => original,
                };
                let changed = dab(&mut e, brush(op, selected));
                assert_eq!(changed, expected != original);
                assert_eq!(e.document().cells()[index], expected as u16);
                assert_eq!(
                    e.document()
                        .cells()
                        .iter()
                        .filter(|&&x| x != original as u16)
                        .count(),
                    usize::from(expected != original)
                );
                if changed {
                    e.undo().unwrap();
                    assert!(e.document().cells().iter().all(|&x| x == original as u16));
                    e.redo().unwrap();
                    assert_eq!(e.document().cells()[index], expected as u16);
                }
            }
        }
    }
}
#[test]
fn stroke_is_atomic_cancelled_noop_keeps_redo_nonempty_commit_invalidates() {
    let mut e = editor(ContentsMaterial::Air);
    let b = brush(BrushOperation::Replace, ContentsMaterial::Water);
    assert!(dab(&mut e, b));
    e.undo().unwrap();
    let initial = e.document().cells().to_vec();
    let revision = e.document().revision();
    let p = center(&e);
    e.begin(b).unwrap();
    e.segment(p, p).unwrap();
    assert_eq!(e.document().cells(), initial);
    assert_ne!(e.preview_cells(), initial);
    assert!(matches!(e.freeze(), Err(ContentsError::Busy)));
    assert!(!e.undo().unwrap()); // first Undo cancels; it does not consume history
    assert_eq!(e.document().revision(), revision);
    assert!(e.redo_label().is_some());
    assert!(!dab(
        &mut e,
        brush(BrushOperation::Add, ContentsMaterial::Water)
    ));
    assert!(e.redo_label().is_some());
    e.begin(b).unwrap();
    e.segment(p, p).unwrap();
    assert!(matches!(
        e.segment(p, [f64::NAN; 3]),
        Err(ContentsError::InvalidPoint)
    ));
    assert!(!e.active());
    assert_eq!(e.document().cells(), initial);
    assert!(e.redo_label().is_some());
    assert!(dab(&mut e, b));
    assert!(e.redo_label().is_none());
    let committed = e.document().revision();
    e.undo().unwrap();
    e.redo().unwrap();
    assert_eq!(e.document().revision(), committed + 2);
}
#[test]
fn fast_sweeps_match_independent_axis_capsule_and_partitioned_input() {
    for depth in [BrushDepth::Layer, BrushDepth::Volume] {
        let mut direct = editor(ContentsMaterial::Vacuum);
        let mut split = editor(ContentsMaterial::Vacuum);
        let mut b = brush(BrushOperation::Add, ContentsMaterial::Water);
        b.radius = 0.36;
        b.depth = depth;
        let g = grid();
        let start = g.cell_position([0, 1, 1]).unwrap();
        let end = g.cell_position([3, 1, 1]).unwrap();
        direct.begin(b).unwrap();
        direct.segment(start, end).unwrap();
        direct.commit().unwrap();
        split.begin(b).unwrap();
        let mut previous = start;
        for step in 1..=17 {
            let next = std::array::from_fn(|d| start[d] + (end[d] - start[d]) * step as f64 / 17.0);
            split.segment(previous, next).unwrap();
            previous = next;
        }
        split.commit().unwrap();
        assert_eq!(direct.document().cells(), split.document().cells());
        for k in 0..4 {
            for j in 0..4 {
                for i in 0..4 {
                    let p = g.cell_position([i, j, k]).unwrap();
                    // Independent axis-aligned capsule: all cell x centers are between
                    // endpoints, so its distance is the yz radial distance exactly.
                    let inside = (p[1] - start[1]).hypot(p[2] - start[2]) <= b.radius
                        && (depth == BrushDepth::Volume || k == 1);
                    assert_eq!(
                        direct.document().cells()[g.cell_index([i, j, k]).unwrap()],
                        if inside { 1 } else { 0 }
                    );
                }
            }
        }
    }
}
#[test]
fn freeze_is_exact_owned_and_vacuum_is_actionable_not_ambient_gas() {
    let mut e = editor(ContentsMaterial::Vacuum);
    assert!(matches!(
        e.freeze(),
        Err(ContentsError::Vacuum {
            count: 64,
            first: [0, 0, 0]
        })
    ));
    assert!(e.fill_air().unwrap());
    assert_eq!(e.document().counts(), [0, 0, 64]);
    let air = e.freeze().unwrap();
    assert!(air.liquid_fraction().iter().all(|&f| f == 0.0));
    assert!(air.materials().gas().density() * air.geometry().cell_volume() > 0.0);
    assert!(dab(
        &mut e,
        brush(BrushOperation::Replace, ContentsMaterial::Water)
    ));
    let frozen = e.freeze().unwrap();
    assert_eq!(
        frozen
            .liquid_fraction()
            .iter()
            .filter(|&&f| f == 1.0)
            .count(),
        1
    );
    dab(&mut e, brush(BrushOperation::Remove, ContentsMaterial::Air));
    assert!(matches!(
        e.freeze(),
        Err(ContentsError::Vacuum {
            count: 1,
            first: [1, 1, 1]
        })
    ));
    assert_eq!(frozen.cells().iter().filter(|&&x| x == 1).count(), 1);
    assert!(air.cells().iter().all(|&x| x == 2));
    // Replace deliberately does not fill a vacuum hole. Add does.
    assert!(!dab(
        &mut e,
        brush(BrushOperation::Replace, ContentsMaterial::Air)
    ));
    assert!(dab(
        &mut e,
        brush(BrushOperation::Add, ContentsMaterial::Air)
    ));
    e.freeze().unwrap();
}
#[test]
fn history_and_candidate_admission_reject_before_mutation_and_evict_whole_commands() {
    let mut baseline = editor(ContentsMaterial::Vacuum);
    baseline.fill_air().unwrap();
    let one = baseline.history_bytes();
    let mut e = ContentsEditor::with_limits(
        InitialContents::new(SceneObjectId(7), grid(), ContentsMaterial::Vacuum).unwrap(),
        one,
        CANDIDATE_LIMIT,
    );
    e.fill_air().unwrap();
    dab(
        &mut e,
        brush(BrushOperation::Replace, ContentsMaterial::Water),
    );
    assert!(e.history_bytes() <= one);
    assert_eq!(e.evicted_commands(), 1);
    e.undo().unwrap();
    assert!(e.document().cells().iter().all(|&x| x == 2));
    assert!(!e.undo().unwrap());
    let mut limited = ContentsEditor::with_limits(
        InitialContents::new(SceneObjectId(7), grid(), ContentsMaterial::Air).unwrap(),
        0,
        CANDIDATE_LIMIT,
    );
    let before = limited.document().cells().to_vec();
    let p = center(&limited);
    limited
        .begin(brush(BrushOperation::Replace, ContentsMaterial::Water))
        .unwrap();
    limited.segment(p, p).unwrap();
    assert!(matches!(limited.commit(), Err(ContentsError::Limit { .. })));
    assert_eq!(limited.document().cells(), before);
    assert_eq!(limited.document().revision(), 0);
    assert!(!limited.active());
    let mut tiny = ContentsEditor::with_limits(
        InitialContents::new(SceneObjectId(7), grid(), ContentsMaterial::Air).unwrap(),
        HISTORY_LIMIT,
        1,
    );
    assert!(matches!(
        tiny.begin(brush(BrushOperation::Remove, ContentsMaterial::Water)),
        Err(ContentsError::Limit { .. })
    ));
    assert_eq!(tiny.document().revision(), 0);
    assert!(!tiny.active());
}
#[test]
fn picking_is_checked_half_open_and_works_from_inside_domain() {
    let g = grid();
    let b = brush(BrushOperation::Add, ContentsMaterial::Water);
    let plane = b.slice_position(&g).unwrap();
    let hit = b
        .pick(&g, [-0.1, -0.15, -0.79], [0.0, 0.0, 1.0], 0.015)
        .unwrap();
    assert_eq!(hit, [-0.1, -0.15, plane]);
    assert!(
        b.pick(&g, [-0.1, -0.15, -0.79], [1.0, 0.0, 0.0], 0.015)
            .is_err()
    );
    assert!(
        b.pick(&g, [g.upper()[0], -0.15, -0.79], [0.0, 0.0, 1.0], 0.015)
            .is_err()
    );
    assert!(
        b.pick(&g, [g.origin()[0], -0.15, -0.79], [0.0, 0.0, 1.0], 0.015)
            .is_ok()
    );
    assert!(
        b.pick(&g, [-0.1, -0.15, plane], [0.0, 0.0, 1.0], 0.015)
            .is_err()
    );
    for radius in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(ContentsBrush { radius, ..b }.validate(&g).is_err());
    }
    assert!(ContentsBrush { axis: 3, ..b }.slice_position(&g).is_err());
    assert!(ContentsMaterial::from_id(3).is_err());
}

#[test]
fn dense_command_evicts_small_commands_and_their_metadata_jointly() {
    let mut baseline = editor(ContentsMaterial::Vacuum);
    baseline.fill_air().unwrap();
    let cap = baseline.history_bytes();
    let mut e = ContentsEditor::with_limits(
        InitialContents::new(SceneObjectId(7), grid(), ContentsMaterial::Vacuum).unwrap(),
        cap,
        CANDIDATE_LIMIT,
    );
    let b = brush(BrushOperation::Add, ContentsMaterial::Water);
    for i in 0..3 {
        let p = e.document().geometry().cell_position([i, 1, 1]).unwrap();
        e.begin(b).unwrap();
        e.segment(p, p).unwrap();
        assert!(e.commit().unwrap());
    }
    let before = e.document().cells().to_vec();
    assert!(e.fill_air().unwrap());
    assert!(e.history_bytes() <= cap);
    assert_eq!(e.evicted_commands(), 3);
    assert!(e.document().cells().iter().all(|&x| x == 2));
    e.undo().unwrap();
    assert_eq!(e.document().cells(), before);
    assert!(!e.undo().unwrap());
    e.redo().unwrap();
    assert!(e.document().cells().iter().all(|&x| x == 2));
}
#[test]
fn ray_direction_scale_does_not_change_hit_or_world_near_distance() {
    let g = grid();
    let b = brush(BrushOperation::Add, ContentsMaterial::Water);
    let origin = [-0.1, -0.15, -0.79];
    let expected = b.pick(&g, origin, [0.0, 0.0, 1.0], 0.015).unwrap();
    for scale in [1e-200, 1.0, 1e200] {
        assert_eq!(
            b.pick(&g, origin, [0.0, 0.0, scale], 0.015).unwrap(),
            expected
        );
        let mut close = expected;
        close[2] -= 0.01;
        assert!(b.pick(&g, close, [0.0, 0.0, scale], 0.015).is_err());
    }
    assert!(b.pick(&g, origin, [0.0; 3], 0.0).is_err());
}
