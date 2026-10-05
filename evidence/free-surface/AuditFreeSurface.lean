import FreeSurface
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.FreeSurface.energy, `Rheon.FreeSurface.row,
    `Rheon.FreeSurface.eliminated_pressure_quadratic,
    `Rheon.FreeSurface.anchored_energy_nonnegative,
    `Rheon.FreeSurface.two_cell_anchor_removes_constant,
    `Rheon.FreeSurface.half_distance_coefficient,
    `Rheon.FreeSurface.wet_residual_correction,
    `Rheon.FreeSurface.hydrostatic_surface_cancellation,
    `Rheon.FreeSurface.shared_surface_transfer_balance]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.FreeSurface." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required free-surface contract not audited: {name}"
  logInfo m!"Free-surface axiom audit passed for {audited.size} declarations"
