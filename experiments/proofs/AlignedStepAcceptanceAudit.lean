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
    let userName := Lean.privateToUserName name
    if userName.toString.startsWith "RheonExperiment.AlignedStepAcceptance." then
      -- Audit logical opaque bodies and unused safe definitions too. Compiler
      -- code-generation specializations are unsafe definitions, not proof terms.
      -- Use kernel safety metadata rather than special-name exclusions.
      let isLogical ← match info with
        | .thmInfo _ => pure true
        | .axiomInfo _ => pure true
        | .opaqueInfo _ => pure true
        | .defnInfo value =>
            if value.safety == .safe then pure true
            else if value.type.getUsedConstants.all (env.contains ·) then
              liftTermElabM (Lean.Meta.isProp value.type)
            else do
              -- Erased runtime types of unsafe compiler stages contain constants
              -- absent from the logical environment. They are not kernel types.
              -- The source policy separately forbids user unsafe/partial forms.
              logInfo m!"EXCLUDED unsafe runtime artifact {name}"
              pure false
        | _ => pure false
      if isLogical || expected.contains name then
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
