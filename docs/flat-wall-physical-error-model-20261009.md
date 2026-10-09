# Manufactured-goal error and a face-average wall observable

This analysis uses the retained N6/N9/N12 fields from the refused one-shot at
`3f7d1ce7b39d1f63cba8385e2b0ebb46953d00da`. No physical or reference solve is
performed. The original campaign, its receipts and failures remain unchanged.
New Rust evaluations read those velocity values into explicitly supplied,
unqualified immutable owners. They are observations of existing data.

The symmetric-stress energy context is Batty and Bridson,
[Accurate Viscous Free Surfaces](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf),
sections 5 and 5.1. The reconstruction below is newly derived from this Ritz
flux interpretation; it is not a stencil claimed verbatim from that paper.

## What dominates the measured goal error

The independent reference is
`psi=A B(x-1) B(z-1)(z-1/2) y^4(1-y)^2`, `B(s)=s^6(1-s)^6`,
`A=12012^2/2`, `u*=(psi_y,-psi_x,0)`, and `p*=0` on the declared support.
Exact beta integrals give `F*=(-1,0,0)` N and
`T*=(0,-1/60,-1/2)` Nm. No reference field is passed to the solver.

For each represented z layer, let `Ipsi` be its exact integral of psi at the
x/y potential nodes. Differences of these integrals divided by **exact
geometric face areas** give the true continuum face averages `u_bar*`.
Separately form `C_stored Ipsi` from the independently verified native rounded
coefficients. For any fixed linear wall observable Lh the exact signed split is

`Lh(u_native)-J* = [Lh(u_native)-Lh(C_stored Ipsi)]`
`                 + [Lh(C_stored Ipsi)-Lh(u_bar*)]`
`                 + [Lh(u_bar*)-J*]`.

The first bracket is the measured field/discrete-model goal difference. It
aggregates trial-space, energy/boundary consistency, source and solve errors;
it does not isolate each or certify their propagation. The second is the
stored-area/inverse representation effect, zero at N6/N12 and tiny at N9.
The third is wall-observation and surface-quadrature bias on an independently
known exact face-average field. The decomposition is reference-specific and
can contain cancellations; none of its terms is a general PDE error bound.

For tangential P1 and normal P1, the Fx split is:

| N | Actual retained Fx (N) | Field/model bracket (N) | Reference observation bias (N) |
| --- | ---: | ---: | ---: |
| 6 | -0.214056774 | -0.122412486 | +0.908355713 |
| 9 | -0.305619089 | -0.115123929 | +0.809504840 |
| 12 | -0.548770144 | -0.234222551 | +0.685452408 |

N9 representation effects are separately retained in exact rational output;
their omission from this rounded table is below its displayed precision.
The large observation bias persists even on exact continuum face averages.

At dyadic h define `S_h = h sum_i B(ih) / integral(B)` for the existing x hats.
The exact reference P1 force is `-S_h(1-h)^4`, since its first velocity sample
is the average over the whole wall cell, not its center point value. Thus
`Fx_bias=(1-S_h)+S_h[1-(1-h)^4]`. At N12 these are approximately **+0.0058743 N
x quadrature** and **+0.6795781 N wall secant**. Wall reconstruction dominates
this measured baseline force bias.

N12 Tz is `-0.137192536` Nm. Its reference observation bias is +0.421363102 Nm:
+0.342726204 from the tangential-force bias and +0.078636898 spurious normal
torque on the exact reference averages. Its measured field/model contribution
is -0.058555638 Nm. Normal P2 alone leaves a larger reference normal torque
at these coarse levels; the original failure is retained. Small reduced
residuals and arithmetic source errors cannot resolve these spatial/trace
effects or certify continuous wall derivatives.

## Small optional improvement from two face averages

Let `s=wall_y-y`, stationary tangential trace `v(0)=0`, and first two cell
averages be over `[0,H1]` and `[H1,H2]`, with `0<H1<H2`. For
`v(s)=a s+b s^2`, their moments are

`v1=a H1/2 + b H1^2/3`,
`v2=a(H1+H2)/2 + b(H1^2+H1 H2+H2^2)/3`.

Eliminating b gives the wall derivative

`a_hat=w1 v1+w2 v2`,
`w1=2(H1^2+H1 H2+H2^2)/(H1 H2^2)`, `w2=-2H1/H2^2`.

For uniform cells this is `(7v1-v2)/(2h)`. Ordinary point-center P2 weights
would not be quadratic-exact for face averages. The implementation uses actual
grid planes to construct H1/H2 and their outward arithmetic intervals, including
coordinate-subtraction rounding. It reads the first two existing owned X-face
layers; it consumes no analytic velocity/derivative, new solve or finer state.

For constant mu and a smooth stationary flat no-slip wall, the tangential
derivative of the normal wall trace is zero, so fluid-on-body x traction is
`mu * partial_s v(0)`. This interpretation concerns the averaged tangential
profile on each face; normal Y samples retain their separate point-plane
P1/P2 semantics. Exact x-hat masses and first moments, z-layer moments and all
normal force/torque contributions remain in the observable.

If v is C^3 on `[0,H2]`, its Peano remainder kernel is

`K(t)=w1(H1-t)_+^3/(6H1)`
`     +w2[(H2-t)_+^3-(H1-t)_+^3]/[6(H2-H1)]`.

It is nonpositive and integrates to `-H1 H2/12`. Therefore
`|a_hat-v'(0)| <= H1 H2/12 * sup |v'''|`, conditional on those smooth-average
and zero-trace premises. Input errors additionally contribute
`|w1| epsilon1 + |w2| epsilon2`. On a uniform grid this amplification is
`4 epsilon/h` for equal error bounds, versus `2 epsilon/h` for P1. Quadratic
exactness consequently does not imply second-order loads from unknown native
fields. Their trace errors can still dominate.

For this manufactured profile,
`v(s)=A X Z[-2s+12s^2-24s^3+20s^4-6s^5]`; its third derivative has supremum
`144 A X Z` on `[0,1]`. This supplies a benchmark-specific remainder bound,
which is still coarse at these h. General native smoothness bounds are unknown.

## Actual retained-data evaluations and remaining cancellation

With the new tangential option and unchanged normal P1:

| N | Fx (N) | Ty (Nm) | Tz (Nm) |
| --- | ---: | ---: | ---: |
| 6 | -0.428113547 | -0.014938383 | -0.107028387 |
| 9 | -0.598440128 | -0.003001413 | -0.197347034 |
| 12 | -1.005841242 | -0.015176613 | -0.365728085 |

Rust evaluates all six components on supplied copies of the retained fields.
Independent rational integration verifies that every exact result and native
nearest result lies inside its new arithmetic interval. These evaluations are
post-processing of the original three fields, not a new numerical campaign.

At N12 the new force splits as
`-1.005841242 = -1 -0.438878915 +0.433037673` N.
The small net error is cancellation between two approximately **0.43 N**
contributions. The new observation reduces reference wall bias but amplifies
the existing state discrepancy. New Tz splits as
`-0.365728085 = -0.5 -0.160883820 +0.295155735` Nm.
Changing normal P2 as well gives Tz=-0.274385072 Nm, worse here; neither
normal traction nor its failures are suppressed to improve the displayed goal.

Production change is limited to the opt-in `TwoCellAverageP2` tangential
observable. The original `flat_wall_owned_traction` API keeps its P1 arithmetic
path and values. Matrix, force source, solver, strain rows, mass, pressure and
timestep behavior are unchanged. No general error guarantee, convergence or
physical capability is established. The useful next solver question is the
consistency of its retained energy with face-average wall data; this observation
alone does not settle it. Unknown field/trace/goal propagation and unspecified
application force/torque tolerances still prevent physical qualification.

Exact diagnostic reports use lossless rational interning to fit four fixed
64 KiB output slots, with 1 MiB inputs and 180 s arithmetic-worker deadlines.
The original single 64 KiB monolithic report refused before payload write;
that refusal is preserved. The offline analytical report now uses four
explicitly declared 64 KiB slots (256 KiB aggregate), with the per-slot limit
preserved and lossless interning. This is a separate analytical artifact budget
and does not change the physical campaign's 4 MiB output cap.
