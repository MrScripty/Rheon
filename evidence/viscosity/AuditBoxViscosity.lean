import BoxViscosity
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.BoxViscosity.mac_patch_work,
    `Rheon.BoxViscosity.mac_patch_dissipation_nonnegative,
    `Rheon.BoxViscosity.mac_patch_symmetry,
    `Rheon.BoxViscosity.rigid_rotation_local_shear,
    `Rheon.BoxViscosity.affine_shear_local_dissipation,
    `Rheon.BoxViscosity.explicit_coordinate_energy,
    `Rheon.BoxViscosity.rounded_coordinate_energy,
    `Rheon.BoxViscosity.explicit_energy_nonincrease,
    `Rheon.BoxViscosity.modal_factor_nonexpansive,
    `Rheon.BoxViscosity.modal_step]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.BoxViscosity." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required box-viscosity contract not audited: {name}"
  logInfo m!"Box-viscosity axiom audit passed for {audited.size} declarations"
