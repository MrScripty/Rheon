import AffineProjection
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.BoxFlux.fluxResidual, `Rheon.BoxFlux.momentumResidual,
    `Rheon.BoxFlux.kinetic, `Rheon.BoxFlux.correctionEnergy, `Rheon.BoxFlux.boundaryWork,
    `Rheon.BoxFlux.affine_adjoint, `Rheon.BoxFlux.momentum_work,
    `Rheon.BoxFlux.kinetic_polarization, `Rheon.BoxFlux.projection_budget,
    `Rheon.BoxFlux.compatible_flux, `Rheon.BoxFlux.gauge_boundary_work,
    `Rheon.BoxFlux.zero_flux_energy_nonincrease]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.BoxFlux." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required box-flux contract not audited: {name}"
  logInfo m!"Box-flux axiom audit passed for {audited.size} declarations"
