# Experimental aligned-box viscous step

This contract defines a bounded, opt-in, unforced viscous update followed by a
pressure proposal. The experiment lives in `experiments/aligned_stokes.rs`,
outside the production library, and is invoked only by its example and tests.
Its starting source is
`0e13b02294b247d9f59326f98e6233fb9280967e`. It adds a finite-step experiment;
it does not qualify the older liquid transport facade or a continuum solver.

## Domain and stored coefficients

Borrow one immutable `AlignedStrain` and its exact geometry owner. Admission
remains one retained internal grid-aligned box with at least one full fluid
cell of padding. The obstacle is stationary and no-slip; the outer domain is
sealed and free-slip. Use the existing pressure-active face map, stored areas,
wet-cell volumes and stored positive face masses throughout. The object's
constant density and dynamic viscosity supply the material values; both are
strictly positive under the current constructor. There is no second material
parameter or zero-viscosity override. A zero velocity state remains an
admissible identity with positive viscosity.

Caller velocities are full MAC arrays of binary64 values. Shapes must match;
every outer or blocked face must be exactly zero. Inputs and used coefficients
must be finite normal values or zero, with positive denominators. Before any
proposal and again on the final candidate, enclose each wet cell's actual
integrated flux divided by its stored positive volume. Require

    upper(abs(Q_i(u))/V_i) <= divergence_limit,

where the caller supplies a finite nonnegative limit in `1/s`. Check all wet
cells, including every pressure gauge cell. Dry cells are excluded only by
their explicit zero volume. Use one shared oriented area per active face; the
sealed component flux cancellation is then an exact algebraic property of the
incidence, not a floating sum that happens to lie near zero.

The [reconstructed strain derivation](reconstructed-aligned-strain.md) fixes
the normal rows with weight `2V`, engineering shear rows with actual represented
fluid-quadrant weights, reflected corner rows and eliminated stationary wall
traces. Retain included zero rows. Outer-wall shear is analytically eliminated
by sealed-normal and free-slip traces before edge enumeration. Treat the
stored binary64 coefficients and weights as exact real input numbers in the
following algebra; do not infer exact geometric products from their rounded
assembly.

## Viscosity proposal and actual acceptance

Write `M=diag(m_f)`, `K=E^T W E`, and

    D0(u) = sum_r w_r (sum_f E_rf u_f)^2,
    H(u)  = sum_f m_f u_f^2 / 2.

The exact unforced explicit reference is
`v=u-dt*mu*M^-1*K*u`. The numerical proposal may round its gather, transpose
and update. A nearest-rounded row-bound diagnostic does not authorize it.
Independently enclose the bound from the actual stored rows and masses:

    L_r = sum_g abs(E_rg),
    B   = max_f [ (sum_r w_r abs(E_rf) L_r) / m_f ].

Require a finite nonnegative enclosing upper endpoint `B_upper` and an outward
upper bound on `dt*mu*B_upper` at most `2`, for a finite positive requested `dt`.
This supplies the exact-real Euler premise. It does not control the rounding
of the proposed velocity.

For the actual stored proposal `v_hat`, independently enclose

    DeltaH_visc = sum_f (m_f/2) (v_hat_f-u_f) (v_hat_f+u_f)

and require its upper endpoint to be at most zero. This factorized difference
is exactly `H(v_hat)-H(u)` for the represented states. It avoids subtracting
two separately enclosed, almost equal energies. An overlapping sign enclosure
is a refusal, not an energy tolerance.

Energy decrease alone would permit an unrelated damping operation. Therefore
also enclose each coordinate's momentum equation defect and its dimensional
scale:

    e_visc_f = m_f (v_hat_f-u_f) + dt*mu*(K*u)_f,
    C_f = sum_r abs(E_rf*w_r*(E*u)_r),
    s_visc_f = abs(m_f (v_hat_f-u_f)) + dt*mu*C_f.

The caller supplies a finite dimensionless relative defect limit
`0 <= tau_visc <= 1e-8`. Require
`upper(abs(e_visc_f)) <= lower(tau_visc*s_visc_f)` for every active face.
The shared `relative_update_limit` sets both defect limits; its default is
`4096*f64::EPSILON`. The accepted binary64 maximum is `1e-8_f64.next_down()`,
the largest representable value below the exact decimal cap; the nearest
`1e-8` literal lies slightly above it. There is no absolute floor. A provably zero defect and zero scale passes as an
identity; an unresolved zero-scale enclosure refuses. This is a quantified
approximate discrete equation, not a claim that the rounded proposal is the
exact Euler solution. Summing absolute scatter contributions in the scale
keeps it meaningful when different strain rows cancel in the net force.

## Pressure proposal, mass mismatch and qualification

Run existing `StaticObstaclePressure::project` only on private scratch holding
`v_hat`. Its solver settings, iteration limit and cancellation remain explicit
proposal restrictions. Its ordinary residual, divergence and energy budgets
are insufficient to qualify the experiment's caller state.

Let `P_if` be `+1` at a face's head and `-1` at its tail. Thus
`Q(u)=-P(A*u)`, and `g_f=(P^T p)_f=p_head-p_tail`. The exact projection matched
to the **stored** mass has correction `v-dt*(A/m)*g` and pressure matrix
`L=P diag(A^2/m) P^T`. Existing pressure code separately rounds `rho*A*d`,
`rho*d`, and `A/(rho*d)`. Even bit-identical pressure and strain masses do not
establish the real-number identity `m/(rho*d)=A`. Do not use that identity to
declare the existing rounded pressure proposal mass-matched.

Instead, for its actual stored candidate `z_hat` and proposed pressure `p`,
enclose the defect against the stored-mass reference:

    e_pressure_f = m_f (z_hat_f-v_hat_f) + dt*A_f*g_f,
    s_pressure_f = abs(m_f (z_hat_f-v_hat_f)) + abs(dt*A_f*g_f).

Require the analogous coordinate inequality with an explicit dimensionless
`0 <= tau_pressure <= 1e-8`, without an absolute floor. This includes both
arithmetic rounding and the existing metric mismatch. The mismatch is neither
silently discarded nor asserted to be zero. Require finite pressure, valid
gauges, unchanged zero wall traces, and the independent final interval
divergence gate on **all** wet cells.

Finally enclose the actual pressure-stage energy difference

    DeltaH_pressure = sum_f (m_f/2)
        (z_hat_f-v_hat_f) (z_hat_f+v_hat_f)

and require its upper endpoint to be at most zero. Both stages must pass
separately. A pressure energy decrease cannot hide a failed viscous stage.

For clarity, the following exact identities explain what an approximate
pressure solve can and cannot imply. With
`r=P(A*v_hat)/dt-L*p` and the defect above,

    Q(z_hat) = -dt*r - P(A*e_pressure/m),
    H(z_hat)-H(v_hat)+H(z_hat-v_hat)
        = dt*sum_i p_i Q_i(z_hat) + sum_f e_pressure_f*z_hat_f.

Consequently the solver's small rounded residual alone proves neither actual
divergence nor energy decrease. The final flux and energy enclosures are the
acceptance authority. The per-face defect gate binds that accepted state to
the stated numerical momentum equation.

## Arithmetic premise and bounded transaction

The enclosure engine uses closed binary64 intervals and outward rounding at
**every** basic operation. Under round-to-nearest, ties-to-even scalar basic
operations, an endpoint operation is expanded with `next_down`/`next_up`;
product and quotient extrema include all endpoint combinations. Division
requires a denominator interval strictly separated from zero. No fused or
reassociated expression may replace the specified operation sequence.
Exact zero and identity cases may return point intervals only when established
algebraically, such as multiplication by a point zero, adding point zero, or
subtracting equal point inputs. Merely obtaining a rounded zero is not evidence
of an exact identity.

The supported experiment environment is Linux x86_64 under Rust's documented
scalar floating-point semantics: default round-to-nearest and no flush-to-zero
or altered floating-point controls. Refuse unsupported targets, NaN, infinity,
overflow, nonzero multiplication/division underflow to zero, subnormal inputs,
subnormal intermediate results or outward endpoints, invalid divisions and
unresolved required inequalities. Conservative refusals at an interval boundary
are allowed. In particular, blindly widening zero produces subnormal endpoints;
the documented exact identity cases are necessary for the resting state.
[Rust RFC 3514](https://rust-lang.github.io/rfcs/3514-float-semantics.html)
states the basic-operation and floating-environment assumptions;
[Rust's `next_up` and `next_down` documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.next_up)
specifies adjacent representable endpoints. The module can refuse unsupported targets and observed arithmetic-probe
failures, but it does not introspect floating-point control registers or prove
that external code never changes them. The documented standard environment
is an explicit premise; externally altered controls are unsupported. These
are external arithmetic premises, not a Lean proof of the Rust compiler or
machine execution.

Construction checks integer products and sums, fallible allocation and actual
vector capacity payload against a caller cap. Account for the borrowed strain's
reported payload, pressure payload, full MAC proposal arrays and all interval
scratch in the declared combined cap; state explicitly that borrowed geometry,
caller arrays, stack, allocator metadata and process RSS are outside this cap.
Preallocate scratch before stepping. Iteration caps and cancellation bound work;
the workspace and its operator/pressure call paths allocate no heap memory
during a step, change no geometry and preserve the borrowed operator. Caller
cancellation callbacks remain caller code and are outside that allocation claim. On any
refusal, caller velocity and any caller-visible accepted pressure, counter or
report remain unchanged. Publication occurs only after every acceptance gate
and a final cancellation checkpoint. Scratch need not roll back. The wrapper
describes an interval of length `dt`; it does not introduce an independent
simulation clock or silently shorten the requested interval.

## Units, evidence and proof boundary

| Quantity | Units |
| --- | --- |
| `u`, `v_hat`, `z_hat` | `m/s` |
| `E`, `w`, `m` | `1/m`, `m^3`, `kg` |
| `rho`, `mu`, `dt` | `kg/m^3`, `Pa*s`, `s` |
| `K*u`, `-mu*K*u` | `m^2/s`, `N` |
| `mu*D0`, `H`, `B` | `W`, `J`, `m/kg` |
| `Q`, `Q/V`, `p`, `r` | `m^3/s`, `1/s`, `Pa`, `m^3/s^2` |
| momentum defects and their scales | `kg*m/s` |

The exact-real Euler coefficient theorem already supplies a conditional
reference result. New proof obligations are the stored-mass pressure defect
identities above and composition: sound enclosures of both energy differences
with nonpositive upper endpoints imply `H(z_hat)<=H(u)`. Sound enclosures of
flux ratios and momentum defects imply the stated tolerance inequalities.
Enclosure containment is an explicit premise in Lean; interval implementation,
native source correspondence and the floating-point environment require their
own reviewed tests and qualification evidence. No exact theorem or unenclosed
numeric `B=17` authorizes an IEEE step by itself.

A planned independent rational qualification fixture uses the unit `3^3` grid,
center box `[1,2]^3`, `rho=mu=1`, and `dt=1/32`. All face velocities are zero
except `u_x(1,0,0)=1/2`, `u_x(1,1,0)=-1/2`, `u_y(0,1,0)=-1/2`, and
`u_y(1,1,0)=1/2`, where coordinates index the respective MAC face array.
This curl has zero initial flux in all 26 wet cells. Its exact targets are
initial energy `1/2`, viscous energy
`679/2048`, final projected energy `303995/917504`, and maximum post-viscosity
integrated flux `1/64`. The nonzero intermediate flux requires a genuine
pressure correction. These targets must be reproduced from the fixture and
compared with the experiment's outward enclosures; listing them is not a test
receipt. Rest, refusal and rollback cases are additional obligations.

[Batty and Bridson (2008)](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf),
section 3 (PDF page 4), describes splitting viscosity and pressure; section 5,
equations (9)–(11) (PDF page 5), derives symmetric-strain dissipation and an
implicit variational update. This experiment's bounded explicit integration
and acceptance gates are new choices.
[Batty, Bertails and Bridson (2007)](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf),
equations (4)–(6) (PDF page 4), supplies the mass-weighted kinetic projection
principle. Neither paper supplies these interval gates or the stored-mass
defect qualification.

An implicit follow-on would need its own solver and residual-work qualification:
for `r_v=M(v-u)+dt*mu*K*v`, the exact identity is
`H(v)-H(u)+H(v-u)+dt*mu*D0(v)=sum_f r_v_f*v_f`.
A small residual norm alone does not imply nonincrease. This first experiment
therefore remains explicit. It excludes advection, forcing, transport,
capillarity, free surfaces, variable materials, moving solids and global PDE
convergence claims.
