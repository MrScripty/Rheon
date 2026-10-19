import Publication
import StokesWork
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let expected : Array Name := #[
    `Rheon.FollowOn.Publication.publish,
    `Rheon.FollowOn.Publication.rejected_preserves_state,
    `Rheon.FollowOn.Publication.accepted_is_candidate,
    `Rheon.FollowOn.Publication.checked_invariant_preserved,
    `Rheon.FollowOn.Publication.fallback,
    `Rheon.FollowOn.Publication.fallback_checked_invariant,
    `Rheon.FollowOn.Publication.both_rejected_preserve_state,
    `Rheon.FollowOn.Publication.two_checked_stages,
    `Rheon.FollowOn.Publication.checked_does_not_imply_accuracy,
    `Rheon.FollowOn.StokesWork.flux,
    `Rheon.FollowOn.StokesWork.gradient,
    `Rheon.FollowOn.StokesWork.strain,
    `Rheon.FollowOn.StokesWork.viscous,
    `Rheon.FollowOn.StokesWork.dissipation,
    `Rheon.FollowOn.StokesWork.energy,
    `Rheon.FollowOn.StokesWork.pressure_work,
    `Rheon.FollowOn.StokesWork.viscous_work,
    `Rheon.FollowOn.StokesWork.coupled_work_with_defect,
    `Rheon.FollowOn.StokesWork.exact_constrained_work,
    `Rheon.FollowOn.StokesWork.exact_constrained_energy_nonincrease]
  for name in expected do
    let _ ← getConstInfo name
    pure ()
  let mut count := 0
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.FollowOn." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      count := count + 1
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Expected declaration not audited: {name}"
  logInfo m!"Follow-on axiom audit passed for {count} declarations"
