import Rheon
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let expected : Array Name := #[
    `Rheon.Discrete.gradient,
    `Rheon.Discrete.incidence,
    `Rheon.Discrete.laplace,
    `Rheon.Discrete.corrected,
    `Rheon.Discrete.adjoint_identity,
    `Rheon.Discrete.pressure_energy,
    `Rheon.Discrete.pressure_nonnegative,
    `Rheon.Discrete.pressure_symmetric,
    `Rheon.Discrete.constant_gradient_zero,
    `Rheon.Discrete.constant_pressure_nullspace,
    `Rheon.Discrete.internal_flux_conservation,
    `Rheon.Discrete.conservative_update,
    `Rheon.Discrete.projection_residual,
    `Rheon.Discrete.exact_projection,
    `Rheon.Indexing.flatten,
    `Rheon.Indexing.flatten_in_bounds,
    `Rheon.Indexing.staggered_count,
    `Rheon.Transport.blend,
    `Rheon.Transport.blend_lower,
    `Rheon.Transport.blend_upper,
    `Rheon.Transport.blend_constant,
    `Rheon.Transport.convex_sum_bounds,
    `Rheon.Transport.upwind_positive,
    `Rheon.Transport.explicit_diffusion_positive,
    `Rheon.Transport.cfl_counterexample,
    `Rheon.Transport.interpolation_not_mass_conservative,
    `Rheon.Physics.force_work_identity,
    `Rheon.Physics.hydrostatic_increment,
    `Rheon.Physics.positive_transmissibility,
    `Rheon.Physics.weighted_pressure_energy,
    `Rheon.Physics.internal_amount_balance,
    `Rheon.Physics.sealed_flux_compatibility,
    `Rheon.Physics.volume_source_balance,
    `Rheon.Physics.weighted_energy_identity,
    `Rheon.Physics.implicit_energy_identity,
    `Rheon.Physics.implicit_energy_nonincrease,
    `Rheon.Physics.slip_power_nonpositive,
    `Rheon.Physics.young_adhesion_identity,
    `Rheon.Physics.adhesion_bounds,
    `Rheon.BoundedPhysics.segment,
    `Rheon.BoundedPhysics.wallValue,
    `Rheon.BoundedPhysics.wallHit,
    `Rheon.BoundedPhysics.wall_segment_affine,
    `Rheon.BoundedPhysics.wall_hit_range,
    `Rheon.BoundedPhysics.wall_hit_on_surface,
    `Rheon.BoundedPhysics.wall_first_hit,
    `Rheon.BoundedPhysics.clipped_segment_in_halfspace,
    `Rheon.BoundedPhysics.strain,
    `Rheon.BoundedPhysics.viscousOperator,
    `Rheon.BoundedPhysics.dissipation,
    `Rheon.BoundedPhysics.viscous_work,
    `Rheon.BoundedPhysics.dissipation_nonnegative,
    `Rheon.BoundedPhysics.backward_euler_work,
    `Rheon.BoundedPhysics.backward_euler_energy_nonincrease,
    `Rheon.WallFriction.wall_force_work,
    `Rheon.WallFriction.common_translation,
    `Rheon.WallFriction.relative_dissipation_nonnegative,
    `Rheon.WallFriction.explicit_work_identity,
    `Rheon.NoSlip.compatible_reaction_preserves_trace,
    `Rheon.NoSlip.relative_reaction_work_zero,
    `Rheon.NoSlip.momentum_balance,
    `Rheon.NoSlip.constrained_work_identity,
    `Rheon.ColumnForcing.units_agree,
    `Rheon.ColumnForcing.reaction_includes_body_force,
    `Rheon.ColumnForcing.forced_momentum,
    `Rheon.ColumnForcing.forced_work,
    `Rheon.ColumnForcing.parabola,
    `Rheon.ColumnForcing.parabola_endpoints,
    `Rheon.ColumnForcing.quadratic_stencil_balance,
    `Rheon.ColumnForcing.recurrence_about_equilibrium,
    `Rheon.StaticObstacle.overlap,
    `Rheon.StaticObstacle.overlap_nonnegative,
    `Rheon.StaticObstacle.overlap_symmetric,
    `Rheon.StaticObstacle.overlap_bounded,
    `Rheon.StaticObstacle.fluidVolume,
    `Rheon.StaticObstacle.fluid_volume_bounds,
    `Rheon.StaticObstacle.shared_flux_cancels,
    `Rheon.StaticObstacle.component_constant_jump_zero,
    `Rheon.StaticObstacle.residual_divergence_scale,
    `Rheon.ObstacleOperators.edgeIncidence,
    `Rheon.ObstacleOperators.edge_gradient,
    `Rheon.ObstacleOperators.edge_balanced,
    `Rheon.ObstacleOperators.outwardFlux,
    `Rheon.ObstacleOperators.pressureWeight,
    `Rheon.ObstacleOperators.faceMass,
    `Rheon.ObstacleOperators.pressureCorrected,
    `Rheon.ObstacleOperators.pressureRhs,
    `Rheon.ObstacleOperators.pressureResidual,
    `Rheon.ObstacleOperators.kineticEnergy,
    `Rheon.ObstacleOperators.kinetic_increment,
    `Rheon.ObstacleOperators.face_mass_coefficient,
    `Rheon.ObstacleOperators.pressure_corrected_flux,
    `Rheon.ObstacleOperators.pressure_flux_residual,
    `Rheon.ObstacleOperators.pressure_divergence_residual,
    `Rheon.ObstacleOperators.component_rhs_compatible,
    `Rheon.ObstacleOperators.pressure_correction_energy,
    `Rheon.ObstacleOperators.exact_pressure_energy_nonincrease,
    `Rheon.ObstacleOperators.shearConductance,
    `Rheon.ObstacleOperators.navierConductance,
    `Rheon.ObstacleOperators.navierTrace,
    `Rheon.ObstacleOperators.shear_conductance_positive,
    `Rheon.ObstacleOperators.navier_conductance_nonnegative,
    `Rheon.ObstacleOperators.navier_free_slip,
    `Rheon.ObstacleOperators.navier_series_traction,
    `Rheon.ObstacleOperators.navier_dissipation_split,
    `Rheon.ObstacleOperators.stationaryWallDiagonal,
    `Rheon.ObstacleOperators.stationary_wall_nonnegative,
    `Rheon.ObstacleOperators.stationary_wall_force,
    `Rheon.ObstacleOperators.shearOperator,
    `Rheon.ObstacleOperators.shearDissipation,
    `Rheon.ObstacleOperators.shear_work,
    `Rheon.ObstacleOperators.shear_dissipation_nonnegative,
    `Rheon.ObstacleOperators.forced_shear_energy,
    `Rheon.ObstacleOperators.forced_shear_momentum,
    `Rheon.ObstacleOperators.two_wall_shear_momentum,
    `Rheon.ObstacleOperators.unforced_shear_energy_nonincrease,
    `Rheon.ObstacleOperators.reducedShearGradient,
    `Rheon.ObstacleOperators.reduced_symmetric_strain_dissipation]
  for name in expected do
    let _ ← getConstInfo name
    pure ()
  let mut audited := 0
  let mut auditedNames : Array Name := #[]
  for (name, info) in env.constants.toList do
    let isProofOrAxiom := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon." && (isProofOrAxiom || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited + 1
      auditedNames := auditedNames.push name
  for name in expected do
    unless auditedNames.contains name do
      throwError "Expected declaration not audited: {name}"
  logInfo m!"Axiom audit passed for {audited} declarations"
