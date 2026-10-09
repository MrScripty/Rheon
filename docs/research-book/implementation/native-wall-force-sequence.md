# One progression: wall traction, exact trace and external force

Read [finite wall friction](column-wall-friction.md), then
[compatible no-slip](column-no-slip.md), then
[uniform forcing and Poiseuille startup](column-poiseuille.md). These three
native operations share one fixed flat-column model and one endpoint basis.
They do not compose a general liquid solver. The original six 3D reference
laboratories remain separate analytical/dense-solve teaching mechanisms.

## What stays fixed, and what each step adds

The slab has fixed height H and area A, periodic lateral directions, zero
normal motion and layer-uniform tangential velocity. Density rho is constant
and positive; dynamic viscosity mu is in Pa s. Liquid dual lengths omega_j
give V_j=A omega_j and m_j=rho V_j. Each viscous edge has stiffness mu A/h.
The retained first/last wet-center value extends constantly to the physical
wall, so a constrained trace can be exact while the continuum field retains
endpoint spatial error. This is not an evolving free-surface/pressure update.

| Step | Native API | New physical term | What the recorded lab compares |
| --- | --- | --- | --- |
| PR22: finite Navier walls | `update_with_walls` | F_b=-beta_b A(u_b-w_b), beta in Pa s/m | Six native cases; both walls have finite friction. Symmetric continuum denominator H+2ell, ell=mu/beta. |
| PR23: exact compatible trace | `update_with_boundaries` | Delta u_b=0 and constraint reaction J_b=-dt F_unconstrained | Two no-slip cases and one lower-Navier/upper-no-slip case. Mixed continuum denominator H+ell, not H+2ell. |
| PR24: one uniform tangential force | `update_with_forcing` | f_body=ma for acceleration; f_body=Vq for force density; include it before reaction | Seven stationary-wall Poiseuille cases, with continuum steady, discrete steady and finite-time discrete references. |

The no-slip API requires the incoming trace to match its prescribed fixed
speed. It is stateless and cannot infer an intercall wall-speed change when
the caller supplies a matching state. Two no-slip walls on one wet node are
refused because their separate reactions are nonunique. A mixed one-node law
can have a unique reaction. No large artificial beta replaces a constraint.

## A common momentum and work language

Let delta be the combined explicit increment before storage rounding, and let
J contain no-slip reaction impulses. The most general step in this progression
is

\[
m_j\delta_j=\Delta t(f_{bulk,j}+f_{Navier,j}+f_{body,j})+J_j.
\]

Free nodes have J=0. Compatible constrained nodes have delta=0, so reaction
cancels every unconstrained contribution, including body force and any Navier
traction sharing that node. Body force is applied to its actual liquid dual
volume even at constrained endpoints; wall reaction is not obtained by dropping
those nodes from the external impulse.

With bulk and relative Navier dissipation measured at the incoming velocity,

\[
\Delta T+\Delta t(P_{bulk}+P_{Navier})-W_{wall}-W_{body,old}
-\tfrac12\delta^TM\delta-W_{round}=0,
\qquad W_{body,old}=\Delta t\sum_j f_{body,j}\cdot u_j.
\]

Momentum changes by body impulse plus total wall impulse plus stored-f32
rounding momentum. Prescribed wall work is wall velocity dotted with delivered
wall impulse and can be negative. Compatible no-slip relative reaction work
J dot (u-w) vanishes without removing bulk dissipation. A stationary wall does
no work while still exchanging momentum. From rest, body old-speed work is
zero on the first step, but the explicit squared-increment energy is positive.
This combined-step convention differs from midpoint applied work in the
separate [box force stage](explicit-forces.md).

The free-row explicit admission is unchanged by a constant body source.
No-slip rows have zero increment; arithmetic checks still apply. All gates and
cancellation callbacks precede output publication, with the same five fixed
scratch vectors and no per-call allocation. These are native runtime contracts,
not an implicit integrator, general geometry theorem or whole-program memory
certificate.

## Keep the reference problems separate

For symmetric finite slip, u(y)=U(y+ell)/(H+2ell). The native discrete
equilibrium is U(jh+ell)/((L-1)h+2ell). For lower slip and an upper no-slip wall,
the respective denominators are H+ell and (L-1)h+ell. Both no-slip Couette
references are Uy/H and Uj/(L-1).

For stationary walls driven by uniform q=G, the continuum steady reference is
Gy(H-y)/(2mu), with flow per width GH³/(12mu). The retained discrete equilibrium
is Gh²j(L-1-j)/(2mu). At fixed G, density changes relaxation but not steady
amplitude; at fixed acceleration, G=rho a changes too. The Poiseuille lab uses
a body source and does not solve a pressure gradient.

Only the forcing lab supplies a closed-form exact-real explicit discrete
startup comparison. Its binary64 evaluation has trigonometric, power and sum
rounding, while the native assembly also rounds before f32 storage. There is
no certified reference-evaluation bound. A steady reference at an early saved
time is not a transient oracle. Distance to equilibrium includes unfinished
relaxation; decreasing endpoint deviation does not establish temporal order.

## Proofs and recorded demos

| Lean source | Checked statement family | Explicit limits |
| --- | --- | --- |
| [WallFriction.lean](../../../proofs/Rheon/WallFriction.lean) | Force/work decomposition, translation, relative loss, explicit work | Assumes the finite step and assembled power; no stencil/IEEE/wetting proof. |
| [NoSlip.lean](../../../proofs/Rheon/NoSlip.lean) | Compatible trace, zero relative reaction work, momentum, constrained work | Compatible fixed trace and per-node step; no wall-speed evolution or geometry proof. |
| [ColumnForcing.lean](../../../proofs/Rheon/ColumnForcing.lean) | Units, body-inclusive reaction, forced momentum/work, parabola/stencil algebra, equilibrium recurrence | Step, internal cancellation and assembled power remain explicit; no spectral-reference/convergence or IEEE proof. |

The integrated native proof source has 57 public theorems and 77 audited
declarations at accepted PR24 source. The older expansion receipt's 42/60
counts remain historical, with their own source identity. Definitions and
generated helpers are not added to the public-theorem count.

The [native lab hub](../../education/native-labs.html) connects all three
recorded packets when supplied to the book builder. Every case/time control
is explicitly snapshot playback. The PR22 plot is its final saved profile;
its CSV contains four saved times. PR23/24 sliders choose five actual saved
profiles. They do not recompute arbitrary parameters or run a live fluid solver.
The builder verifies frozen input bytes and performs zero numerical calls.
A source-only CI build without recorded packets states that they are absent,
rather than silently substituting new runs or fabricated results.

These steps qualify local wall/force mechanisms. Continue to the
[requirements map and next geometry contract](requirements-roadmap.md) before
extending beyond the fixed slab.
