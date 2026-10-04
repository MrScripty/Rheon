import VisibleWeights
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.TracerBarrier.masked, `Rheon.TracerBarrier.total,
    `Rheon.TracerBarrier.normalized, `Rheon.TracerBarrier.acceptedValue,
    `Rheon.TracerBarrier.masked_nonnegative, `Rheon.TracerBarrier.masked_blocked,
    `Rheon.TracerBarrier.normalized_nonnegative, `Rheon.TracerBarrier.normalized_partition,
    `Rheon.TracerBarrier.normalized_blocked, `Rheon.TracerBarrier.visible_bounds,
    `Rheon.TracerBarrier.normalized_constant, `Rheon.TracerBarrier.accepted_bounds]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.TracerBarrier." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required tracer-barrier contract not audited: {name}"
  logInfo m!"Tracer-barrier axiom audit passed for {audited.size} declarations"
