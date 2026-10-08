/-! Appended after AlignedStepAcceptance.lean by check_acceptance.py. -/
open Lean Elab Command
run_cmd do
  let env ← getEnv
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let expected : Array Name := #[
    `RheonExperiment.AlignedStepAcceptance.enclosed,
    `RheonExperiment.AlignedStepAcceptance.energyDelta,
    `RheonExperiment.AlignedStepAcceptance.factor_energy_difference,
    `RheonExperiment.AlignedStepAcceptance.energy_gate_nonincrease,
    `RheonExperiment.AlignedStepAcceptance.separate_energy_gates_compose,
    `RheonExperiment.AlignedStepAcceptance.relative_defect_gate_sound,
    `RheonExperiment.AlignedStepAcceptance.zero_scale_requires_zero_defect,
    `RheonExperiment.AlignedStepAcceptance.divergence_gate_sound,
    `RheonExperiment.AlignedStepAcceptance.enclosed_bound_exact_euler,
    `RheonExperiment.AlignedStepAcceptance.viscousDefect,
    `RheonExperiment.AlignedStepAcceptance.viscousScale,
    `RheonExperiment.AlignedStepAcceptance.zero_viscous_defect_is_exact_euler,
    `RheonExperiment.AlignedStepAcceptance.pressureDefect,
    `RheonExperiment.AlignedStepAcceptance.pressureScale,
    `RheonExperiment.AlignedStepAcceptance.matchedPressureResidual,
    `RheonExperiment.AlignedStepAcceptance.stored_mass_pressure_flux_identity,
    `RheonExperiment.AlignedStepAcceptance.stored_mass_pressure_energy_identity,
    `RheonExperiment.AlignedStepAcceptance.pressure_work_gate_nonincrease]
  for name in expected do
    let _ ← getConstInfo name
    pure ()
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let isProofOrAxiom := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "RheonExperiment.AlignedStepAcceptance." &&
        (isProofOrAxiom || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Expected declaration not audited: {name}"
  logInfo m!"Experimental axiom audit passed for {audited.size} declarations"
