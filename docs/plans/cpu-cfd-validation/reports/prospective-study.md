# Prospective acoustic resolution-study support

## Decision and authority

After reviewing the full native diagnosis, the parent approved preparing a
16/24/32 resolution-study revision, subject to independent methodological
review and canonical solver-plan update by its planning owner. Parent subsequently admitted canonical plan
`fc2270a85e38080d9943048aaf689d99545d3715` (PR75) and native source
`5a049ed604b99817a428d10bbc0140732d3421d6` (PR74). The capture selector
follow-up and actual native execution are recorded below.
This report implements verifier support only; it supplies no higher-resolution
native run or acceptance. The original 8/12/20 study remains failed and its
external origin records remain unchanged, as recorded in
[native diagnosis](native-s1-diagnosis.md).

## Versioned exporter contract for parent coordination

Suite JSON schema remains 1 and numerical method remains
`llf-euler-periodic-v1`. Select the prospective family with the explicit top-level
field:

```json
{"study_id": "s1-cell-average-acoustic-16-24-32-v2"}
```

| Study ID | Acoustic meshes | Compatibility |
| --- | --- | --- |
| `s1-cell-average-acoustic-8-12-20-v1` | 8³ / 12³ / 20³ | Missing `study_id` means this original study only |
| `s1-cell-average-acoustic-16-24-32-v2` | 16³ / 24³ / 32³ | Explicit ID required |

Uniform/differential cases and Sod 24/48/64 remain unchanged: exactly eight
cases, with no mixed populations. Unknown or malformed IDs, old fields labelled
as prospective, prospective fields without an ID and mixed families reject.
Successful numerical assessments include `study_id`; origin already hashes the
entire suite manifest and verifier source identity, binding the selected study.
Neither parser support nor a synthetic numerical assessment supplies external
plan admission. No old receipt is refreshed or reclassified.

Acoustic order uses each selected mesh ratio, including 32/24 for the prospective
second interval. All numerical criteria, cell-average oracles, mode amplitudes,
phases, final time and norm remain unchanged. This change does not alter or
propose changing native CFL; exporter/core owners retain its existing contract.
The >0.6 order, <8% first-mode speed, conservation, admissibility and Sod gates
remain identical. The mode-two speed scope remains reported-only as previously
admitted and tested. The resolution heuristic is motivation for prospective
sampling, not a convergence theorem or evidence that the new native runs pass.

## Verification

The original suite probes remain in `ExportTests`; `ProspectiveExportTests`
runs the same numerical/layout/time negatives against the new fixed family.
Synthetic positive fields remain labelled policy probes, with artificial Sod
exterior compensation, and never supply native evidence. A new negative
prescribes decreasing density errors proportional to n^-0.55 and requires the
unchanged `acoustic/order-below-0.6` failure for both families. Another check
compares reported orders with direct logs using literal study mesh ratios.

Commands, with existing NumPy 2.3.5 and pinned Rust 1.92 for the origin protocol:

```sh
source /workspace/rheon-setup/env.sh
python3 -m unittest discover -s tools -p test_compressible_verifier.py
python3 -O -m unittest discover -s tools -p test_compressible_verifier.py
python3 -m py_compile tools/verify_compressible.py tools/test_compressible_verifier.py
git diff --check
```

Final 41-test runs passed normally and under `-O`, including both new frozen-order
negatives; Python compilation and diff checks passed. A separate read-only
implementation review found no defects in study selection, legacy behavior,
unchanged gates or mesh-ratio calculation, and separately confirmed the added
frozen-order negative normally and under `-O`. Diagnostic-only application of the current oracle to both
preserved original field sets still fails `acoustic/order-below-0.6`; their
original `origin.json` SHA-256 values remain exactly
`7b53940fd168ae26e9cd0b29ffde0c191dab5b0d01a5be2953de21eae5c4dbec` and
`e7c098d13eb19fcc54abb37436c0f0e0ffc7e851cd30487e0a590a712c1866a7`.
This diagnostic does not replace origin verification with its pinned verifier.

## Follow-up disposition

Parent supplied the reviewed canonical-plan and revised native pins, and actual
higher-resolution capture is now complete as recorded below. Current validation
execution has no remaining blocker. Parent retains integration/source admission
and combined native API evidence ownership. Original failures remain preserved;
forecasts, synthetic probes and hosted dense3D contracts do not supply this result.

## Admitted execution follow-up

Read the exact admitted canonical resolution report/first-slice criteria and
native selector at the pins above. Capture now accepts `--study-id` and forwards
the explicit prospective ID to the unchanged native executable. It records the
selected study in origin and rejects a different emitted native suite ID before
numerical assessment, both during capture and retained-origin verification.
Default capture retains the original native command without a study flag. No
manifest is edited to select a family. All 43 tests pass normally and under -O;
the actual tiny protocol probe executes both paths and verifies forwarded ID.
Independent read-only review found no defects and independently reran all 43
tests normally/-O. Actual native results follow at their exact recorded pins.

The required fresh capture recipe is:

```sh
source /workspace/rheon-setup/env.sh
python3 tools/verify_compressible.py capture \
  --source-repo /workspace/Rheon-native-prospective \
  --native-commit 5a049ed604b99817a428d10bbc0140732d3421d6 \
  --study-id s1-cell-average-acoustic-16-24-32-v2 \
  --output FRESH_EXTERNAL_DIRECTORY
```

## Completed actual execution

The remaining action above is complete at clean verifier 94a8317 and native 5a049ed
under admitted canonical plan fc2270a8. Actual capture and externally pinned repeat
verify satisfied exported-field checks; no original failure was changed. See
[native result](native-prospective-results.md) for full exact pins, hashes, commands,
observed metrics and remaining parent integration scope.
