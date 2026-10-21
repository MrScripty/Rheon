# Independent standards and design review

**Result: PASS for the docs-only planning/design deliverable (P2).** This is not
acceptance of I1–I7, numerical claims N1–N7, an implementation admission, or a
Standards Engine certificate. P3 publication and retained-resource evidence remain
the planning lead's separate delivery gate.

Reviewer: independent `/root/standards_review`, GPT-6.1 Sol High, 2026-10-14.
Subject: the authored canonical [plan](../plan.md), [issues](../issues.md),
[ledger](../execution-ledger.md), and the five investigation/handoff reports in
this directory, after the single review round below. Repository base is main
`06eb5daee9e77e2b0843a5ee0a4105f6950e9b1b`; standards source independently
confirmed at `188beda1fa477d21d576c233dd8c7f4c4c267d23`. Review attaches to this
material design and acceptance contract. Adding unchanged review, delivery or
lifecycle evidence does not invalidate it; a material change requires affected
re-review.

## Authority and observations

Read Core/Router and applicable required closure: Planning/Discovery,
Implementation, Development Proportionality, Verification/Independent Oracles,
Architecture, Code Design, Contracts/Evolution, Dependencies, Licensing,
Documentation, Commit, Library and Persistence. The route selects future
language/mechanism guidance again when implementation actually changes those
mechanisms. Serial integration of non-authorizing investigation reports does not
require the Concurrent Plan Integration profile.

Checked actual source assumptions in `src/static_obstacle.rs`,
`src/aligned_strain.rs`, `experiments/aligned_stokes.rs`, related contracts and
the broader-physics pressure-load counterexample. The sealed/free-slip/reduced
operators cannot silently supply the proposed external no-slip solve. Inspected
the pinned PR76 source copy's recording adjacency, limits and pressure-inspection
lifetime; the proposed lane preserves those contracts and their distinct owners.
The repository output policy and Rust 1.92.0/safe-Rust premise were confirmed.

Independently opened the [NASA grid-convergence tutorial](https://www.grc.nasa.gov/www/wind/valid/tutorial/spatconv.html)
and [official SUBOFF experimental-report record](https://ntrl.ntis.gov/NTRL/dashboard/searchResults/titleDetail/ADA359226.xhtml).
They support methodological attribution and report provenance respectively; they
do not supply raw-data permission, matched uncertainty, or new numerical evidence.
Inaccessible full-text references remain explicitly limited in the source register.

## Review round and dispositions

| Finding | Final disposition |
| --- | --- |
| Aggregate numerical status was `pending` despite required N7 being blocked. | Corrected to `blocked`, with N1–N6 pending and planning delivery tracked separately. |
| Validation closure required exact-head review, conflicting with Planning's material-content review rule. | Replaced by review of the identified implementation and material acceptance contract; unchanged evidence additions preserve review validity. |
| Proposed 1.5-ratio refinement did not establish alignment in the supplied centered fixed box. | Replaced by `D/8,D/16,D/32,D/64`, ratio 2, preserving exact centered wall alignment. These overcap studies remain prospective and unsatisfied. |
| I7 numerical promotion needed exact source/consumer migration ownership. | Added exact promoted paths, coordinated experiment removal/import changes, serial shared wiring, conditional migration of both I6 consumers, and no duplicate kernels or unavailable placeholder modules. |
| Experimental rights were unnecessarily a prerequisite to all numerical library/recording promotion. | Promotion now requires the exactly advertised qualified numerical subset; blocked experimental status remains explicit and cannot be relabeled as validated submarine results. |
| Uniform demonstration inputs alone could not execute the manufactured forcing/boundary gate. | Added private nonuniform analytic-boundary/volume-forcing fixtures through the same I1 assembly/solve, without a second solver or generic production seam. |

All findings in this round are resolved. No blocking planning/design finding
remains. The planning lead made all semantic corrections; the reviewer writes
only this report.

## Acceptance reasoning and limits

The first slice is a stationary original aligned cuboid in a finite six-plane
full-vector Dirichlet Stokes box, with no invented physical timestep, convective
wake or experimental hull claim. The tiny `(8,5,5)` grid has 705 full face slots
and 200 cell slots: 905 is an upper bound on the combined active velocity/pressure
DOFs before elimination, below the provisional 4096 cap. Its supplied origin and
spacing give two full fluid-cell layers around the body. The exact active count,
matrix/rank/gauge/conditioning, pressure and shear traces, allocation accounting
and actual execution still require their independent implementation gates. Larger
qualification families cannot be accepted by shrinking them after observing output.

The pressure-load counterexample is correctly deciding: gauge cancellation,
projection residual and work balance cannot establish affine traction accuracy.
Independent native exports, separately captured execution provenance, rational/
analytic expectations, raw stress/geometry records and targeted negative fixtures
can distinguish the named errors. N1/N2 discrete controls do not close N3 continuum
mesh/domain uncertainty; time refinement applies to later transient suites.
Acceleration-pressure recovery and moment-reference formulas have consistent signs.
GCL controls distinguish moving-mesh uniform flow, co-moving no-slip flow and
physical body displacement; rebuilt volumes alone do not count as conservative
remapping.

The complete composed-design probe identifies physical, geometry, trajectory,
recorder, consumer and validator ownership; atomic numerical publication is
separate from durable capture. Strict profile/frame/stage/precision semantics,
actual accepted generations, cold reload, bounded payloads, and independent Tuldok
consumer acceptance remain necessary. A visual archive is not a restart checkpoint.
PR76 source, the separate compressible jet lane, GPL/unlicensed imports and existing
dense3D meanings remain protected. Dependency comparison justifies only the tiny
auditable local proposal; it is not approval of a scalable in-house solver.

Independent supporting checks found no missing local file links in the eight
reviewed Markdown documents and no trailing whitespace. No numerical commands or
existing solver tests were executed as a proxy for new capability acceptance.

## Delivery and remaining work

The docs-only write set is appropriate. Before claiming P3, the planning lead
must review the exact staged and cumulative range against the named base, pass
affected documentation checks, commit/push normally, observe the draft PR, and
record the exact retained head and named feature ref. The retained-protected
worktree contract identifies purpose, parent owner, explicit synchronization and
next disposition; no deletion, rewrite, tag or merge authority follows from this
review. Source tests are not required for this planning-only change.

Remaining implementation dependencies are intentionally visible: affine numerical
derivation and independent native execution; overcap convergence studies; I7
interval-arithmetic placement/access admission and integrated recording API;
actual Tuldok consumer evidence; and matched experimental rights/model/uncertainty.
They do not block publication of this plan, and this PASS closes none of them.
No reviewed action was denied, pending approval, or left with uncertain outcome.
