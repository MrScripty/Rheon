# Why the bounded viscous wrench reverses the analytic torque

This is research from preserved commit
`7d3c8df2d9149e5cd643a2af1f29f2ccf94bfb7e`, not a production correction.
The native aligned strain, rigid lift, masses, pressure, geometry and old
expected values remain unchanged. Pressure coupling and stepping are disabled.
The physical load failure is retained; no accuracy qualification is gained here.

## Reproduced failure and its components

The unchanged control is the stationary no-slip, smooth solenoidal field
`u=(partial_y psi,-partial_x psi,0)` with
`psi=225*(1+(x-3/2))*[(x-1)(x-2)(y-1)(y-2)(z-1)(z-2)]^2`.
A smooth cutoff outside the sample plateau preserves the solid stresses and all
active samples while making the sealed outer traces stationary. This does not
make the point-sampled MAC field discretely divergence-free. No pressure solve,
steady momentum balance or global PDE claim is assumed.

For the unit solid `[1,2]^3`, `mu=1`, reference `(3/2,3/2,3/2)`, exact geometric
surface integration gives `F_y=-1/2`, `tau_z=-1`. The six-cell grid with `h=1/2`
gives native `F_y=-72225/65536`, `tau_z=8775/8192`.

The table separates observable ablations; they are not corrected operators.
Exact fractions, freshly produced native records and independent derivation are
retained outside Git with their source and binary identities.

| Observable | Force y | Torque z |
| --- | ---: | ---: |
| Exact continuum surface integral | `-1/2` | `-1` |
| Exact wall stress on the original flat face cubature nodes | `-2025/4096` | `-2025/2048` |
| Original secant shear with moments at wall points | `-199125/131072` | `-43875/32768` |
| Original flat rows with the actual restored rigid lift | `-199125/131072` | `-37125/16384` |
| Original cell-normal row contribution | `54675/131072` | `54675/16384` |
| Original assembled native solid wrench | `-72225/65536` | `8775/8192` |

The cell-normal rows take `(U_plus-U_minus)/h` with weight `2V`. At a solid wall,
the missing stationary face is eliminated; the surviving normal face is a full
cell away. The polynomial's true `partial_n u_n` vanishes on every solid face,
but its normal velocity at distance `h` does not vanish. Thus the finite normal
strain is nonzero and its virtual boundary reaction has a positive torque
`54675/16384`, larger than the negative flat-row torque. This explains the
sign reversal exactly, including the cyclic xy/yz/zx controls.

There is no common sign error: the solid outward normal, `omega cross (x-c)`,
transpose sign and right-handed moment convention agree with direct geometric
integration and the exact finite action. Actual fluid, solid and outer resultants
balance in the stored-coefficient unit model, and the stationary force-work
identity is unchanged. A global sign flip would break those checks.

Flat shear has additional errors. Its center-to-wall secant is not the analytic
wall derivative. On this fixture the wall cubature applied to exact derivatives
has about 1.1% force/torque error, much smaller than the secant/lift effects.
The restored shear lift includes the derivative of the normal virtual trace:
this is required to annihilate a common rigid motion. Its torque therefore uses
the equivalent sample lever, not only the wall-point lever. Moving it to the wall
adds `30375/32768` to the torque, worsening the total. That move deletes part of
the virtual strain derivative; it is not a valid isolated correction.

Actual quadrant/volume weights and the exact-product-volume identity are exact
dyadic quantities in this unit fixture. There is no corner contribution because
these particular corner samples vanish. Changing corner signs, snapping wall
locations, inventing omitted rows, or changing geometric area conventions cannot
explain or repair this failure.

## Why weak boundary reaction is not an exact stress trace

The finite reaction is the negative derivative of
`Phi=(mu/2)*(Eu+Cs*xi+Co*eta)^T W (Eu+Cs*xi+Co*eta)` with respect to a boundary
test twist while the active fluid samples are held fixed. This selects a
particular nonrigid extension of that test into neighboring fluid volumes.

Let `b` denote that continuum extension, equal to the solid rigid test on the
solid wall and zero on the other boundary. With `n_fluid=-n_solid` there,
integration by parts gives

```
-integral_fluid tau : grad(b)
 = integral_solidwall (tau n_solid) dot b
   + integral_fluid div(tau) dot b.
```

The volume term need not vanish for this manufactured field. It is not a
steady unforced Stokes solution; no missing pressure or inertia balance is
silently assumed. Thus improving quadrature of the weak reaction alone does
not establish its equality to an arbitrary continuum wall stress integral.
For a scalar illustration, `u(r)=r^2`, `b(r)=1-r/h`, `0<=r<=h` has zero wall
derivative, but the exact normal-energy boundary derivative is `2*mu*A*h`.
It is the volume residual term, even with exact polynomial integration.
This distinction does not excuse the recorded load error or approve coupling.

The symmetric-deformation energy and variational boundary work are motivated
by [Batty and Bridson (2008)](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf)
and [Batty and Bridson (2010, revised 2011)](https://arxiv.org/pdf/1010.2832),
especially the latter's solid-boundary treatment in section 6.2. The forensic
decomposition, no-go statements and proposed trace contract here are new Rheon
research, not stencils claimed verbatim from those papers.

## Corrections that are ruled out

With old fluid resultant `F_f(u)` and outer wrench `g_o(u)` frozen, exact rigid
closure fixes `g_s(u)=-R_f^T F_f(u)-g_o(u)`. Any nonzero solid-only change then
breaks force or torque closure. Likewise, equality of every old virtual-work
pairing fixes each component of the old wrench. Both statements are conditional
exact-real theorems in `research/viscous-wrench-consistency/FixedLoadLimits.lean`.
They do not prove floating-point refinement or continuum accuracy.

Deleting normal reactions leaves `tau_z=-37125/16384` and still-inaccurate force.
Moving only shear moments to the wall breaks rigid reproduction. Changing
weights changes the energy, fluid action and bound and would require a new
derivation. Redistributing missing torque to the stationary outer wall would
invent nonlocal boundary work. None is accepted.

A plausible two-center quadratic wall-derivative estimate also fails this fixed
control. For trace `B` and outward distances `0<d1<d2`, its exact coefficients are

```
q'(0) = -(d1+d2)/(d1*d2)*B
        + d2/[d1*(d2-d1)]*U1 - d1/[d2*(d2-d1)]*U2.
```

At `d1=1/4,d2=3/4,B=0`, this is `6U1-(2/3)U2`. It reproduces affine and quadratic
profiles but gives `F_y=26325/32768`, `tau_z=-22275/16384` when paired with the
original flat cubature. The force sign is wrong. It is rejected as a repair;
neither the analytic values nor the sample cutoff is changed to make it pass.

There is a stronger information obstruction. In the admitted three-cell unit
fixture, all 48 active component samples of this smooth nonzero-load control
are exactly zero. The zero continuum field has the same samples and stationary
boundary traces but zero load. Therefore no function of the unchanged sampled
data, even nonlinear, can recover the exact continuum load for both. The Lean
sampling theorem states this with explicit equal-sample/different-load premises;
the geometric/polynomial premises are supplied by the independent exact lab.

## Smallest justified next feature and decision gate

There is no justified scalar/sign patch to the old wrench. The smallest next
feature is an isolated **boundary trace consistency contract**, before any
advancing coupling. It must declare how wall-normal velocity derivatives are
determined. Extra boundary derivative information, an enriched reconstruction
with enough independent support, or a restricted polynomial class with an
explicit error certificate is needed; unchanged samples alone cannot provide
the requested general continuum claim.

The concrete first research interface should be bounded planar-wall gradient
traces with a declared reconstruction class and actual represented distances.
Its symmetric stress is `tau=mu*(G+G^T)` and its solid traction is `tau*n_solid`.
Use positive wall cubature with separately stated exactness for both force and
moment integrands. Reject insufficient support; keep the old one-cell aliasing
failure as a failure. This interface first measures the wall load; it does not
consume old variational reactions as physical loads or authorize a body update.

Before identifying a new variational reaction with this wall load, derive the
same reconstruction's positive volume strain energy and its matched transpose,
including a discrete Green identity and the volume residual work of each
virtual boundary extension. Specify whether the load contains this residual
or whether a consistent momentum balance removes it. A supplied boundary jet
alone does not prove force/work closure with the old fluid action. Neither a
new pressure closure nor a broad solver is selected here.

Required controls before a production proposal:

1. Affine and quadratic flat-wall derivative reproduction at nonmidpoint
   distances, with explicit unsupported one-sample cases; smooth no-slip,
   solenoidal normal-trace conditions are premises, not inferred from MAC points.
2. All reflected flat/corner permutations and anisotropic lengths; common rigid
   motion cancels both shear derivatives and reference shifts give
   `tau_new=tau-shift cross F`. No wall-point-only torque substitution.
3. Positive strain weights and a derived nonnegative quadratic energy;
   every fluid and boundary action comes from its declared transpose. Test
   arbitrary cross-component fields and independent virtual-work pairings.
4. An explicit discrete boundary/divergence Green identity and the corresponding
   total force/torque balance, including any declared volume residual term.
5. The original one- and two-cell polynomial controls, unchanged. Keep all
   physical errors and signs visible; choose and review an accuracy contract
   before declaring a new result qualified. A correct torque sign alone fails
   the force control, as the rejected quadratic estimate demonstrates.
6. New coefficient/memory bounds if rows/support change; stored masses and the
   pressure face space stay preserved. Any stability estimate remains unenclosed
   and cannot authorize stepping. Native comparisons and independent design
   review precede production changes.

The blocking input is the boundary-trace representation and permitted additional
information/accuracy contract. This investigation does not silently choose it.
Independent design review must approve this research path before production work.
