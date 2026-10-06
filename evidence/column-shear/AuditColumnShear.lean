import ColumnShear
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.ColumnShear.dual_length_partition,
    `Rheon.ColumnShear.cap_mass_positive,
    `Rheon.ColumnShear.row_sum_mass,
    `Rheon.ColumnShear.edge_force_cancellation,
    `Rheon.ColumnShear.three_node_force_cancellation,
    `Rheon.ColumnShear.three_node_dissipative_work,
    `Rheon.ColumnShear.dissipation_nonnegative,
    `Rheon.ColumnShear.translation_zero_force,
    `Rheon.ColumnShear.coordinate_update_preserves_momentum,
    `Rheon.ColumnShear.coordinate_update_old_work,
    `Rheon.ColumnShear.weighted_energy_identity,
    `Rheon.ColumnShear.energy_nonincrease_given_update_bound,
    `Rheon.ColumnShear.rounded_momentum_identity,
    `Rheon.ColumnShear.zero_endpoint_traction_force,
    `Rheon.ColumnShear.two_neighbor_convex_bounds]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.ColumnShear." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required column-shear contract not audited: {name}"
  logInfo m!"Column-shear axiom audit passed for {audited.size} declarations"
