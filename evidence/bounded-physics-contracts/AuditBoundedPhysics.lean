import Rheon.BoundedPhysics
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.BoundedPhysics.segment, `Rheon.BoundedPhysics.wallValue,
    `Rheon.BoundedPhysics.wallHit, `Rheon.BoundedPhysics.wall_segment_affine,
    `Rheon.BoundedPhysics.wall_hit_range, `Rheon.BoundedPhysics.wall_hit_on_surface,
    `Rheon.BoundedPhysics.wall_first_hit, `Rheon.BoundedPhysics.clipped_segment_in_halfspace,
    `Rheon.BoundedPhysics.strain, `Rheon.BoundedPhysics.viscousOperator,
    `Rheon.BoundedPhysics.dissipation, `Rheon.BoundedPhysics.viscous_work,
    `Rheon.BoundedPhysics.dissipation_nonnegative, `Rheon.BoundedPhysics.backward_euler_work,
    `Rheon.BoundedPhysics.backward_euler_energy_nonincrease]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.BoundedPhysics." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required bounded contract not audited: {name}"
  logInfo m!"Bounded-physics axiom audit passed for {audited.size} declarations"
