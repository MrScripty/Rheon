# Independent I1 validator implementation and contract proposal

Status: independent exact-source native capture completed with unchanged gates;
all 18 records and complete controls passed recheck. Final independent review
and bounded N1/N2 disposition are recorded below. Canonical I1 remains Active; N1/N2, N3 continuum
convergence and N7 experimental validation remain pending. This report does not
change the canonical plan/ledger/issues or native owner authority.

Parent reassigned independent validation to the completed recording integration
executor after two fresh saved-environment container creations failed. Actual
filesystem/command access succeeded in `/workspace/Rheon`; clean branch
`validation/submarine-i1-independent-20261014` starts from main
`447ffc03ebc1f24ba99a221b868b153146b13992`. Recording/PR76/PR78 and Pages work
are complete or independently running and are not restarted. Native implementation
owner remains `01a139a7-da3b-71b3-b155-ddca2f780005`.

## Standards and bounded execution contract

Read canonical [plan](../plan.md), [first slice](first-slice.md),
[validation design](validation-design.md), storage and standards/design records,
and pinned Coding-Standards `188beda1fa477d21d576c233dd8c7f4c4c267d23`
implementation, independent-oracle, build and commit workflows. This is source-
guided application, not a Standards Engine certificate. Existing admitted I1
operation and parent reassignment authorize exactly these six paths:

- `validation/submarine/run.py`
- `validation/submarine/oracle.py`
- `validation/submarine/test_validation.py`
- `validation/submarine/README.md`
- `validation/submarine/fixtures.json`
- this `reports/i1-validation.md`

One coherent result is a runnable independent stdlib record/row/load validator,
a fresh exact-source native capture runner, prospective fixtures and typed
availability/failure results. The [contract README](../../../../validation/submarine/README.md)
is the normal-readable proposal before capture. Parent/native agreement on its
fixed schema and exact fixture bytes is required; no producer source, shared
exports, Cargo/CI, recorder, canonical plan or book inventory is edited.

Expectations come from independently differentiated analytic polynomials,
geometry-derived MAC incidence/strain, closed cuboid identities, exact pressure/
Newtonian traction and a separate rational no-body solve. No native assembly,
flux, stress quadrature or reconstruction helper is imported. The test serializer
is confined to tests and labels its analytical uniform/rest records as synthetic;
it is never available as a runner replacement for native output.

## Prospective numerical decisions and independent review repairs

The half-dissipation functional, clipped edge quadrature, signed reflection,
component compatibility, affine boundary/gauge elimination, all-row residual
checks and wall stencils are derived in the README. The admitted tiny body has
474 velocity plus 195 nongauge pressure unknowns (669 total). Only a separate
2³ no-body 19-unknown fixture is rational-solved; the exact backend cap is 128.
No body rational-solve or continuum-drag claim follows.

Independent GPT-6.1 Sol High review identified and repaired:

- Reflection must retain the upper/lower Cartesian sign; a rigid rotation is
  the deciding zero-strain control. Zero tangential derivatives apply only to
  the stationary body, not nonuniform outer MMS data.
- The functional is half physical dissipation. Tangential ghost-Dirichlet
  reactions/work and known pressure-gauge momentum terms remain explicit.
- Net-zero load and reconstructed-trace comparisons need independently
  accumulated primitive/patch magnitudes for roundoff, rather than a near-zero
  net-value budget. No unit-sized allowance is introduced.
- Constitutive controls cannot choose their own expectations or use an empty
  closed surface. Fixed fixture parameters, exact patch geometry/inventory,
  patch-level shear, Couette/Poiseuille signs and wall work are checked.
- Capture must remove ambient compiler/feature/wrapper overrides and bind actual
  source archive, executable, fixture, validator, case argv and output bytes.
  Parent-pinned origin integrity is distinct from signed/authenticated execution.

The frozen proposal has 18 native inputs and 34 constitutive-only controls.
MMS uses native point-value forcing times clipped dual volumes; continuum samples
are not discrete exact fluxes or an invented solution. Fine MMS orders and physical
versus algebraic load convergence remain unobserved. Source/schema agreement,
not merely matching this prose, precedes any actual native capture.

## Observed verification and remaining handoff

Harness tests exercise analytical rest/uniform records, exact 19-unknown solve,
669-body counting/cap refusal, rigid rotation/extension and clipped quadrature,
MMS differentiation/body no-slip, quadratic wall stencils, affine/hydrostatic
pressure and shifted moments, closed-net-zero patch shear, Couette/Poiseuille
stress/work, precisely identified corruption mutations, scaled pivot refusal,
JSON/partial-output bounds, fresh-output policy, native unavailability, provenance
pin mutations and build-environment sanitization. Observed current harness runs: **25 passed** normally and **25 passed** under
optimized Python, zero skips. Local authored links and exact six-path ownership
checks pass. Final independent review and draft-PR exact-head checks remain
separate; these harness results do not qualify any missing native output.

Native capture attempted only as an explicit availability test against the
current pinned main subject lacking the native example. The runner returned
`unavailable/native-capability`, exit 2, and created no output directory.
No numerical field, source capture, native force or N1/N2 pass was fabricated.
All test-run temporary files are outside the checkout; only authored source,
input fixtures and this report are tracked.

Remaining work: parent/native export-contract agreement and actual implementation
commit/binary handoff; execute fresh external native cases/controls, retain failures,
independently recheck their rows/loads/provenance and report claim dispositions.
Any changed quadrature, wall stencil, pivot or point/integrated MMS semantics
requires prospective reviewed contract revision, never retuned acceptance after
output. N3 needs its admitted resource-bounded mesh/domain family; N7 needs
separate rights/model/uncertainty evidence. No later slice starts here.

Branch is a retained protected proposal for parent serial integration, targeted
at main; normal push and draft PR only. Original refs remain intact, no force
push/rebase/cleanup/merge is performed. Parent owns subsequent acceptance and
branch disposition. The existing workspace is reused; no worktree is created.

## Historical corner-profile decision at 597c3ca — agreement required then

The original PR79 subject `52322ca18dd4e9a32c83e834b5f05a558163216a` and
fixture/profile SHA256 `6b6f0d55afaf3a92b6e88c116a0b858107e3e24e21a426f080d19345a78dab9b`
remain the earlier clipped-edge proposal. Its 25 normal/25 optimized harness
results prove their named properties for that proposal; they are not agreement
with the native owner's subsequently supplied corner operator. No receipt or
profile hash is relabeled. At this report-only decision subject, oracle/runner/
tests/fixtures/README bytes remained unchanged.

**Recommendation:** parent should explicitly agree
`rheon-i1-sector-half-cell-v1` for three-fluid-sector body corners, then authorize
the minimal prospective validator correction. This decision is reviewed but not
authoritative profile admission. Capture and N1/N2 agreement remain on hold.
No numerical threshold, solver cap, MMS convention or wall-load stencil changes.

Let the solid quadrant be `--`. Write `a=u_a(-h_b/2)`, `b=u_a(+h_b/2)`,
`c=u_b(-h_a/2)`, `d=u_b(+h_a/2)`, with full edge traces `g_a,g_b`.
The proposed fluid rows at `(-,+)`, `(+,-)`, `(+,+)` are respectively:

```text
2(b-g_a)/h_b - 2(c-g_b)/h_a
-2(a-g_a)/h_b + 2(d-g_b)/h_a
2(b-g_a)/h_b + 2(d-g_b)/h_a
```

Each has quadrature volume `V/4` and contributes `mu V S_q^2/8` to
half-dissipation. For the stationary body `a=c=g_a=g_b=0`, set
`X=b/h_b`, `Y=d/h_a`. The sector and original singleton energies are:

```text
Phi_sector = mu V (X^2 + Y^2 + X Y)
Phi_clipped = 3 mu V (X+Y)^2 / 8
difference = mu V (5 X^2 + 2 X Y + 5 Y^2) / 8
```

The difference is positive for every nonzero `(X,Y)`; this changes the local
Hessian and affine/reaction operator, not merely row packaging or quadrature
notation. At `h=mu=1`, `b=1`, `d=-1`, sector half-dissipation is **1 W** and
singleton half-dissipation is **0 W**; actual dissipation is twice each value.
This is a local corner shear comparison, not a claim of a full-system nullspace
or a native accepted Stokes solution.

An independently executed exact-rational diagnostic also locates the actual
PR79 body corner `(6,3,2)` with `h=1/4 m`, `V=1/64 m^3`, `mu=1 Pa s`.
A MAC curl variation supplies `X=1/100 s^-1`, `Y=-1/100 s^-1`, has zero
prescribed normal velocity and exact zero divergence in every fluid cell. Local
sector half-dissipation is `1/640000 W`; PR79's singleton contribution is zero.
Other rows can retain positive energy. This variation is analytical operator
analysis, never a fabricated native capture or a claimed stationary solution.

Consistent affine diagnostics supply both face samples and the edge trace from
`u=G x+u0`; every sector row is then `G_ab+G_ba`. Translation and rigid rotation
therefore have zero shear. Supplying rotation-like active values while keeping
the stationary body's prescribed faces/traces zero is inconsistent boundary data.
It cannot disprove the sector rule's affine patch consistency.

Pinned existing `src/aligned_strain.rs` at
`9efef18798d48268d9c5f28ea49e968cde55a39f` (corner lines 437–470) confirms
the stationary active corner coefficients and represented quadrant volumes.
It uses centered interior rows, a common half-cell flat-wall prototype,
inactive-column elimination and outer free-slip rows. It does **not** qualify
full-vector outer I1 Dirichlet treatment or the proposed retained prescribed
columns and virtual reactions. Those remain explicit I1 extensions to verify.

The smallest correction after explicit agreement is to replace only the
three-fluid-corner singleton with the distinct sector rows and their full affine
columns. Retain the equivalent four-fluid centered and stationary two-fluid flat-wall
rules; qualify their full prescribed-column/outer-Dirichlet extensions separately.
Never add a zero anchor at a fully interior edge. Record actual corner edge
traces, prescribed face columns, coefficients and physical reaction locations.
Derive momentum, reaction, torque and work from that same functional, rather
than adding an unexplained stiffness penalty or assigning its reaction to an old
face site. Update the named semantic profile/fixture digest and discriminating
corner/affine/virtual-work tests prospectively, preserving the original proposal.
Then independently qualify the full Dirichlet extension and actual native fields.
N3 continuum and N7 experiment gates remain pending.

GPT-6.1 Sol High read-only review independently confirmed the supplied formulas,
positive energy difference, affine consistency, original-source boundary scope
and minimal correction boundary. Exact diagnostic artifacts remain external:
`/tmp/rheon-i1-corner-decision-82q53qqr/compare.py` and `result.json`.
Script SHA256 `c8c838adf882e863f9e52d0363a677032605fcb824a4c261d5f726a55c8b8927`;
result SHA256 `781a874cff593da66e6af504419a6d9d5752f6aaa6ee9a7d351f536969a24f48`.
These hashes bind analytical diagnostics only, not native execution provenance.

## Historical correction and handoff at dedf04f

Parent explicitly accepted the independently reviewed `rheon-i1-sector-half-cell-v1`
operator and authorized the smallest correction described above. This section
supersedes the historical pending-agreement status at `597c3ca`; it preserves the
original proposal, mismatch decision and their receipt subjects. All six changes
remain on `validation/submarine-i1-independent-20261014` for draft PR79 and
parent serial integration. No native implementation or canonical ledger changes.

Only three-fluid body corners change numerical rows: three distinct sector rows,
each with `mu V/4` Hessian weight, retain prescribed face and full-vector edge
trace columns. Interior centered and stationary flat-wall aggregate rows remain
as before. Physical reaction sites explicitly bind every prescribed variable to
component, body/outer owner and SI point; corner torque uses its edge point.
Momentum, reactions and work derive from the same retained-column functional.
No interior zero anchor or arbitrary stiffness penalty is introduced.

The producer handoff is:

- Fixture schema: `rheon-submarine-i1-fixtures-v2`.
- Record schema: `rheon-submarine-i1-mac-v2`.
- Operator profile: `rheon-i1-sector-half-cell-v1`.
- Exact fixture SHA256:
  `3762540394238820ad47ca7ce4f8fc5b140e11abc0d52458a2451a4fe588f160`.
- Required `reaction_sites` map: every prescribed face/ghost/corner key has an
  integer component, `body`/`outer` owner and three SI-point binary64 bit strings.
- Corner trace key: `corner:A:B:I:J:K:COMPONENT`, `A<B`, at the MAC edge point
  with half-cell offset on the remaining axis. Full-vector body traces include
  these points and existing prescribed face/ghost points.
- Tiny body: 669 unknowns, 1464 strain rows, 780 prescribed scalar columns/sites,
  54 unique body trace points. Prescribed normal-face columns are not discarded.
- Origin envelope remains `rheon-submarine-i1-origin-v1`, additionally requiring
  the exact `record_schema` and `operator_profile`; old envelopes cannot acquire
  new provenance by relabeling.
- The existing external-fixture producer CLI and exact-source runner remain as
  documented in the [README](../../../../validation/submarine/README.md).
  Native source/binary/export implementation still belongs to
  `01a139a7-da3b-71b3-b155-ddca2f780005` and requires actual handoff.

All 18 case objects, 34 constitutive control objects, record/DOF/exact-solve limits,
pivot minimum, physical residual roundoff factor, normalized divergence threshold
and wall reconstruction coefficients are unchanged. This is a prospective named
operator/schema correction, not retuned acceptance after native output.

Observed corrected harness results: **31 passed normally and 31 under optimized
Python, zero skips**. Focused controls retain the prior divergence-free corner
variation (`Phi=1/640000 W` versus the old clipped `0 W`), verify exact corner
coefficients/weights and full prescribed columns, cover all three axis pairs and
corner orientations with consistent affine traces, and check zero rigid virtual
translation/rotation, boundary virtual work and exact functional differentiation.
Whole-functional affine Newtonian energy is exact for both the no-body and body
geometries when every prescribed face/ghost/corner site receives consistent data.
These moving affine traces are constitutive diagnostics, not stationary-body
native runs. Negative mutations refuse old schema/profile/digest, missing corner
trace/reaction, wrong reaction location/type and old origin profile members.

Independent GPT-6.1 Sol High read-only review reproduced the exact corner, affine
energy and whole-functional force/moment identities. Its oracle subject is SHA256
`6708b48b8d6adebe24d4e43c6fd2a6ccfc9431bfe0d77dc5de1cf6d893d83500`;
fixture subject is the revised digest above. Final source-set review/commit
identity is supplied in the PR handoff separately from this report's own bytes.
Pinned old `aligned_strain.rs` validates stationary coefficients only; full outer
Dirichlet, nonuniform traces and retained reactions are I1 qualification scope.
Oracle algebra tests and review do not establish that the native implementation
uses these columns, sites, forcing or work definitions.

At correction subject `dedf04f`, native execution had not occurred. Source/schema
handoff, fresh captures, full case/control execution and independent numerical
rechecks remain pending; N1/N2 are not accepted. N3 continuum mesh/domain and N7
experimental rights/model/uncertainty evidence remain pending. No restart,
signed execution attestation or later-slice claim is made.

## Fresh independent native execution at d7a3c01

Parent supplied draft PR80 final head
`f3d737cf62c8a359da2776424d6cab2a4f4f33ec` and explicitly authorized fresh
independent capture of implementation
`d7a3c01fbab2d78e5404afb84f15cf2cf17505fa`. Fetched that source without
changing the native branch. Exact Git comparison confirms the final head differs
only in `reports/i1-implementation.md`; the three native Rust files retain their
reviewed hashes. The independent validator remained clean at
`dedf04fb0d094064f5a163a4f7b78e7237e4438f` throughout capture, with its source,
fixtures, schema/profile, caps and gates unchanged. Only this assigned validation
report changes afterward. Canonical plan/ledger/issues and both PR dispositions
remain parent-owned; no merge, deployment or later slice is performed.

Read the exact PR80 implementation evidence and reproduction contract, the
canonical N1/N2 definitions and tiny/fine-grid separation, and the pinned standards
implementation/commit/build/oracle workflows before execution. This is an
application of those standards, not a Standards Engine certificate. Earlier
proposal523, report597, correctiondedf and native failed-run evidence retain their
original subjects. The native report preserves failed `a486f7c` output-parent
admission and `1c3dc0c` outer-ledger sign captures, plus the refused provisional
MMS; this worker did not delete, resume or relabel them. No fresh-capture failure
occurred in this independent run.

### Observed execution and separately anchored provenance

The unchanged runner created a new external directory
`/tmp/rheon-i1-validator-independent-d7a3c01-20261014`, archived the exact native
commit, selected actual Rust/Cargo 1.92.0, and built with
`cargo build --verbose --locked --release --no-default-features --example submarine_stokes`
in an isolated target with one build job and sanitized environment. It invoked
the freshly built binary for every fixture and consumed actual accepted output.
Every one of the 21 compiler/build/native commands returned zero. Driver exit
was zero; the origin is terminal `completed` with all 18 output associations.
No analytic field or previous owner's output substituted for fresh execution.

Root separately decoded every raw record, checked exactly 34 unique frozen
constitutive IDs per record (612 checked controls, explicitly not 612 native
solves), reran the unchanged numerical oracle, and invoked unchanged
`verify_origin` on each output with the root-observed pin. The audit also checked
all 1,127 extracted tracked files against Git blob identities, exact source/tree/
archive, actual executable, compiler streams/flags, immutable fixture bytes,
validator identities, 21 argv/stream hashes and 18 record hashes. Recomputed
case assessments match the captured assessment exactly. The actual source tree
is `9ff16940c14ded69f261885ec31b6fbc617357c1`.

The observed hashes bind this fresh execution. An observed integrity pin is not
a signed remote attestation or proof of reproducible builds. This rebuilt binary
has its own hash and size (1,138,688 bytes); the prior owner's executable hash
is not relabeled or required to be identical across build contexts.

| Fresh artifact | SHA256 |
| --- | --- |
| `origin.json` | `5e723946a4186435478cb1dcdb587b3a9afa4d82a816b95bec4f1bf117daf6aa` |
| `source.tar` | `152e9a0ccfb2be29437a87b5b755c1a8edcbe8cb164007fca6996f0f7e872e58` |
| native executable | `eff14352fd3a283b9d0dfcf5e2b0a8bb096a4f1a127290bd74ea80ea4ad1238c` |
| fixture bytes | `3762540394238820ad47ca7ce4f8fc5b140e11abc0d52458a2451a4fe588f160` |
| captured `assessment.json` | `4ff80c33a14bd56cc46772a32d3cd2e031821aaad606fa3899faf4c0e55ec420` |
| separate raw-record/provenance audit | `5e756bd39e502e795eb2af61b34ecd76b2566325393d1319ef68ac3b6b47aa71` |
| independent native-contract command evidence | `ac016e512f439cbc52b7c4d36b619f89a82a6c08984e6590aa701b81d10184f4` |
| sampled actual-record negative evidence | `7fcab3bcafb1be82618333d5970fb0af3af50149a52b103beeeecec3b02c1105` |

The separate audit/script remain at
`/tmp/rheon-i1-validator-independent-audit.json` and `.py`; native-contract
command evidence/stdout/stderr at
`/tmp/rheon-i1-validator-independent-native-contracts.{json,stdout,stderr}`;
negative evidence/script at `/tmp/rheon-i1-validator-independent-negatives.json`
and `.py`. Raw archives, binaries, records, command streams and assessments
remain external and unmodified. No numerical payload is committed.

### Numerical observations and controls

All 18 records pass the independent all-row physical residual, every-cell
divergence/compatibility, no-slip/full-boundary trace, geometry, forcing, prescribed
reaction/site, force/moment/work/dissipation, wall pressure/gradient and separate
physical pressure/shear load checks. The 19-unknown no-body case additionally
matches the separate rational solve. Body/no-body cases with 669/694 unknowns
are all-row checked, not claimed rational-solved. All 34 native-exported
constitutive controls per record pass independent geometry, pressure/shear,
hydrostatic, gauge-offset, affine trace, rotation/translation, channel work and
sign expectations. The controls remain analytical constitutive diagnostics,
separate from actual solved fixed-body records.

Maximum momentum defect is `2.2134156757881024e-16 N`; maximum physical divergence
is `1.2464495005604731e-15 s^-1`. MMS momentum defect is
`3.364080335042506e-17 N`, divergence `1.257314531080602e-22 s^-1`.
Coarse MMS volume-weighted velocity and pressure L2 errors are
`2.52605341465863e-08` and `2.3611373596309016e-07` respectively; these are
single-grid dimensional observations, not fine-grid V07 orders or acceptance.

Decoded native velocity/pressure differences from their expected linear
relations are exactly zero for reversal, doubled speed, doubled viscosity,
doubled density and shifted moment reference: reversal negates both fields;
doubled speed doubles both; doubled viscosity preserves velocity and doubles
pressure; density/reference preserve both. This is finite-case observation,
not a new comparison threshold or arbitrary conditioning guarantee. All-axis
and oblique actual solutions also passed the same independent row/load checks.

The straight-case reconstructed physical stream force is
`-0.0974765752821589 N`; algebraic body force is `-0.09124856245761553 N`.
Their difference is **6.825327%** relative to the algebraic force. V05 fine-grid
surface/control-volume agreement and N3 continuum force accuracy remain pending;
algebraic conservation does not excuse or erase this coarse trace discrepancy.

Separately ran actual native release contracts from the archived source with
sanitized pinned build environment:
`cargo test --locked --release --no-default-features --test submarine_stokes_contract`.
**13 passed, zero failed/ignored**, including actual-body drag/reversal, all-axis
uniform flow with nonzero pressure datum, tiny MMS with order-one pressure,
corner discrimination, cap/geometry checks, complete cancellation/invalid-data
publication preservation and partial reconstruction/refinement rollback.
Contract assertions are native implementation tests; independent raw-record
oracles remain the deciding numerical evidence. The previously reviewed
31 normal/31 optimized validator harness tests bind unchanged validator source.

Nine one-condition mutations of the actual accepted `tiny-straight` record hit
their intended diagnostics: active velocity `V01-momentum`, body trace
`V02-body-no-slip`, corner location `reaction-sites`, prior outer-force sign
`V04-outer_force`, wall normal `wall-patch-geometry`, wall pressure
`wall-pressure-stencil`, gradient `wall-gradient-stencil`, changed dissipation
`V04-dissipation` and omitted Couette patch shear `V06-patch-shear_force`.
Captured records remain intact; mutation evidence covers these sampled failures,
not complete corruption detection. Acceptance gates were not weakened.

### Complete fresh record integrity index

Paths are relative to the fresh capture root above.

| Native case (`cases/CASE/record.json`) | SHA256 |
| --- | --- |
| `exact-no-body` | `320c86f6ef92dccf5ffd975e383a26de1478acb7896fc37df04ba9b560766849` |
| `no-body-0-+1` | `8e88da08abeb51f79a95725a4dedf4abd43396ffde3358c44668226a6e3fe82f` |
| `no-body-0--1` | `a43fc8ee139398765fda75dc91bd5286fb2e48710d8bb397b788d2c0060f1ba2` |
| `no-body-1-+1` | `8a7e5ec737bd44c8250dc89b7824bd3a8bfb0323831c0a640499036dc20dad22` |
| `no-body-1--1` | `da58b2c9ac9517a3211a533923c895b3f626340d978075490d8fe9f5f8a96ec0` |
| `no-body-2-+1` | `1a757ae57815fb0e6b6518320bc6a46c0edb97de6acbcee0b1eb7b6639047f14` |
| `no-body-2--1` | `45e910f6b6b06f4f6efac0fd387b8ed80015a5dcaff1bc555905ca1ea6c43e41` |
| `tiny-axis-1` | `888c1d9124560e72bb7eecfcfff00a2700e5da42503ac3830f2a71c4a8156c9e` |
| `tiny-axis-2` | `d2b0e9e5bacceafad00d9f257dba92407025c02f6a98ac4dfbdaeedf42c9fbc9` |
| `tiny-double-mu` | `6031e5024b38e98045ab5a076ba65e4749596fb4b0c20eb14bd0edcd7291d4ef` |
| `tiny-double-rho` | `ef5984dcdade311605ccda748dae896a607d96759a32a8d36b9a3bc9cb3b9a95` |
| `tiny-double-speed` | `e513072b6422ded0b8d9df04469b50d844442a1e5563454bccdf35a72538e75c` |
| `tiny-mms` | `66b7c7ba7a67ce2a6681bc6fb84659c2d0102e93db3da350f07b579fc5fbebca` |
| `tiny-oblique` | `3a9b691df0b8bb0e8c045267b3690ae9082724fc8bacdd01c9650a39242c9cf7` |
| `tiny-rest` | `3ff97779b4aba90820197a883b72c4e93997c786622c74bbbf5b7e4c92681887` |
| `tiny-reverse` | `df246a18f32d3f14065b256d4780db5d8023ea5861b29621f9b63c8bbee90582` |
| `tiny-shifted-reference` | `fb530e80c0154e573914ffac0085ff79588dbb3577202ba80a2d4252468d0784` |
| `tiny-straight` | `6e9bae2d99af34a235644714e76b01a42b0d73db2cd443a02ffefe90fab07274` |

### Independent review and bounded N1/N2 decision

Final independent GPT-6.1 Sol High numerical/provenance review of this fresh
run returned **PASS**, with no blocker for the admitted tiny N1/N2 boundary.
The reviewer separately checked all 18 raw records/612 controls, the observed
origin pin, every tracked blob/mode, exact 21 command/stream bindings, every
output/index hash and the recomputed assessment. Own axis/load checks and six
additional actual-record mutations passed; it checked root's 13-contract command
streams/hashes without claiming a duplicate Cargo run. Its external assessment
is `/tmp/rheon-i1-agent-independent-recheck-d7a3c01-20261014/assessment.json`,
SHA256 `e8148677753c8427cf77a30fbe0147c0e92b371255de0aea538064ed6f7bbdfc`;
mutation evidence is `/tmp/rheon-i1-agent-negative-recheck-d7a3c01-20261014.json`,
SHA256 `d4b825970b510f4820ff079bc0cbf134fc09c6defff4272152a5cba7be7b4bf9`.

**Independent decision: N1 PASS and N2 PASS** for source `d7a3c01` under the
unchanged validator `dedf04f`, admitted sector profile and fixture digest, for
the tiny fixed aligned-body steady creeping-Stokes **discrete verification
boundary only**. Complete accepted native fields, represented no-slip/full outer
Dirichlet rows, reactions/sites/work and pressure/shear reconstructed loads have
actual independently checked evidence; the analytic constitutive controls remain
separately labeled. Parent retains canonical ledger and
integration authority. This does not accept N3 mesh/domain/fine-MMS/load accuracy,
N7 experimental rights/model/uncertainty, any later slice, full submarine physics,
recording/Tuldok promotion or restart. The unmodified captured assessment keeps
its template pending N1/N2 values; this report separately records the deciding
complete controls and review, without rewriting evidence.

Reproduce using the exact validator subject, source object and a new absent path:

```sh
source /workspace/rheon-setup/env.sh
python3 validation/submarine/run.py --suite stokes \
  --source d7a3c01fbab2d78e5404afb84f15cf2cf17505fa \
  --native-contract-sha256 3762540394238820ad47ca7ce4f8fc5b140e11abc0d52458a2451a4fe588f160 \
  --output /tmp/rheon-i1-new-independent-capture
```

For each fresh or retained actual record, use `--input`, the corresponding
`--origin` and separately observed `--origin-sha256` with the same source;
write each recheck to a new absent directory. A copied correct field cannot
establish execution provenance. Any refusal preserves its terminal origin,
actual output prefix and command streams; it must not be resumed or marked
complete without a fresh authorized source/output subject.
