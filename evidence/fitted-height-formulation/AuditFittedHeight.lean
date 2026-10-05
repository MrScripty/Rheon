import FittedHeight
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.FittedHeight.triangle_mass_partition,
    `Rheon.FittedHeight.shared_momentum_cancel,
    `Rheon.FittedHeight.donor_pair_work,
    `Rheon.FittedHeight.donor_pair_nonnegative,
    `Rheon.FittedHeight.gcl_constant_momentum,
    `Rheon.FittedHeight.kinetic_derivative,
    `Rheon.FittedHeight.residual_energy_work,
    `Rheon.FittedHeight.pressure_adjoint,
    `Rheon.FittedHeight.full_strain_factors,
    `Rheon.FittedHeight.full_strain_nonnegative,
    `Rheon.FittedHeight.changing_mass_BE,
    `Rheon.FittedHeight.donor_step_energy,
    `Rheon.FittedHeight.force_BE_work,
    `Rheon.FittedHeight.coupled_BE_energy,
    `Rheon.FittedHeight.flat_cap_shear_numerator]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.FittedHeight." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required fitted-height identity not audited: {name}"
  logInfo m!"Fitted-height axiom audit passed for {audited.size} declarations"
