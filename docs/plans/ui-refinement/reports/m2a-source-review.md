# Independent M2a source review

Reviewed 2026-10-20 against clean prerequisite
`7318023f04946171d7909119300af76e455c2926`, the M2a admission in `plan.md`
and `m2a-plan-review.md`. Reviewed settings SHA256:
`0937b5e293fdd024641f46382900debac0b6f8c263d2a3c5d61fc0f8191dc102`.
Only this report was authored by the reviewer. No runtime tests, numerical,
model or native workloads were executed by this reviewer.

## Disposition

No remaining newly introduced material source issue in the bounded M2a
presentation diff. Production changes remain confined to
`src/bin/desktop/settings.rs`; this finding does not accept native appearance,
general keyboard value preservation, accessibility, whole M1/M2 or the word-count
target. The verification incident below also prevents describing the overall
continuation as having stayed within its no-numerical-execution instruction.

## Invariants and design conditions

- `preset_text` and the material/fill/ambient ComboBoxes bound complete selected
  captions using existing button typography, and wrap opened option labels.
  Existing selector salts remain `material-preset`, `density-unit`,
  `domain-fill` and `ambient-material`. The density selector's minimum changes
  from the default 100 px to 80 px after a real narrow-form failure; complete
  unit labels remain available. No icon substitution or font reduction.
- `fluid` retains state, density, viscosity, liquid-only surface tension, wall
  drag, gas molar mass/reference pressure and the phase/model limitation outside
  optional disclosures. Gas reference pressure still reads canonical density,
  gas constant and temperature. The thermal title reports current °C without
  writing its rounded display back into configuration.
- Explicit `fluid-temperature`, `domain-thermal-environment` and
  `material-property-help` salts separate persistent disclosure state from
  changing captions. Egui 0.33.3 headers default closed. Editing temperature
  therefore does not replace the disclosure's identity or close it.
- Existing `.changed()` guards, temperature ranges/speeds and °C-to-Kelvin
  conversion remain intact. Density display-unit factors, g/mol-to-kg/mol
  conversion, gravity, volume/initial mass, fill semantics and defaults remain
  unchanged. The shared `quantity` helper and numerical/worker/recording owners
  are untouched.
- The plan review's copy-ownership condition is resolved: local constants share
  density, viscosity, tension, wall-drag and wall-thermal meanings between hover
  and visible help. Fixed-viscosity/thermal meaning remains in the fluid thermal
  disclosure; the phase/no-boiling/no-freezing/no-elastic-slime limitation stays
  directly visible.

## Focused evidence

Inspected the final focused logs: five `settings::tests::` tests and nine
`setup_ui_tests::ui_setup_` tests pass. Final compile, strict Clippy and formatting
logs show success; reviewer `git diff --check` passes. The reviewer did not rerun
runtime tests or broaden their selection.

The settings tests use actual egui pointer events to toggle both thermal
disclosures, edit 20 °C to 37.25 °C, check the independent expected 310.4 K,
retain open state across the dynamic-title change, close/reopen without
value loss, and edit again to 38.5 °C/311.65 K after reopening. Actual
material/ambient preset selection resets only the existing
material owner; fill selection and all three density-unit display choices retain
the expected other settings. All presets, edited captions and opened menus are
checked at 240/280 logical-pixel form widths, with explicit assertions that
non-selected popup options actually appear. The help test verifies essential
model/gas-pressure text with optional sections closed, then actual keyboard
activation/closure and exit through the limited route described below.

These are isolated egui layout/input results, not rendered native screens,
physical-scale measurements or proof of the application's complete focus order.
Current-revision idle native validation at both supported sizes/scales remains
pending, as do representative usability and matched-state word measurements.

## Open preexisting numeric-navigation defect: UI-5

The original forward-Tab test exposed canonical Air density changing from
`1.2041288413018594` to `1.2041300000000001` kg/m³ and molar mass from
`0.02896546` to `0.028965499999999998` kg/mol without typing. The exact failure is
retained in `/tmp/rheon-ui-m2a-tests-4.log`.

The prerequisite already uses the same eight-decimal DragValue formatting and
SI conversions. Pinned egui `drag_value.rs` reparses cached formatted text on
lost focus. This is an actual existing numeric-editor defect, not deliberate
editing or a harmless test artifact. It is tracked as UI-5; fixing the shared
editor is outside this slice and needs a separate bounded admission/regression.

The passing help test uses actual Tab to the first selector, Shift+Tab around
the isolated form to its final help disclosure, Enter to open, Space to close
and Tab away. Exact configuration equality is retained without normalizing Air
or weakening tolerances. This proves that particular route only. It does not
repair forward traversal or establish an equivalent shortcut in the full app.
Preserve/report UI-5 in native validation; general keyboard value immutability
and accessibility remain unaccepted.

## Verification-process incident

The lead disclosed that an earlier `ui_` substring filter selected numerical
Inspector/archive/engineering fixtures and a configured comparison fixture with
a Python analytic pressure-adapter runtime. Its retained log records 19 passed,
one ignored; the genuine cancellation test remained ignored. This execution
violated the no-numerical-execution instruction. It supplies no M2a acceptance
credit and is not authorization for any repetition or broader workload.

`m2a-material-domain.md` records the exact affected tests, bounds, disclosure and
artifact limits. Some existing tests removed their own disposable outputs before
detection; those bytes cannot be represented as preserved. The broad log remains
at `/tmp/rheon-ui-m2a-all-ui-tests.log`. No reviewer runtime execution, rerun or
cleanup followed. Subsequent successful focused logs are
`/tmp/rheon-ui-m2a-final-material-tests.log` and
`/tmp/rheon-ui-m2a-final-setup-tests.log`; compile/lint evidence is separate.

Live cancellation remains parked on missing containment prerequisites. This
source review does not independently verify the host preflight or historical
native artifacts, authorize provisioning/publication, or close those gaps.
