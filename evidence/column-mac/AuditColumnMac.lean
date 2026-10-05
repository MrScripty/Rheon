import ColumnMac
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.ColumnMac.two_cell_face_mass_partition,
    `Rheon.ColumnMac.constant_transfer,
    `Rheon.ColumnMac.fixed_wall_momentum_accounting,
    `Rheon.ColumnMac.two_cell_mass_adjoint,
    `Rheon.ColumnMac.lift_energy_partition,
    `Rheon.ColumnMac.restriction_energy_loss,
    `Rheon.ColumnMac.pressure_weight_positive,
    `Rheon.ColumnMac.edge_mass_metric_curvature,
    `Rheon.ColumnMac.two_node_pressure_power,
    `Rheon.ColumnMac.two_node_pressure_power_nonnegative,
    `Rheon.ColumnMac.integrated_divergence_gradient_adjoint,
    `Rheon.ColumnMac.weighted_velocity_energy_expansion,
    `Rheon.ColumnMac.projection_energy_given_residual_work]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.ColumnMac." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required column-mac contract not audited: {name}"
  logInfo m!"Column-MAC axiom audit passed for {audited.size} declarations"
