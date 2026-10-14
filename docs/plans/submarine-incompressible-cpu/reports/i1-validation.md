# Independent I1 validator implementation and contract proposal

Status: parent-admitted sector correction implemented; native numerical
acceptance unavailable pending revised source/schema handoff. Canonical I1 remains Active; N1/N2, N3 continuum
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

## Parent-admitted sector correction and revised handoff

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

Actual native execution has not occurred for this revised subject. Source/schema
handoff, fresh captures, full case/control execution and independent numerical
rechecks remain pending; N1/N2 are not accepted. N3 continuum mesh/domain and N7
experimental rights/model/uncertainty evidence remain pending. No restart,
signed execution attestation or later-slice claim is made.
