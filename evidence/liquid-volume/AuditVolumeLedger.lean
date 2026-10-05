import VolumeLedger
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.LiquidVolume.amountUpdate, `Rheon.LiquidVolume.massTotal,
    `Rheon.LiquidVolume.donorUpdate, `Rheon.LiquidVolume.shared_face_balance,
    `Rheon.LiquidVolume.closed_source_balance, `Rheon.LiquidVolume.constant_density_mass_balance,
    `Rheon.LiquidVolume.donor_nonnegative, `Rheon.LiquidVolume.donor_unit_interval,
    `Rheon.LiquidVolume.donor_preserves_constant,
    `Rheon.LiquidVolume.outward_courant_counterexample,
    `Rheon.LiquidVolume.post_clamp_changes_source_balance]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.LiquidVolume." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required liquid-volume contract not audited: {name}"
  logInfo m!"Liquid-volume axiom audit passed for {audited.size} declarations"
