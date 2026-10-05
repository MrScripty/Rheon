import StepBudget
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.BoxFluxStep.stageWork, `Rheon.BoxFluxStep.transport,
    `Rheon.BoxFluxStep.stage_telescope, `Rheon.BoxFluxStep.full_step_budget,
    `Rheon.BoxFluxStep.conditional_energy_nonincrease, `Rheon.BoxFluxStep.transport_sum,
    `Rheon.BoxFluxStep.column_balanced_conserves,
    `Rheon.BoxFluxStep.row_normalized_preserves_constant,
    `Rheon.BoxFluxStep.normalized_two_cell_counterexample]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.BoxFluxStep." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required box-flux-step contract not audited: {name}"
  logInfo m!"Box-flux-step axiom audit passed for {audited.size} declarations"
