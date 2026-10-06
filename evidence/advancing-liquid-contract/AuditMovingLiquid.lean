import MovingLiquid
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.MovingLiquid.sealed_flat_height_fixed,
    `Rheon.MovingLiquid.sealed_flat_rate_zero,
    `Rheon.MovingLiquid.unequal_rates_leave_flat,
    `Rheon.MovingLiquid.balanced_exchange_volume,
    `Rheon.MovingLiquid.atmospheric_normal_traction_requires_zero_strain,
    `Rheon.MovingLiquid.omitted_nonzero_normal_energy,
    `Rheon.MovingLiquid.normal_top_mass_basis_difference]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.MovingLiquid." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required moving-liquid obstruction not audited: {name}"
  logInfo m!"Moving-liquid axiom audit passed for {audited.size} declarations"
