# Independent native-feedback width repair review

Reviewed against prerequisite `8bf5f41feea2e06d2b94b9b2e83382091e6cf3a7` on
2026-10-20 by `ui_source_review`. This reviews the local width-repair diff, not
the prerequisite or later planned redesign. No native repair verification is
claimed; only this report was authored by the reviewer.

## Disposition and finding

No remaining material source finding in this bounded repair. It is suitable for
the declared static checks and exact-revision native verification; source review
does not close that native gate.

**W1 — medium, resolved:** the initial wrapped information caption could overlap
the fixed-y collision annotation at narrow widths. The repair computes the
information galley first, then width-wraps and places the mesh note below its
actual height with an 8px gap. All policy/sampling text remains. The revised
300px viewport test uses an inert single-triangle mesh and asserts both text
widths and non-overlapping vertical bounds. No Worker or simulation is involved.

## Other source checks

- Wrapped form rows retain text, field order, numeric owners, ranges, units,
  conversion, enabled state and returned numeric-widget responses. No scientific
  or worker/recording contract changed.
- The recording disclosure's explicit original string salt matches the old
  `ui.collapsing`/`CollapsingHeader::new` identity in pinned egui 0.33.3. Supplying
  a wrapped galley preserves the full semantic label and normal disclosure
  keyboard behavior; no new persisted identity is introduced.
- Information/navigation painter text preserves original copy, font sizes and
  colors. Width-bounded layout fixes horizontal clipping; collision annotation
  stacking now follows the laid-out information caption rather than a fixed y.
- `git diff --check` passed. The lead's focused width log showed seven UI tests
  pass, including the loaded-mesh regression after W1 repair. Affected strict
  Clippy reported success at the inspected intermediate diff; the lead owns the
  final static-gate record. These prove their exact egui layout/input assertions
  only. The expanded form fixture covers both setup workflows and opens the
  recording/workspace disclosures where present. It is not a native 150% capture
  or exhaustive package/mesh/error-state validation.

No solver, model, evaluator, corpus, GUI or scientific workload was run by this
reviewer. Exact-revision native expanded-form, caption and interaction evidence
remains required after the repair. Do not infer a native pass from headless
layout tests or earlier JPEGs.

Final reviewed source SHA256:

- `src/bin/rheon-desktop.rs`: `f3b9bba3e6834a183220b4ff2f2b9413bb84d0f70dc951e4ee089458feb5e16d`
- `src/bin/desktop/settings.rs`: `ee0a8ccf666a917dec92ef695b4e55e32f268fd1681773324d821735bc4a0613`
- `src/bin/desktop/configured_controls.rs`: `5463e18b3b5a139c448dd327b23318605664e90aea4690187981177e7f68e60b`
- `src/bin/desktop/viewport.rs`: `e187ce147a09c2bd2ff789c0c2172e60e466538483a00dd4ec3ebbbc6642f3f2`
- `src/bin/desktop/setup_ui_tests.rs`: `f226120c32c7a20a71d5f7f9f282d17d1d58772354781a41b73514a6ea96b7f8`
