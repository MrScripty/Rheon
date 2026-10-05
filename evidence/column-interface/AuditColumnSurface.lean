import ColumnSurface
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.ColumnSurface.ghost_interpolation_zero,
    `Rheon.ColumnSurface.ghost_gradient_reduction,
    `Rheon.ColumnSurface.height_crossing_zero,
    `Rheon.ColumnSurface.variable_distance_positive,
    `Rheon.ColumnSurface.column_reconstruction_volume,
    `Rheon.ColumnSurface.column_update_cancels_internal_axial_transfers,
    `Rheon.ColumnSurface.shared_column_transfer_balance,
    `Rheon.ColumnSurface.upward_swept_slab_bound,
    `Rheon.ColumnSurface.downward_swept_slab_bound,
    `Rheon.ColumnSurface.transverse_donor_convex_bound]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.ColumnSurface." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required column-surface contract not audited: {name}"
  logInfo m!"Column-surface axiom audit passed for {audited.size} declarations"
