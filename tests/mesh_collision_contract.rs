use rheon::{
    CollisionMesh, DomainFill, GridGeometry, MeshCollisionError, MeshPolicy, ParticleFluidConfig,
    ParticleFluidError, ParticleFluidSimulation,
};
use std::sync::Arc;

fn cube_data(low: f64, high: f64) -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
    let vertices = vec![
        [low, low, low],
        [high, low, low],
        [high, high, low],
        [low, high, low],
        [low, low, high],
        [high, low, high],
        [high, high, high],
        [low, high, high],
    ];
    let triangles = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [3, 7, 6],
        [3, 6, 2],
        [0, 4, 7],
        [0, 7, 3],
        [1, 2, 6],
        [1, 6, 5],
    ];
    (vertices, triangles)
}
fn cube(low: f64, high: f64) -> Arc<CollisionMesh> {
    let (v, t) = cube_data(low, high);
    Arc::new(CollisionMesh::new(v, t, MeshPolicy::ConvexSolid).unwrap())
}
fn plane() -> CollisionMesh {
    CollisionMesh::new(
        vec![[0.5, 0., 0.], [0.5, 1., 0.], [0.5, 1., 1.], [0.5, 0., 1.]],
        vec![[0, 1, 2], [0, 2, 3]],
        MeshPolicy::TwoSidedSurface,
    )
    .unwrap()
}
#[test]
fn swept_plane_prevents_tunneling_in_both_directions_and_preserves_tangent_velocity() {
    let mesh = plane();
    for direction in [-1.0, 1.0] {
        let start = [0.5 - direction * 10.0, 0.4, 0.5];
        let end = [0.5 + direction * 10.0, 0.6, 0.5];
        let (p, v) = mesh
            .resolve_segment(start, end, [direction * 1000.0, 10.0, 0.0])
            .unwrap();
        assert!(direction * (p[0] - 0.5) < 0.0);
        assert!((p[0] - 0.5).abs() < 1e-8);
        assert_eq!(v, [0.0, 10.0, 0.0]);
        let (next, _) = mesh
            .resolve_segment(p, [p[0], p[1] + 0.01, p[2]], v)
            .unwrap();
        assert!((next[1] - p[1] - 0.01).abs() < 1e-12);
    }
}
#[test]
fn triangle_edges_coplanar_entry_and_corners_are_conservative() {
    let mesh = plane();
    let (p, _) = mesh
        .resolve_segment([0., 0., 0.], [1., 0., 0.], [1., 0., 0.])
        .unwrap();
    assert!(p[0] < 0.5);
    let (p, _) = mesh
        .resolve_segment([0.5, -1., 0.5], [0.5, 0.5, 0.5], [0., 1., 0.])
        .unwrap();
    assert!(p[1] <= 0.0);
    let mesh = cube(0.25, 0.75);
    let (p, v) = mesh
        .resolve_segment([0.1, 0.1, 0.5], [0.5, 0.5, 0.5], [1., 1., 0.])
        .unwrap();
    assert!(p[0] < 0.25 && p[1] < 0.25);
    assert_eq!(v, [0.; 3]);
}
#[test]
fn solid_admission_rejects_open_winding_concavity_and_unused_vertices() {
    let (v, mut t) = cube_data(0.25, 0.75);
    t.pop();
    assert_eq!(
        CollisionMesh::new(v, t, MeshPolicy::ConvexSolid).unwrap_err(),
        MeshCollisionError::NotWatertightOrConsistentlyWound
    );
    let (v, mut t) = cube_data(0.25, 0.75);
    t[0].swap(0, 1);
    assert_eq!(
        CollisionMesh::new(v, t, MeshPolicy::ConvexSolid).unwrap_err(),
        MeshCollisionError::NotWatertightOrConsistentlyWound
    );
    let (mut v, t) = cube_data(0.25, 0.75);
    v[6] = [0.4, 0.4, 0.4];
    assert_eq!(
        CollisionMesh::new(v, t, MeshPolicy::ConvexSolid).unwrap_err(),
        MeshCollisionError::NotConvex
    );
    let (mut v, t) = cube_data(0.25, 0.75);
    v.push([0.5; 3]);
    assert_eq!(
        CollisionMesh::new(v, t, MeshPolicy::ConvexSolid).unwrap_err(),
        MeshCollisionError::DisconnectedOrUnusedVertex
    );
}
#[test]
fn globally_reversed_winding_is_admitted_but_surface_policy_never_infers_a_solid() {
    let (v, mut t) = cube_data(0.25, 0.75);
    for face in &mut t {
        face.swap(0, 1);
    }
    let mesh = CollisionMesh::new(v.clone(), t.clone(), MeshPolicy::ConvexSolid).unwrap();
    assert!(mesh.contains_or_touches([0.5; 3]));
    assert!(!mesh.contains_or_touches([0.1; 3]));
    assert_eq!(
        mesh.resolve_segment([0.5; 3], [0.9; 3], [1.; 3])
            .unwrap_err(),
        MeshCollisionError::QueryStartsInsideSolid
    );
    let mesh = CollisionMesh::new(v, t, MeshPolicy::TwoSidedSurface).unwrap();
    assert!(!mesh.contains_or_touches([0.5; 3]));
}
#[test]
fn rejects_invalid_numeric_index_and_resource_data_at_constructor() {
    let (mut v, t) = cube_data(0.25, 0.75);
    v[0][0] = f64::NAN;
    assert_eq!(
        CollisionMesh::new(v, t, MeshPolicy::TwoSidedSurface).unwrap_err(),
        MeshCollisionError::NonFiniteOrExtremeCoordinate
    );
    let (v, mut t) = cube_data(0.25, 0.75);
    t[0][0] = 100;
    assert_eq!(
        CollisionMesh::new(v, t, MeshPolicy::TwoSidedSurface).unwrap_err(),
        MeshCollisionError::InvalidIndex
    );
    let (v, mut t) = cube_data(0.25, 0.75);
    t[0][0] = t[0][1];
    assert_eq!(
        CollisionMesh::new(v, t, MeshPolicy::TwoSidedSurface).unwrap_err(),
        MeshCollisionError::DegenerateTriangle
    );
    let (v, _) = cube_data(0.25, 0.75);
    assert_eq!(
        CollisionMesh::new(v, vec![[0, 1, 2]; 100_001], MeshPolicy::TwoSidedSurface).unwrap_err(),
        MeshCollisionError::ResourceLimit
    );
    assert_eq!(
        plane()
            .resolve_segment([f64::NAN; 3], [0.; 3], [0.; 3])
            .unwrap_err(),
        MeshCollisionError::NonFiniteQuery
    );
}
#[test]
fn initial_exclusion_repeat_remove_reset_and_failed_replace_are_transactional() {
    let grid = GridGeometry::new([16; 3], [1. / 16.; 3], [0.; 3]).unwrap();
    let config = ParticleFluidConfig {
        fill: DomainFill::EmittedFluid,
        samples_per_axis: 8,
        max_particles: 512,
        gravity: [0.; 3],
        ..ParticleFluidConfig::default()
    };
    let mut sim = ParticleFluidSimulation::new(grid.clone(), config).unwrap();
    let mesh = cube(0.25, 0.75);
    let initial_bytes = sim.allocated_bytes();
    assert_eq!(sim.particles().len(), 512);
    sim.set_collision_mesh(Some(mesh.clone())).unwrap();
    assert_eq!(sim.particles().len(), 448);
    assert_eq!(
        sim.allocated_bytes(),
        initial_bytes + mesh.allocated_bytes()
    );
    sim.set_collision_mesh(Some(mesh.clone())).unwrap();
    assert_eq!(sim.particles().len(), 448);
    let before = sim.particles().to_vec();
    assert_eq!(
        sim.set_collision_mesh(Some(cube(-0.1, 0.5))).unwrap_err(),
        ParticleFluidError::CollisionMeshOutsideDomain
    );
    assert_eq!(sim.particles(), before);
    assert!(Arc::ptr_eq(sim.collision_mesh().unwrap(), &mesh));
    sim.set_collision_mesh(None).unwrap();
    assert_eq!(sim.particles().len(), 448);
    assert_eq!(sim.allocated_bytes(), initial_bytes);
    let mut reset = ParticleFluidSimulation::new(grid, config).unwrap();
    reset.set_collision_mesh(Some(mesh)).unwrap();
    assert_eq!(reset.particles(), sim.particles());
}
#[test]
fn balanced_bvh_matches_analytic_planes_for_many_distributed_queries() {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for i in 0..1024 {
        let x = i as f64 * 2.0;
        let base = vertices.len();
        vertices.extend([[x, 0., 0.], [x, 1., 0.], [x, 0., 1.]]);
        triangles.push([base, base + 1, base + 2]);
    }
    let mesh = CollisionMesh::new(vertices, triangles, MeshPolicy::TwoSidedSurface).unwrap();
    for i in 0..1024 {
        let x = i as f64 * 2.0;
        let (p, v) = mesh
            .resolve_segment([x - 0.5, 0.2, 0.3], [x + 0.5, 0.2, 0.3], [1., 0., 0.])
            .unwrap();
        assert!(p[0] < x && x - p[0] < 1e-5);
        assert_eq!(v, [0.; 3]);
    }
}

#[test]
fn grazing_solid_contact_retreats_by_normal_clearance_and_can_resume() {
    let mesh = cube(0.25, 0.75);
    let (p, v) = mesh
        .resolve_segment([0.24, 0.30, 0.50], [0.26, 0.70, 0.50], [0.02, 0.40, 0.0])
        .unwrap();
    assert!(p[0] < 0.25);
    assert!(!mesh.contains_or_touches(p));
    assert_eq!(v, [0.0, 0.4, 0.0]);
    let (next, _) = mesh
        .resolve_segment(p, [p[0], p[1] + 0.01, p[2]], v)
        .unwrap();
    assert!(!mesh.contains_or_touches(next));
    assert!(next[1] > p[1]);
}

#[test]
fn oblique_plane_removes_only_normal_velocity() {
    let mesh = CollisionMesh::new(
        vec![[0., 0., 0.], [1., 1., 0.], [0., 0., 1.]],
        vec![[0, 1, 2]],
        MeshPolicy::TwoSidedSurface,
    )
    .unwrap();
    let (p, v) = mesh
        .resolve_segment([0.4, 0.6, 0.4], [0.6, 0.4, 0.4], [1., -1., 2.])
        .unwrap();
    assert!(p[0] < p[1]);
    assert!(v[0].abs() < 1e-14 && v[1].abs() < 1e-14);
    assert_eq!(v[2], 2.0);
}

#[test]
fn failed_memory_admission_preserves_initial_particles_and_selected_mesh() {
    let grid = GridGeometry::new([16; 3], [1. / 16.; 3], [0.; 3]).unwrap();
    let mut config = ParticleFluidConfig {
        fill: DomainFill::EmittedFluid,
        samples_per_axis: 4,
        max_particles: 64,
        ..ParticleFluidConfig::default()
    };
    let baseline = ParticleFluidSimulation::new(grid.clone(), config).unwrap();
    config.memory_limit = baseline.allocated_bytes();
    let mut sim = ParticleFluidSimulation::new(grid, config).unwrap();
    let before = sim.particles().to_vec();
    assert!(matches!(
        sim.set_collision_mesh(Some(cube(0.25, 0.75))),
        Err(ParticleFluidError::BufferLimit { .. })
    ));
    assert_eq!(sim.particles(), before);
    assert!(sim.collision_mesh().is_none());
}

#[test]
fn approaching_contact_band_stops_before_excluded_tolerance() {
    let mesh = cube(0.25, 0.75);
    let (p, v) = mesh
        .resolve_segment([0.24, 0.5, 0.5], [0.25 - 1e-11, 0.5, 0.5], [1., 0., 0.])
        .unwrap();
    assert!(!mesh.contains_or_touches(p));
    assert!(p[0] < 0.25 - 1e-11);
    assert_eq!(v, [0.; 3]);
}

#[test]
fn oblique_triangle_edge_entry_does_not_tunnel_past_contact_band() {
    let mesh = CollisionMesh::new(
        vec![[0.5, 0., 0.], [0.5, 1., 0.], [0.5, 0., 1.]],
        vec![[0, 1, 2]],
        MeshPolicy::TwoSidedSurface,
    )
    .unwrap();
    let (p, v) = mesh
        .resolve_segment([0.499, -0.1, 0.5], [0.501, 0.1, 0.5], [0.002, 0.2, 0.])
        .unwrap();
    assert!(p[0] < 0.5);
    assert_eq!(v, [0.0, 0.2, 0.0]);
}

#[test]
fn numerically_unresolvable_long_sweep_is_rejected_instead_of_crossing() {
    let mesh = CollisionMesh::new(
        vec![[-0.5, 0., 0.], [-0.5, 1., 0.], [-0.5, 0., 1.]],
        vec![[0, 1, 2]],
        MeshPolicy::TwoSidedSurface,
    )
    .unwrap();
    assert_eq!(
        mesh.resolve_segment([-1e100, 0.2, 0.3], [1e100, 0.2, 0.3], [1., 0., 0.])
            .unwrap_err(),
        MeshCollisionError::QueryPrecisionUnsupported
    );
}
