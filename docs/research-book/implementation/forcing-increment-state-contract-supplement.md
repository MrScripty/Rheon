# Increment contract supplement: pinned refusal and missing work terms

This additive research supplement preserves the proposal at
`fd73aece1dc2a8c0308f9f6f80935276f86a08b5` (tree
`6fdcd991e8837e48388bf8b2d7e4b1351d507765`) and verification at
`46a3d2986bc013ee1787f6af1d82b2d8c6ae50da` (tree
`5aa0c3fc85aef138a9ce39fe0c2dddd0c17cdfc6`). No earlier document, capture,
receipt, source or failed result is rewritten. It addresses the review's
specific capture binding and two analytical coverage gaps. Production, initial
projection, equations, thresholds and trajectories remain unchanged.

Read this with the original
[ephemeral-workspace/stored-endpoint contract](forcing-increment-state-contract.md).
The current endpoint-only contract **and native operation graph** remain the
baseline. Ephemeral compensation can change acceptance/public trajectories;
persistent compensation also changes restart/replay state. Neither has been
implemented or qualified here.

## Pin the pressure-forward second-step refusal

The proposed discriminating comparison must use this exact frozen case, not
an arbitrary one of the five failures:

| Field | Required frozen value |
| --- | --- |
| Fixture / field / load | `pressure_state` / `nonconstant` / `forward` |
| h | `0.00078125` (`1/1280` as real decimal; its binary64 input is pinned in JSON) |
| Accepted stamp | `{id:131, version:1}` |
| Accepted time | `0.00078125`, the time after one accepted step |
| Attempt being examined | Second step; accepted version stays 1 after refusal/retry |
| Body load | One whole-domain `Acceleration` force `(0.0625,-0.125,0.03125)` |
| q0 | `(0.0007793548156308786, 0.5009738578027717, 0.9999999053098506)` |
| eta0 | `(0.9975806549491791, 1.24653375278169, -0.00011317764692525493, 0.0031556959156992053, 0.0030864331245584603, 0.41742838575907687)` |
| Terminal coarse norm | `1.1895444217920825e-13`, above unchanged Newton `1e-13` |
| Controller outcome | Seven corrections / 50 equation calls, `correction_budget_exhausted`; no publication |

The real fraction for h is a description, not permission to replace the stored
binary input with an exact rational time. All runtime values must use the
captured bits. In particular, the source's accepted time and version must not
be inferred from the report metadata in the older research-only native probe:
that probe supplies dummy stamps `81/0 -> 81/1` to isolated planar qualification.
Those are not the actual owner identity or a second-step acceptance report.

[Selected capture and provenance](../../../evidence/forcing-increment-contract-supplement/selected-capture.json)
contains the complete actual accepted version-0 and version-1 publications,
the input selection row (index 3), the first Newton check from version 1,
terminal/refusal events, both post-window observations, and the native fixed
candidate capture. Its provenance pins complete source-file hashes, line
indices and raw selected JSON-line hashes. The independent binding reader
checks these against frozen source bytes; it does not regenerate a trajectory.

The accepted version-1 publication is the input authority for positions,
triangles, periodic indices, **all 16 three-component velocities**, all 16
masses, 16 accepted pressure coefficients, time/stamp and the actual memory
observation. q/eta must agree with the trace and original input row. Its
nonconstant third velocity must not be replaced by the planar probe's zero
third field. Version 0 and the pinned example source define the original
constructor/prefix: `CAP`, bottom `(0,.5,1)`, width 1, density 3, viscosity .05,
`PRESSURE_VELOCITY`, nonconstant `XI`, default fitted/flow settings and id 131.
The public prefix must be reproduced and matched before a future second-step
public-call comparison; a constructor with version-1 velocity is not a restart.

The rejected candidate is a different authority: its 22 terminal unknowns
include six accelerations and 16 **candidate** pressure coefficients. These
must not overwrite accepted pressure. Pin its end_q, endpoint velocity/mass,
stable/direct vectors and both quadrature observations. The native capture
additionally binds R, start/end z, D/B, endpoint forces, face pairs and positive/
negative transfers for an independent fixed-candidate arithmetic comparison.
These are observations after refusal; they never participated in acceptance.
They may be reference inputs for a diagnostic, but a coherent new evaluator
must recompute every dependent operator/force/flux on its chosen endpoint.

The fixed-candidate diagnostic and a future public-call comparison are distinct:

1. **Fixed candidate:** compare the original complete 16/32 equation at the
   pinned q/eta, accepted U/m, load, h and terminal unknown vector. Preserve
   the original coarse 22-row vector bit-for-bit and its refusal. Measure any
   changed arithmetic/endpoint/operator/work independently; do not qualify an
   inertia-only substitution, change the norm definition or promote a captured
   planar pass into a completed step.
2. **Public call:** from the exact original constructor/prefix, first verify
   all version-1 observable bits. Then compare the second call under the
   reviewed evaluator policy, recording actual controller outcome and every
   candidate/third/work/GCL/refinement/range/publication gate. The baseline
   must retain its refusal and unchanged accepted state. If a prototype alters
   the first step or cannot reproduce version 1, it is a different trajectory
   comparison and must be reported separately. No continuation past refusal is
   executed or reclassified by this supplement.

## Endpoint embedding defect: exercise the first ledger term

The original embedding example had e1=0, so it did not independently exercise
the first term in equation (4). The new exact case uses one illustrative
interpolated nodal row `R=(1/2,1/2)`, old coefficients `(1,1)` and stored U0=1.
New binary coefficients are `(1,1+2^-52)`; their exact embedding is `1+e` with
`e=2^-53`, whereas round-to-nearest-even storage gives U1=1. Thus e0=0 and
**e1=e is nonzero**. Take m0=m1=1, h=1/32 and a=-64, so the integrated body-load
term in rN is +2 and external work is -2. There are no transfers or strain/
pressure terms in this small algebra example. It is not a complete native mesh.

The exact nodal residual is rN=2 and the nodal work is 2. The reduced stable
work is `(1+e)(2+e)`. The ledger discrepancy is

    U1*rN-W_R = -3e-e²
              = (-e1*rN) - (U1+e1)*m1*(e1-e0).
                ^ -2e       ^ -e-e²

Both terms are independently nonzero. Exact Fraction checks derive the work
from stored energies, BE loss and signed external work, then reject omission
and sign reversal of each ledger-defect term separately. The native floating embedding is also checked
to produce U1=1. This strengthens an exact-input identity check; it does not
attribute the original five refusals to this defect or change either residual
or work gate.

## GCL work: exercise a nonzero local mass defect

Use the earlier two-node donor example with a deliberate small local mass
defect g, for **both signs** `g=+/-2^-48`:

    m0=(2,3), m1=(1+g,4), U0=(2,-1), U1=(3,1),
    Fplus=2, Fminus=1, net=1,
    G=(g,0), W_G=9g/2.

Both transfers remain positive; their sum still supplies mixing loss 6.
The exact energy change is `1+9g/2`, BE loss 7, nodal residual `(4+3g,2)` and
nodal residual work `14+9g`. Consequently

    DeltaT+D_BE+D_mix+W_G = 14+9g = U1 dot rN.

Dropping W_G leaves ledger error `-9g/2`; reversing its sign leaves `-9g`.
These are tested for positive and negative g. GCL work is signed and is not
physical viscosity or necessarily a loss. This analytical perturbation tests
the work identity independently of native conservation acceptance; it does
not waive the unchanged `128*eps*(m0+m1)` local GCL gate or authorize a mass
defect in a published state.

## Freeze the comparison policy before a new evaluator runs

The [policy manifest](../../../evidence/forcing-increment-contract-supplement/comparison-policy.json)
fixes three distinct stages; this packet executes no new Rheon evaluator:

- **B0 baseline:** frozen native source/operation graph and the observed capture;
  binary64 selected-block solve, existing checked arithmetic, stable/direct
  residuals, native norm and all current budgets/gates.
- **A0 identical fields:** exact Fraction products/sums on the very same captured
  binary U/z/m/R/forces/transfers, with an exact squared-norm comparison. No chart
  solve, new geometry or candidate is permitted. This diagnoses finite arithmetic
  on fixed fields and is not a higher-precision geometry assembly or acceptance.
- **E1 changed chart solution:** a proposed research-only fixed-precision
  increment solve, followed by single rounded binary z, then complete native
  embedding/inspection/flux/force/rate/norm recomputation. All qB/cap/mass/D/B/
  geometry policy stays native. An E1 endpoint that changes must be reported as
  changed fields, not as a same-fields arithmetic comparison. Alternate geometry
  is excluded from this first E comparison.

For E1, the manifest proposes a fixed **106-bit binary significand** with
round-to-nearest, ties-to-even after every scalar add/multiply/divide, and a
normal binary64 magnitude envelope on every nonzero scalar intermediate. The
significand/exponent representation uses a fixed 32-byte slot; it does not
depend on an unbounded multiprecision backend or binary64 low-lane arithmetic.
Use largest-absolute row pivoting with the native last-row tie rule, 15-column
elimination/back-substitution, **zero iterative refinement**, fixed selected
rows/unknown columns, and one round-to-nearest-even conversion to zB, retaining
known binary targets. Reject nonfinite/range/underflow, nonnormal pivots, division
failure or a nonzero multiplication/division rounded to zero. If an implementation
instead decomposes into native float lanes, every lane/intermediate still needs
the existing checked range guards; that is a different policy and requires a
separate manifest. No clipping, tiny-tail discard or fallback precision is allowed.

Keep the original absolute selected-block and full constraints checks on the
rounded zB, all original momentum/work/GCL/refinement/third gates, and seven
corrections/200 calls. Norm/rate assembly in E1 initially uses the unchanged
native operation graph. An exact-input residual on E1's changed fields must be
reported separately as a diagnostic, never silently substituted into Newton.

Reserve one additional **65,536-byte** fixed scratch region in the existing
work/candidate ownership scheme. Its simultaneous scalar inventory is five
15x15 matrices, twelve length-22 vectors, one 24x22 D buffer, nine coordinate
scalars and eight scalar temporaries: 1,934 slots x 32 = 61,888 bytes. Remaining
3,648 bytes cover this region's fixed descriptors/alignment; the future code
must demonstrate that bound. Existing baseline workspace/stack reservations
remain required, and every added simultaneously live object must fit this
region or be explicitly counted and reviewed before execution. There is one
reused region, no recursive/high-precision point copies, retained tail or owner
snapshot. It is discarded between evaluations/publications and fully initialized
before each use. This reservation is a **proposed budget**, not a compiled
sizeof/allocation result or proof that an absent implementation fits.

No E1 comparison is ready to execute until a frozen implementation/source
identity, scalar-operation conformance checks, actual layout/live-memory receipt
and policy identity match this manifest. If any item changes, freeze a new policy
before running it; do not reinterpret this comparison retrospectively. This
supplement freezes the specification for review and does not implement E1.

The pinned case's measured start chart reconstruction gap is exactly zero in
the captured vectors. Nevertheless exact `R*z-U` nodal embedding defects reach
`5.551115123125783e-17` at both endpoints. The projected **rate** contribution
`R^T m1(e1-e0)/h` has Euclidean norm `7.359808400080196e-17` (max component
`5.204170427930421e-17`). The supplement independently computes these exact-input
diagnostics and labels their units. Zero chart gap does not imply zero nodal
embedding defect. This small projected contribution does not establish a cause
or repair of the refused norm `1.1895444217920825e-13`.

## Verification and remaining scope

[Supplement scripts/results](../../../evidence/forcing-increment-contract-supplement/README.md)
add two analytical examples (one with two GCL signs), eight analytical corruption
rejections, and explicit capture-field corruption controls. Ordinary/optimized
Python results are compared; all prior 6,959 tracked files remain byte-for-byte
preserved. No existing eight examples/eleven controls or evidence identities
are rewritten. No Rust/Lean/source/workflow change, heavy build, restart API,
compensated implementation, trajectory advance or numerical floor is claimed.
The original source references, real-algebra proof boundary and physics/temporal
limitations remain those of the original proposal and frozen diagnosis.
