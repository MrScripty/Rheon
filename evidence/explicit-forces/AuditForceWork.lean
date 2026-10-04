import ForceWork
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.ExplicitForces.kineticEnergy,
    `Rheon.ExplicitForces.storedWork,
    `Rheon.ExplicitForces.stored_work_energy_change,
    `Rheon.ExplicitForces.acceleration_work,
    `Rheon.ExplicitForces.force_density_increment,
    `Rheon.ExplicitForces.hydrostatic_acceleration_balance]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.ExplicitForces." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Expected force declaration not audited: {name}"
  logInfo m!"Explicit-force axiom audit passed for {audited.size} declarations"
