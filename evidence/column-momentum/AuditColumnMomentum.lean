import ColumnMomentum
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.ColumnMomentum.interval_partition,
    `Rheon.ColumnMomentum.target_mass_positive,
    `Rheon.ColumnMomentum.mass_with_explicit_caps,
    `Rheon.ColumnMomentum.weighted_momentum,
    `Rheon.ColumnMomentum.constant_preservation,
    `Rheon.ColumnMomentum.mixing_identity,
    `Rheon.ColumnMomentum.mixing_nonnegative,
    `Rheon.ColumnMomentum.energy_nonincrease,
    `Rheon.ColumnMomentum.rounding_momentum,
    `Rheon.ColumnMomentum.rounding_energy_work,
    `Rheon.ColumnMomentum.closed_caps_preserve_momentum,
    `Rheon.ColumnMomentum.convex_bounds]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.ColumnMomentum." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required column-momentum contract not audited: {name}"
  logInfo m!"Column-momentum axiom audit passed for {audited.size} declarations"
