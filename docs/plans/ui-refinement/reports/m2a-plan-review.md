# Independent M2a admission review

Reviewed 2026-10-20 by `ui_plan_review`, against local base
`7318023f04946171d7909119300af76e455c2926` and the current M2a admission in
`plan.md`. Read the baseline material/domain forms, material preset/SI owners,
existing egui input tests, and egui 0.33.3 disclosure/ComboBox behavior.
Only this non-authorizing review report was authored.

## Disposition

M2a is an independently useful, reversible presentation slice while M1's live
cancellation and native accessibility gaps remain parked. Its production write
set is correctly limited to `src/bin/desktop/settings.rs`; no worker, supervisor,
launch, numerical or Inspector edits are needed. Preserve M1 as unaccepted and
keep all existing commits, native evidence and branch identities intact.

No missing authority prevents this bounded implementation. Incorporate the
following concrete design conditions and focused inert checks. Native appearance,
focus visibility and the 40% word-reduction target remain separate unmet claims.

## Findings and conditions

| ID | Severity | Condition and evidence |
| --- | --- | --- |
| M2P1 | Medium | Use explicit stable disclosure IDs independent of current temperature text, distinct for fluid and domain. Egui derives the default ID from the title, so a dynamic °C title otherwise changes disclosure state when a value changes. Test edit while open, repaint, close/reopen and verify retained value plus stable disclosure state. The in-progress diff already uses explicit IDs; retain them. |
| M2P2 | Medium | Bound opened menu labels as well as selected captions. Egui 0.33.3 `ComboBox` sets popup content to `TextWrapMode::Extend` internally; wrapping only `selected_text` does not satisfy the declared all-preset/fill-choice width claim. Override wrapping inside the relevant menu closures or render bounded full option galleys. Exercise actual open menus at 240/280 logical-pixel form widths, including Slime/Syrup and the edited selected caption. |
| M2P3 | Low | Give the help copy one local owner. Property-help text should reuse the existing density, viscosity, surface-tension, wall-drag and thermal explanation content rather than creating separately maintained definitions. Shared local constants suffice; no framework/helper module is necessary. |

No numerical redesign is needed. Preserve `.changed()`-guarded conversion:
display °C to stored Kelvin by adding 273.15, density display-unit factor,
gas molar-mass g/mol to kg/mol division by 1000, existing ranges and edit speeds.
The thermal header is a readout of current configuration, not a new stored
temperature. Do not round the canonical value to the header's display precision
or write back during idle paint/disclosure toggling.

Gas molar mass and reference gas pressure must remain outside the collapsed
fluid thermal section and be calculated from the current canonical material.
Phase limitations and fixed-viscosity/thermal meaning remain available as
specified. Ambient wall temperature and wall heat-exchange editing remain
distinct from emitted-fluid temperature; preserve their existing SI owners.
Keep gravity, fill choices and initial mass semantics unchanged.

## Proportionate evidence

- Use actual egui pointer input for each disclosure and actual keyboard
  Tab/Shift+Tab plus Space/Enter activation where feasible. Direct
  `request_focus` proves focused activation only, not keyboard reachability;
  label the result accordingly and retain the native gap.
- Check values remain unchanged through idle closed/open paint, close/reopen,
  help navigation and preset-menu dismissal. A deliberate preset selection
  must still replace material values exactly as its existing owner defines.
- Perform an actual text edit after opening each thermal disclosure and verify
  independent expected SI results; a source-formula-only assertion would miss
  a disconnected edit control or changed-state error.
- Verify emitted labels include model limitations, current °C and gas reference
  pressure when thermal sections are closed. Inspect actual text/control bounds
  for the complete selected and popup labels, rather than asserting only that a
  widget or string exists.
- Reuse existing test mechanisms. Focused settings/setup tests, formatting,
  locked affected compile and strict Clippy supply the selected local evidence.
  No solver/model/evaluator/scientific or genuine ignored-cancellation test is
  needed or authorized for M2a.

The plan properly limits the earlier containment preflight to read-only
observations and does not treat unavailable cgroup/tmpfs prerequisites as
permission to provision or weaken limits. This review does not independently
verify the recorded host facts and does not authorize live cancellation.

After source checks, hand off exact-revision idle material/domain screens,
expanded thermal/help sections and dropdowns to the existing native owner at
both supported sizes/scales. Preserve reported native evidence as scoped to its
actual revision/states. No native or assistive-technology conformance result,
standards-engine certificate, publication or whole-plan acceptance follows from
this plan review.
