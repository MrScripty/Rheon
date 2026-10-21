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

At admission, implementation and source/capture review were pending and no trend was asserted. The frozen implementation and complete ten-case diagnostic now pass separate source review and independent cold recheck; [draft PR83](https://github.com/MrScripty/Rheon/pull/83) awaits parent disposition. Full N3–N7 remain pending.

Pre-capture supporting checks: original 13 Rust contracts pass (61.58 s); original 31 Python tests pass normal (10.373 s) and optimized (10.121 s); strict core-only exporter Clippy and formatting pass. Independent fixture/exporter admission review passes: baseline controls remain exact copies, refinements change only counts/spacings, and geometry-derived DOF enumeration confirms the table. New fixture SHA256 is `06889b4dc2c059a8d1d10f7d9805f2506e5e2a59da95c4f9ac47fe56d3afc992`; exporter SHA256 is `229287012e30d454ccfc6c160db6d2cdf783da36c43c27275c76b54e2b7c897b`. These supporting checks do not prove native refinement execution.

Reproduction uses the pinned scientific source commit recorded below after capture:

```sh
python3 validation/submarine/directional.py capture \
  --source 4b98877039a9e5d4b73620337e9f21e573b6db4b \
  --fixtures-sha256 06889b4dc2c059a8d1d10f7d9805f2506e5e2a59da95c4f9ac47fe56d3afc992 \
  --output FRESH_ABSOLUTE_OUTSIDE_CAPTURE
python3 validation/submarine/directional.py recheck \
  --source 4b98877039a9e5d4b73620337e9f21e573b6db4b \
  --fixtures-sha256 06889b4dc2c059a8d1d10f7d9805f2506e5e2a59da95c4f9ac47fe56d3afc992 \
  --capture ORIGINAL_CAPTURE --origin-sha256 EXTERNALLY_SUPPLIED_ORIGIN_SHA256 \
  --output FRESH_ABSOLUTE_OUTSIDE_RECHECK
```

Use Rust 1.92.0 and the environment expected by the existing provenance runner. The capture command supplies a candidate origin digest; a separate executor supplies that observed digest to the cold recheck. Neither step authenticates remote execution or implies parent signature/authority. All six validator/study input files must match their bytes at the selected source; later report-only commits do not relabel the original capture.

Settled source-boundary independent review passes. Wrapper SHA256 `c9bd70ac8234821f16f47aab27a8468eaf37d44b1672974797ed9218a79bbdad`, focused-test SHA256 `160246dfcc32b356ad50a04c64542e8b3f071ea3545d52d59c577103790ed2b8`; nine focused tests independently pass (0.144 s), full 40 normal (9.515 s) and optimized (9.563 s) pass. The nested unsigned-integrity wording was reviewed directly; no assurance-specific test is claimed. Default core/oracle/runner/fixture/native contract hashes remain the accepted I1 bytes. At that source-freeze checkpoint, native execution and diagnostic acceptance were pending.


### Actual frozen execution and review

Scientific source `4b98877039a9e5d4b73620337e9f21e573b6db4b`, tree `59d2b29e1251a099045afcab112ec90f697c9bcc`, is the exact clean archive build/execution subject. Later report-only commits preserve its code and do not relabel these captures.

| Subject | Path / SHA256 |
| --- | --- |
| Terminal completed origin; ten outputs, 13 successful commands | `/tmp/rheon-n3-directional-4b98877/origin.json`; `ebc1a18974cfd2be96bc4887583b751f338a8c343757be0d188942d33d262993` |
| Exact Git archive | `/tmp/rheon-n3-directional-4b98877/source.tar`; `96550badde849dc36f3ae7fabaab8594e3cb8a3e1cbbf92617a61a7ad3f89d78` |
| Actual release executable, 1,138,832 bytes | `/tmp/rheon-n3-directional-4b98877/target/release/examples/submarine_stokes`; `51aac36d1491e6fc633ab486efb6df1fbfc41141fcaac2ad3a767467a2da9e7e` |
| Original capture assessment | `/tmp/rheon-n3-directional-4b98877/assessment.json`; `f571fe13522f1ea9fa81e27eb7d6e95a576265e068f5da23daf6a441fb34afdd` |
| Wrapper source-binding envelope | `/tmp/rheon-n3-directional-4b98877/directional-capture.json`; `9708a606df8177102908c8aedaa3a619d73f6392f0554d6162c16a6ddac7fc2e` |
| Independent cold-process study | `/tmp/rheon-n3-independent-recheck-4b98877/directional-study.json`; `e0ca9c89fd980982b129222912f5c6e9dbf68040d103ccb159c4989bf895119d` |

Independent reviewer verified all 1,137 extracted tracked files against the archive, source/tree/archive/executable/stream/output bindings, all ten unchanged-oracle assessments (exactly matching the original capture assessment), and all 34 constitutive controls per record. The maximum momentum defect is `2.3036273316486455e-16 N`; maximum physical cell divergence is `1.5411120730252081e-15 /s`. Every original scale/operation-based gate remains unchanged. Maximum retained solver payload is 65,327,772 bytes; maximum raw record is 1,171,782 bytes, within the unchanged 256 MiB/16 MiB caps. These are retained-payload/record bounds, not RSS or speed certification.

After the ten-run family completed, a separate manual default-exporter regression used the same captured executable once with original fixtures and once with a whitespace-tampered fixture. Original `tiny-straight` raw record is byte-identical to the retained d7a3c01 original capture, SHA256 `6e9bae2d99af34a235644714e76b01a42b0d73db2cd443a02ffefe90fab07274`; tampering returns 2 before output creation. Evidence `/tmp/rheon-n3-default-compat-4b98877/proof.json`, SHA256 `84cacd972469f4e2c99d70c80aa013eabd3e2b14099facd438f8de836a60e7ab`. This is a focused default-admission regression, not a new full original-suite capture; earlier I1 origins and raw pending assessment templates remain unchanged.

### Actual directional findings

Stream components in N; forces are fluid-on-body with the original convention. Full vectors, moments, all raw fields/traces/reactions, per-case residuals/pivots/resources and descriptive series are retained in the pinned records/study.

| Straight grid | Physical pressure | Physical shear | Algebraic total | Absolute gap / F_ref | Relative gap (%) |
| --- | --- | --- | --- | --- | --- |
| baseline | −0.01353017931 | −0.08394639597 | −0.09124856246 | 0.6228012825 | 6.825327059 |
| X medium | −0.01872723956 | −0.08274036599 | −0.08800419177 | 1.3463413780 | 15.298605111 |
| X fine | −0.02264423910 | −0.07955893840 | −0.08714439999 | 1.5058777515 | 17.280258418 |
| Y medium | −0.01445946658 | −0.09197425806 | −0.10325971702 | 0.3174007630 | 3.073810119 |
| Y fine | −0.01475842308 | −0.09290211811 | −0.10763697615 | 0.0023565042 | 0.021893073 |

The Y-fine stream discrepancy is `2.356504151732053e-5 N`, locally below the unchanged `5e-5 N` component target. X-fine disagrees more than the retained baseline. A locally small net discrepancy can include cancellation of reconstruction errors and cannot certify continuum accuracy, all-direction fine-grid qualification, domain independence, or V05/full N3 acceptance.

Actual manufactured volume-weighted L2 norms below are unnormalized integral norms (velocity units `(m/s)*m^(3/2)`, pressure units `Pa*m^(3/2)`), not dimensionless errors or error uncertainty bounds.

| MMS grid | Velocity L2 | Pressure L2 |
| --- | --- | --- |
| baseline | 2.52605341465863e-8 | 2.3611373596309016e-7 |
| X medium | 3.598443388765919e-8 | 9.094777354853161e-7 |
| X fine | 3.82353377581845e-8 | 2.3077607250268064e-6 |
| Y medium | 2.233778617410216e-8 | 5.147953346733589e-7 |
| Y fine | 2.3855988774536352e-8 | 1.5511434543814682e-6 |

Pressure errors worsen in both directional families. X velocity error increases; Y velocity error is nonmonotone. The machine-reported X velocity increment-fit `p=2.252253286616018` fits contracting increments while the error values increase; it does not establish a limiting value and is **not** method accuracy order or error convergence toward zero. No V07 or full spatial uncertainty pass follows. These observations require further operator/coupled-family investigation; their cause is not established by this diagnostic.

The scalar order-resolution heuristic is `4096*epsilon*sum(abs(three QoIs))`, with no unit floor. It does not bound cancellation in gross pressure/shear/wall sums or reference/iterative errors. Exclude symmetry-zero transverse forces, centroid moments and unresolved near-zero components from physical accuracy/order inference even if a descriptive machine field has positive order. Report all raw values and classifications, and do not replace or filter failed trends. No extrapolation/GCI or statistical confidence interval is supplied. Z/coupled/domain/geometry/model uncertainty remains unresolved; steady time discretization is N/A. This is numerical verification, not experiment validation or engineering qualification.

Independent GPT-6.1 Sol High source and cold numerical review returns PASS for this bounded diagnostic evidence only. N3–N7, downstream force feedback, moving/ascent capability, experiment rights/model matching and production/Tuldok promotion remain pending.

### CI and delivery boundary

At scientific source `4b988770`, [Rust CI38614284984](https://github.com/MrScripty/Rheon/actions/runs/38614284984) is terminal success, including all three feature profiles, strict Clippy/core dependencies and executable contracts. [Push scope38614240330](https://github.com/MrScripty/Rheon/actions/runs/38614240330) and [PR scope38614284594](https://github.com/MrScripty/Rheon/actions/runs/38614284594) are terminal success. Later documentation-only head checks are tracked separately; these executions keep their original source identities. No source/scientific input changed after the capture. Draft PR83 remains unmerged; no tags, merges, force pushes or cleanup occur.
