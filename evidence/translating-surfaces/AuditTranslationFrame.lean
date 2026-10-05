import TranslationFrame
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.TranslationFrame.relativePoint, `Rheon.TranslationFrame.worldPoint,
    `Rheon.TranslationFrame.clock, `Rheon.TranslationFrame.wallVelocity,
    `Rheon.TranslationFrame.relative_segment, `Rheon.TranslationFrame.world_relative,
    `Rheon.TranslationFrame.reference_contact_world, `Rheon.TranslationFrame.translated_plane_value,
    `Rheon.TranslationFrame.clock_order, `Rheon.TranslationFrame.clock_range,
    `Rheon.TranslationFrame.velocity_displacement]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.TranslationFrame." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required translation-frame contract not audited: {name}"
  logInfo m!"Translation-frame axiom audit passed for {audited.size} declarations"
