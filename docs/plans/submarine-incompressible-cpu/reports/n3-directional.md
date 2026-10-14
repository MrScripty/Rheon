# Bounded N3 directional refinement diagnostic

## Admitted scope before implementation

Parent delegation of 2026-10-14, following user instruction `Sentinel_39a31f0fa3208191b7fa1486e6188867` (“do those things”), authorizes a bounded next CPU validation slice and draft PR. The canonical plan is [plan.md](../plan.md); inspected clean main is `d51490c70b02d58d2bbc058da401a014d98b3d3a`. Independent GPT-6.1 Sol High design owner `/root/validation_design` and numerical/standards reviewer `/root/standards_review` are separate from implementation owners `/root` and `/root/i1_exporter`. Both reviewers admit the directional diagnostic boundary described here; final source/capture review remains required.

The retained I1 straight case has physical stream force −0.0974765752821589 N and algebraic stream force −0.09124856245761553 N: **6.825327%** difference relative to the algebraic force. Its full isotropic doubling exceeds the unchanged 4096 mixed-unknown cap. This slice measures conditional coordinate response using actual native fields, rather than increasing that cap or calling anisotropic response full continuum convergence.

Freeze two independent three-level families before observing results, each for straight uniform flow and the original polynomial manufactured solution. Body bounds, domain bounds, physical controls, gauge, forcing semantics and reference point stay fixed. Baseline is shared, yielding ten actual native runs.

| Level | Counts | Spacing (m) | Predicted mixed DOFs |
| --- | --- | --- | --- |
| baseline | (8,5,5) | (0.25,0.25,0.25) | 669 |
| X medium | (16,5,5) | (0.125,0.25,0.25) | 1365 |
| X fine | (32,5,5) | (0.0625,0.25,0.25) | 2757 |
| Y medium | (8,10,5) | (0.25,0.125,0.25) | 1383 |
| Y fine | (8,20,5) | (0.25,0.0625,0.25) | 2811 |

All retain body `(-0.5,-0.125,-0.125)` to `(0.5,0.125,0.125)` m, domain `(-1,-0.625,-0.625)` to `(1,0.625,0.625)` m, at least two complete fluid layers, and native sector profile `rheon-i1-sector-half-cell-v1`. Export actual DOFs/payload/pivots and verify independent geometry-derived counts. X/Y refinement changes wall sampling in that direction; Z, coupled refinement and domain uncertainty remain unresolved. A mixed X/Y grid is deferred so this family has one explicit bounded acceptance contract.

## Acceptance and independent execution

1. All ten exported native records must pass the unchanged independent `oracle.py` row, all-cell divergence, geometry, wall trace, pressure/shear/moment/reaction/work and constitutive checks. No tolerance, pivot floor, resource cap or solver equation changes. Failure retains the actual prefix and terminal failed provenance; it does not remove a grid or substitute analytic samples for solved fields.
2. Capture from a clean exact source archive using the existing `run.execute` build/execution protocol, with an explicit second immutable fixture substitution by the new directional wrapper. Default I1 fixtures and runner remain byte-identical. The exporter admits only the original digest or one new reviewed digest and records the selected actual digest. Pin source/tree/archive/compiler/environment/executable/argv/stdout/stderr/raw records, unchanged oracle/runner/default fixture, new wrapper/tests/fixture and origin digest.
3. A separate cold-process command, supplied the externally pinned terminal origin digest and source, checks all ten records/provenance and recomputes the study. Integrity pins are not signed remote attestations. An independent reviewer executes this recheck and assesses numerical interpretation.
4. Report raw physical pressure/shear forces and moments, algebraic force/moment, their mismatch relative to algebraic load and `F_ref=mu U_ref L=0.01 N` (moment scale `F_ref L`), every actual spacing and all residual/resource reports. Retain the baseline 6.825327% gap; no threshold defines a successful trend.
5. Report manufactured volume-weighted velocity/pressure errors at all actual grids. Directional QoI differences and, only for resolved monotone positive-order series, descriptive observed order may be calculated. Noise-limited, constant, oscillatory and nonpositive-order cases are explicitly classified; no simple extrapolation/GCI or asymptotic certification is emitted by this slice.
6. Original 13 Rust contracts, existing 31 Python tests, focused directional negative tests, formatting and strict core-only Clippy remain affected supporting checks. Actual native capture and independent recheck decide this diagnostic acceptance. Final source review and draft-PR CI are tracked separately.

Acceptance here means independently reproduced directional diagnostic evidence. N3 stays pending: original V05 fine physical/control-volume agreement target `0.005 F_ref`, V07 smooth full-mesh behavior and V08 full spatial/domain uncertainty targets are unchanged. Steady I1 has no time discretization. This is numerical verification, not experimental validation; N4–N7, finite-Re I2, motion/ascent, GCL, turbulent submarine readiness, jets and Tuldok promotion remain outside the slice.

## Exact implementation write set and design rationale

Root owns `examples/submarine_stokes.rs` (second pinned fixture admission and selected digest only), `validation/submarine/directional_fixtures.json`, and this report plus `plan.md`, `issues.md`, `execution-ledger.md` in this plan directory. `/root/i1_exporter` owns only `validation/submarine/directional.py` and `validation/submarine/test_directional.py`. Reviewers write no implementation files. No `src`, solver experiment, native contract tests, existing oracle/runner/fixtures, Cargo/dependencies, CI or compressible/recording source edits.

Reuse the existing source-provenance execution and independent rational operator rather than duplicating assembly or building a general solver backend. The new small wrapper adds deciding value: it freezes a matched family, prevents partial captures becoming convergence evidence, and aggregates actual loads/errors with explicit directional limits. Exact byte pins matter because fixture, source and oracle changes alter the numerical subject. Retain the diagnostic while N3 needs conditional-bias evidence; review replacement by a scalable coupled refinement family when admitted. No external code/data import, dependency change or license exception: existing MIT/Apache dual dependencies use the Apache option; new Python uses stdlib.

Primary sources inspected again 2026-10-14: [NASA NPARC spatial convergence tutorial](https://www.grc.nasa.gov/www/wind/valid/tutorial/spatconv.html) supports coordinate-specific refinement and warns about order/asymptotic assumptions; [ITTC 2024 revision 05 uncertainty procedure](https://www.ittc.info/media/11950/75-03-01-01.pdf) distinguishes verification/validation and numerical uncertainties. This slice implements neither a complete ITTC uncertainty procedure nor the plan's future full GCI qualification. No downloaded code/text or SUBOFF dataset is imported.

Source-guided standards at Coding-Standards `188beda1fa477d21d576c233dd8c7f4c4c267d23`: implementation/verification/commit workflows require predeclared scope, independent claim-level evidence and retained history; directly inspected applicable workflows guide this change. No application Standards Engine tool is exposed and its authoring-only skill is inapplicable; no automated standards certificate is claimed.

## Resource and branch disposition

One dense matrix per solve, unchanged native 4096 cap and 256 MiB exporter retained-payload cap; sequential native execution, one build job, no hardware-speed/RSS promise. Records keep the unchanged 16 MiB bound. Fresh outputs reside outside checkout and cannot overwrite retained captures. The branch `validation/submarine-directional-n3-20261014` targets main for a draft PR only; parent owns integration and disposition. Worktree `/workspace/Rheon-submarine-directional-n3` remains retained under this active review contract, protected by its local and pushed branch until parent closes/integrates the proposal and authorizes targeted cleanup. All previous worktrees/refs remain untouched.

## Execution evidence

Pending implementation and independent source/capture review. No directional trend or continuum acceptance is asserted at admission.

Pre-capture supporting checks: original 13 Rust contracts pass (61.58 s); original 31 Python tests pass normal (10.373 s) and optimized (10.121 s); strict core-only exporter Clippy and formatting pass. Independent fixture/exporter admission review passes: baseline controls remain exact copies, refinements change only counts/spacings, and geometry-derived DOF enumeration confirms the table. New fixture SHA256 is `06889b4dc2c059a8d1d10f7d9805f2506e5e2a59da95c4f9ac47fe56d3afc992`; exporter SHA256 is `229287012e30d454ccfc6c160db6d2cdf783da36c43c27275c76b54e2b7c897b`. These supporting checks do not prove native refinement execution.

Reproduction uses the pinned scientific source commit recorded below after capture:

```sh
python3 validation/submarine/directional.py capture \
  --source FULL_SOURCE_COMMIT \
  --fixtures-sha256 06889b4dc2c059a8d1d10f7d9805f2506e5e2a59da95c4f9ac47fe56d3afc992 \
  --output FRESH_ABSOLUTE_OUTSIDE_CAPTURE
python3 validation/submarine/directional.py recheck \
  --source FULL_SOURCE_COMMIT \
  --fixtures-sha256 06889b4dc2c059a8d1d10f7d9805f2506e5e2a59da95c4f9ac47fe56d3afc992 \
  --capture ORIGINAL_CAPTURE --origin-sha256 EXTERNALLY_SUPPLIED_ORIGIN_SHA256 \
  --output FRESH_ABSOLUTE_OUTSIDE_RECHECK
```

Use Rust 1.92.0 and the environment expected by the existing provenance runner. The capture command supplies a candidate origin digest; a separate executor supplies that observed digest to the cold recheck. Neither step authenticates remote execution or implies parent signature/authority. All six validator/study input files must match their bytes at the selected source; later report-only commits do not relabel the original capture.

Settled source-boundary independent review passes. Wrapper SHA256 `c9bd70ac8234821f16f47aab27a8468eaf37d44b1672974797ed9218a79bbdad`, focused-test SHA256 `160246dfcc32b356ad50a04c64542e8b3f071ea3545d52d59c577103790ed2b8`; nine focused tests independently pass (0.144 s), full 40 normal (9.515 s) and optimized (9.563 s) pass. The nested unsigned-integrity wording was reviewed directly; no assurance-specific test is claimed. Default core/oracle/runner/fixture/native contract hashes remain the accepted I1 bytes. This source freeze precedes native execution; capture and diagnostic acceptance are still pending.
