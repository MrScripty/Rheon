import FacetSelection
import Lean.Util.CollectAxioms

open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[
    `Rheon.SurfaceQueries.barycentricPoint, `Rheon.SurfaceQueries.facetCoordinates,
    `Rheon.SurfaceQueries.barycentric_weights, `Rheon.SurfaceQueries.barycentric_plane,
    `Rheon.SurfaceQueries.selectEarliest, `Rheon.SurfaceQueries.select_le_initial,
    `Rheon.SurfaceQueries.select_le_candidate, `Rheon.SurfaceQueries.selected_is_candidate,
    `Rheon.SurfaceQueries.select_in_segment]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    let logical := match info with
      | .thmInfo _ => true
      | .axiomInfo _ => true
      | _ => false
    if name.toString.startsWith "Rheon.SurfaceQueries." && (logical || expected.contains name) then
      let axioms ← Lean.collectAxioms name
      for axiomName in axioms do
        unless allowed.contains axiomName do
          throwError "Disallowed axiom {axiomName} in {name}"
      logInfo m!"AUDITED {name}: {axioms}"
      audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Required surface-query contract not audited: {name}"
  logInfo m!"Surface-query axiom audit passed for {audited.size} declarations"
