# Native uniform forcing of the advancing viscous extrusion

This local successor extends review checkpoint `35b00247` through compiled
native source `5bb0b5d538ac0b1707142cb9691e61994fd68b8e` (tree
`79bdb50c5f98be9c7c9e86df05bc34f3165d3aeb`). Its focused contracts and first
536-state export are frozen in `6458b68`. The independent readers and this
chapter are local, uncommitted work while the parent resolves Git identity.
Neither this source checkpoint nor its native tests imply full temporal
qualification: the original reversed-load geometry refinement band fails.

## Public owner and physical rule

`CoupledDiscreteFlow::new_forced_extruded` reserves 474,768 declared bytes,
compared with 470,992 for the unforced extrusion and 450,320 for the planar
owner. `nominal_forced_extruded_bytes` exposes the checked reservation. The
additional 3,776 bytes account for bounded force scratch, with no additional
accepted owner, growing step history or heap snapshot. The original workspaces,
geometry candidate and common publication barrier remain authoritative.

Call `step_extruded_with_forces(h, forces, cancel)` on that owner. At most sixteen
existing `BodyForce` descriptions supply uniform acceleration or force density
(N/m³, divided by density three). Every region is refused. Slice-order sums and
all native conversions use checked finite arithmetic; nonzero subnormal
intermediates and checked-division failures are refused. No clipping repairs
failed arithmetic. Forces change momentum and pressure, not material mass.

The domain remains `x in R/Z, 0<y<H(x,t), z in R/Z`, both periods one. All fields
are independent of z, with full velocity `(u,v,w)`. Density is three, dynamic
viscosity .05, mass 3.375. The stationary bottom is impermeable with natural
tangential traction; the cap is material with weak natural atmospheric-relative
traction. There are no extrusion end walls or new contact line.

For one interval the body source uses the accepted masses, while strain and
pressure act at the endpoint:

    r_xy = Rxyᵀ(m1 Uxy1 - m0 Uxy0 + cF,xy)
           + h(Kxy1 z1 + B1ᵀ Pi - Rxyᵀ m0 axy),
    Aw xi1 = Rwᵀ(m0 w0) + h Rwᵀ m0 az.

The original changing-mass inertia, actual integrated donor transfers, local
GCL, all moving divergence/acceleration/material rows, pressure space and
physical strain blocks remain. Old-domain body quadrature is first order; it
is not exact source integration along the moving geometry path. Constructor
pressure describes the unforced accepted initial state. The next interval
freshly solves algebraic pressure for its requested force. A pressure change
at that time does not represent an independently evolved pressure state.

`CoupledForcedExtrudedReport` contains the ordinary planar/third/full work
report and `CoupledForceReport`. Force work is signed and is computed before
publication, together with the expected x/z impulses:

    WF = h Σi m0_i U1_i·a,
    E1-E0 + DBE + Dmix + Dmu + Wp + WGCL - WF - WR = 0,
    ΔPx = h·3.375·ax,     ΔPz = h·3.375·az.

The report separates planar, third and total force work. The full-vector work
allowance retains the original factor 128 epsilon and adds the dimensionally
required `abs(WF)` to its scale. All other original numerical gates remain:
true planar Newton rate `1e-13`, full/direct momentum `1e-11`, quadrature
`1e-15`, bounded seven iterations/200 equations and bounded sign roots.
Vertical momentum includes bottom reaction and is not claimed conserved.

The accepted-mass load follows from `Aw 1=Rwᵀ m0`: a constant third field evolves
as `w0+h az`. The retained exact rational two-cell counterexample shows that an
endpoint-mass source produces a nonconstant error. An isolated explicit kick,
old-speed work or omission of `mdot U` is not this coupled equation.

## Native atomicity and retained refusals

Nine focused native contract methods pass. They exercise both the original
initial and nonzero pressure-state fixtures, all eleven force/old/third
cancellation stages and repeated force/quadrature/assembly callbacks from
already accepted forced states. All accepted geometry, velocity, pressure,
time, identity and counters remain bitwise unchanged on refusal; retry bits
match a baseline. Actual late third linear failure and bounded Newton failure
are tested. Force-density/acceleration and ordered split loads agree bitwise.
Empty forcing retains original publication bits and budgets.

The pre-implementation independent flat chart refused its constant-rank
residual. The first native expectation incorrectly inferred that the native
constructor would also refuse; its failed test source and logs are retained.
The native flat-rest constructor actually succeeds. A real .05 gravity step
then returns `WorkFailure` and preserves that accepted state. The corrected
contract requires this actual refusal; no flat-tank gravity benchmark passes,
and neither chart nor work gates were relaxed. The original proposal records
its independent preliminary result and is not overwritten.

## Actual replay, references and failed temporal gate

The example exports forty constructors and 496 accepted steps: both original
fixtures, constant/nonconstant third fields, forward/reversed
`a=±(.0625,-.125,.03125)` and five intervals
`.05,.025,.0125,.00625,.003125` to the common nominal endpoint .1. Actual clocks
are not clamped. The strict reader requires the exported topology, all physical
velocities/pressure/masses, geometry coordinates, owner identity and work/force
reports. It reconstructs all 34 stable and direct momentum equations, both
physical third shear squares, source impulse, signed work and local GCL.
Nineteen actual mutated-input controls are rejected.

The first diagnostic replay finds maximum full momentum rate `1.043e-13`,
direct rate `1.085e-13`, third impulse error `3.557e-16`, total residual-work
ratio .04664, local GCL ratio .02410, quadrature `8.593e-18` and constant-field
acceleration deviation `7.217e-16`. These lie inside the original gates.
Sixteen freshly generated DOP853 references cover every fixture/field/load at
two tolerances. Errors and ratios come from actual exported endpoints; no
producer-supplied convergence claim is accepted.

Full velocity and nonconstant third velocity satisfy the original `1.8..2.2`
first-order ratio band at all five intervals for both signs. Forward geometry
satisfies its original `1.7..2.3` band. Reversed geometry does not: for the
initial fixture its successive max-error ratios are approximately
`2.62875,1.44519,1.75538,1.88450`; the pressure fixture gives
`2.52100,1.48868,1.77222,1.89202`. Constant/nonconstant third cases share this
planar geometry limitation. The strict temporal qualifier refuses. Diagnostic
output retains status `FAIL_ORIGINAL_TEMPORAL_BAND`, while separately reporting
successful physical equations. Its diagnostic flag gathers failed bands; it
cannot turn them into a qualification pass.

Actual error components show that the dominant coordinate switches from cap
height at h=.05 to cap x-position at h=.025. This explains the reported max-norm
switch; it is not a proof that any untested finer interval qualifies. The finer
two ratios lie within the original band, but the five-interval reversed
geometry claim remains unqualified. No interval, failed artifact or gate is
removed to obtain a favorable summary.

The PNG/PDF/GIF and interactive viewer draw the actual published nodes and
triangles, with load, signed force work and ledger metadata. They play recorded
states and do not supply a second simulator. A Node VM checks each canvas
polygon/color and controls against all 536 states. Browser rasterization needs
a working supported Chromium sandbox; a sandbox-disabled launch is not used.

## Proof and simulator limits

No Lean source or theorem is added or requalified. The existing
`Rheon.Physics.force_work_identity` proves an exact-real fixed-mass explicit-kick
identity. It does not prove the accepted-mass moving-domain donor/strain force
rule, its IEEE arithmetic, pressure solve or temporal accuracy. The coupled
work formula above is independently reconstructed numerical evidence under the
declared model, with the observed temporal refusal retained.

This is a spatially fixed, z-invariant fitted two-column viscous family with
uniform forcing. It does not implement localized force quadrature, collision
mesh cut geometry, topology/support changes, wetting/adhesion, variable density
or viscosity, capillarity, general z variation or general surface
reconstruction. Tiny-step/host Newton limits and original Jacobi fixtures
remain. The older viscosity vector-transition and later wet-pressure reader
limits are independent findings; this new replay does not close them. It is
not a fully validated general liquid simulator.
