# Independent UI-5 precision review

Reviewed 2026-10-20 in `/workspace/Rheon-ui-precision`, detached prerequisite
`7318023f04946171d7909119300af76e455c2926` plus the preserved M2a overlay.
Final reviewed `src/bin/desktop/settings.rs` SHA256:
`099fa897a53feee4cb1de483655893f50e6c713c39e66c1a25c8f0549ee2c121`.
Only this report was authored by the reviewer. No runtime tests, numerical,
model or native workloads were executed by the reviewer.

## Disposition

No remaining material source finding in this bounded precision repair. The
reviewed exact-test evidence resolves the observed isolated-form focus-rounding
failure while retaining intentional editing and compact idle formatting. This
does not accept native focused-field appearance, complete application traversal,
assistive technology, scientific correctness or whole M1/M2 acceptance. No later
timeline/scene/property/physics implementation is included in this disposition.

UI-5 was observed after M2a. The deciding before test exercises unchanged M2a
production with the new regression; it is not an untouched-prerequisite baseline
experiment. The earlier M2a review's stronger “preexisting defect” description
was not empirically established. A similar prior format/reparse mechanism is
source-based inference only. This report corrects that evidence boundary without
rewriting the preserved historical reports.

## Source and invariant review

`precise_drag` captures the next automatic ID immediately before adding the
DragValue. In pinned egui 0.33.3, `Ui::add` delegates directly to the widget, and
DragValue uses that same ID and registers focus before calling its formatter.
The callback therefore queries current focus at the correct point, including the
first keyboard-focus frame. It uses round-trip Debug formatting while focused
and the existing style formatter/decimal policy while idle. Scientific notation
avoids very long fixed-decimal strings for tiny valid values.

Unchanged focused text reparses to the same temporary display value, so existing
`.changed()` guards avoid needless canonical SI writeback. The two insertion
sites retain density factors, temperature/molar conversions, ranges, speeds,
parser, clamping and suffixes. No numerical default, worker, recording, archive,
dependency or model code changes. The shared quantity helper also serves main
domain/maximum-step/stiffness and emitter fields; their caller sources are
unchanged, but presentation reach is broader than material fields.

Two implementation hazards identified during review are resolved:

- Full-precision density text initially pushed the following unit selector
  outside the narrow form. An explicit next-row decision when remaining width
  is insufficient preserves complete unit labels. `Ui::end_row` changes layout
  without consuming an automatic widget ID.
- Ordinary pointer click originally selected the compact caption's length,
  leaving a suffix when the edit buffer became longer. The unchanged thermal
  click-to-replace test caught a real editing regression. After widget addition,
  only `clicked() && gained_focus()` resets the existing TextEdit selection to
  the full upcoming buffer length. Later caret clicks remain untouched. The
  pre-widget scalar copy supplies that transient cursor length; it is not stored
  as another canonical value or dirty-field owner. Tests retain ordinary typing
  without forced Ctrl+A, rather than bypassing the regression.

The adapter relies on the pinned egui ID/focus behavior; retain these regressions
when upgrading that dependency.

## Inspected execution boundary and evidence

Inspected all five allowlisted tests and their complete application helper paths:
local Context/frame/text/pointer/key/click/numeric-label helpers, material/domain
forms, quantity, and pure material/configuration defaults. They contain no
simulation, Worker, archive, process, filesystem-output or cleanup path. Numeric
label lookup identifies the isolated density editor or unique g/mol suffix,
without assuming a particular compact spelling or normalizing canonical values.

The lead enumerated 168 names using list-only, then ran exactly one fully
qualified allowlisted name per invocation. The allowlist SHA256 is
`bb8aebb4607281551cfcfcb2c88eb206b8b384b1a39be4c766933de00d7a60d8`.
Reviewed final `/tmp/rheon-ui-precision-final-v3-exact-{1..5}.log`: each records
one passing test. These cover:

- Exact material bits/configuration across pointer focus/blur and 28 forward
  plus 28 reverse Tabs, Air and precise liquid properties including tiny
  viscosity, all three density units, and 240/280 logical-pixel widths.
- Ordinary click-to-type replacement, explicit density and molar SI edits,
  invalid text, upper/lower clamping and actual ArrowUp adjustment.
- Existing thermal edit/reopen, preset/fill/unit ownership and open-menu width
  regressions through the same inert helpers.

The lead's final strict Clippy/affected locked check/format logs pass; reviewer
`git diff --check` also passes. The reviewer did not execute tests or broaden
selection. The original material-bit failure, intermediate width failure and
pointer-edit failure remain in separate before/after/v2 logs; final v3 logs do
not overwrite them.

## Preservation and remaining limits

The original M2a settings source still hashes to
`0937b5e293fdd024641f46382900debac0b6f8c263d2a3c5d61fc0f8191dc102`
in `/workspace/Rheon-ui-hierarchy`. The preserved packet/patch and earlier
test-selection incident retain their historical scope. Earlier automatic
successful-test teardown removed disposable numerical outputs before detection;
those bytes/resource measurements cannot be reconstructed or counted as retained.
No repeat, cleanup or reclassification follows from this precision review.

UI-5 now has source/headless regression evidence for the inspected forms and
finite fixtures. Full application focus order, all shared-helper caller contexts,
native rendering/scales, representative usability and the word-count target
remain unverified. Live cancellation remains parked. There is no commit,
publication, kernel-resource provisioning or execution admission in this review.
