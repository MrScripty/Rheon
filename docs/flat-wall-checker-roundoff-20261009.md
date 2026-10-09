# Exact identities and rounded stored-field moments

This corrects the comparison contract after the once-only physical campaign at
`3f7d1ce7b39d1f63cba8385e2b0ebb46953d00da` refused its N12 comparison. The
original campaign, guard receipt, logs and missing N12 comparison/final summary
remain unchanged. Retained-data comparisons are new checker evaluations; they
cannot relabel that attempted campaign or qualify physical accuracy.

## Two different fields

Let each recorded binary64 coefficient and solved coefficient be interpreted
exactly as a rational. The ideal field is `u*_f = sum_j C_fj q_j`. The native
owned field separately rounds each product and each ordered addition. Its
recorded values `u_f` are exact rationals too, but need not equal `u*_f`.
The checker still requires the recorded native field to match the independently
reproduced binary64 scatter exactly. It still integrates the recorded field
exactly and requires every force/torque component and native rounded result to
lie in the existing outward traction intervals.

P1 has `Tz=(1/2-h)Fx` on each exact N6/N12 stored-coordinate basis column.
Normal P2 has `Tz=-Fx/2` on each exact N6 column. The checker verifies these
column identities and the solved exact rational Cq identity separately. It
does not impose either law at unsupported levels. Linearity then gives the
identity for every exact coefficient combination, with no requirement that
its rounded scatter satisfy it exactly.

## Arithmetic bound, without a fitted tolerance

Premises are finite binary64 round-to-nearest multiplication and addition,
separately rounded in recorded column/term order. This is an arithmetic
comparison under those explicit premises, not a proof of all IEEE behavior.
For finite rounded result x with finite adjacent binary64 neighbors x− and x+,
define the exact rational radius

`r(x) = max(x-x−, x+-x)/2`.

The rounding cell implies `|RN(t)-t| <= r(RN(t))`. The larger half-gap handles
binade boundaries conservatively. At signed zero and subnormal values the
radius is exactly half the subnormal spacing. Nonfinite results or missing
finite neighbors refuse; there is no absolute error floor or fitted multiplier.
Each computed product/addition is independently checked against its exact
rational operands and this radius.

For one face scatter step `p=RN(C_fj q_j)`, `s'=RN(s+p)`, the triangle inequality
gives `|s'-s*'| <= |s-s*| + r(p) + r(s')`. Starting from zero, induction gives
`|u_f-u*_f| <= d_f = sum_steps [r(p)+r(s')]`. Each prefix is checked too.

Let the exact stored-coordinate surface functional be
`L(u)=Tz(u)-a Fx(u)=sum_f l_f u_f`, where a is the applicable coarse factor.
Its coefficients l_f are independently obtained by exact integration of unit
face fields. With `L(u*)=0`, linearity and the triangle inequality give

`|L(u)| = |L(u)-L(u*)| <= sum_f |l_f| d_f`.

The checker checks this bound on the exact reconstructed stored field. It
does not compare the ideal identity to the separately rounded native force
and torque report. Those last rounded numbers can hide the scatter defect.
Synthetic tests reject a perturbation just beyond the derived functional
bound, including both signs, and test underflow, cancellation and overflow
refusal. Direct bound tests run separately from the stricter native scatter
equality gate so they genuinely exercise the bound.

## Retained-data boundary

All new evidence stays outside Git. Each retained comparison keeps the original
record source head, record hashes and binary journal identity, while reporting
the current checker source hash. The comparison worker never launches native
code. Unchanged 180s/16MB managed/1MiB input gates and explicit prewrite output
quotas apply. Native solve binaries, numerical sources, source polynomial,
pressure, stepping and geometry/mesh campaigns are untouched.

The N12 recorded-field defect is `55/2^64` Nm; its exact Cq defect is zero.
Acceptance by the corrected arithmetic comparison would only establish this
bounded comparison, never physical force/torque acceptance. Source-to-field,
wall-trace, goal and spatial uncertainty propagation remains unqualified, and
absolute application tolerances remain unspecified. Physical qualification
and convergence flags remain false.
