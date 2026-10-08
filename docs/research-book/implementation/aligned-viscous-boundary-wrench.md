# Stationary aligned variational viscous boundary wrench

This new implementation restores virtual boundary traces to the reconstructed
aligned strain. It is not recovered source. The admitted domain remains one
exactly grid-aligned retained internal box with at least one full fluid-cell
padding, stationary no-slip solid, stationary sealed/free-slip outer walls and
fully wet constant positive density/viscosity. No pressure is accepted or
computed; no velocity, geometry, body, time or stored mass is advanced.

`AlignedViscousBoundaryWrench` borrows the existing immutable `AlignedStrain`.
It retains the existing active f64 face space, `rho*A*d` masses, row order,
normal weights `2V`, actual fluid-sector shear weights and all empty rows.
All sample/trace positions are represented grid positions. There is no wall
snapping. The result is a **variational discrete generalized wrench**, with
force xyz and torque xyz about a declared world reference. It is not an exact
geometric stress integral, rowwise point traction or P1 facet load. No local
traction distribution is inferred from a six-component total.

## Reconstructed virtual traces

For reference `c`, a test twist `xi=(V,omega)` gives
`R(x)=V+omega cross (x-c)`. All six basis columns use this same reference;
solid and outer lifts are separate. Native construction stores two six-column
vectors `Cs_r,Co_r` per retained row.

For a wet cell-normal row in direction `a`, let actual lower/upper face values
be `U_minus,U_plus` and represented width `h`. The restored expression is
`(U_plus-U_minus)/h`. Each surviving face uses the unchanged existing active
coefficient. Each eliminated face is classified as outer (normal grid index
`0` or `N_a`) or obstacle (admitted blocked internal face), evaluated at its
actual face point and assigned signed coefficient `-/+ 1/h` to the appropriate
rigid basis. A fluid cell between solid and outer can have no surviving terms
but nonzero opposite `Cs` and `Co`; that row remains present with weight `2V`.

An interior four-fluid-sector edge restores no boundary traces. Its existing
center-to-center component differences and weights remain unchanged. Outer
edge shear is still analytically eliminated by the admitted sealed normal and
free-slip condition; no no-slip tangential outer trace is inserted. `Co` here
contains only the represented normal-trace reaction. Arbitrary prescribed
moving/free-slip outer-boundary physics is not exposed by this stationary API.

For a flat obstacle wall, let tangential component `a` be sampled at `x_U`,
normal direction be `b`, fluid side sign `sigma`, and actual center-to-wall
distance `delta`. Let `x_w` be the projection to that wall,
`T=R_a(x_w)` and `q=partial_b R_a`. Then

```
R_a(x_U) = T + sigma*delta*q
gamma = sigma*(U-T)/delta + partial_a R_b
      = sigma*(U-R_a(x_U))/delta,
```

since the symmetric shear of a rigid field is zero:
`partial_a R_b=-q`. Thus the derived solid lift is
`-sigma*R_a(x_U)/delta`, not tangential wall-speed subtraction alone. On
`y=0`, `u=-Omega*y,v=Omega*x`, this restores the missing `partial_x v=Omega`
and cancels the spurious `-Omega` shear. Its angular derivative also contributes
to the row torque, so a row torque is not generally tangential force times the
wall-point lever. Full assembled generalized work is the declared quantity.

For a convex obstacle corner, choose consistently reflected coordinates with
fluid signs `sigma_a,sigma_b` and the solid in the negative/negative quadrant.
Use the same rigid affine background in all three fluid sectors, plus residual
components

```
u_a(x) = R_a(x) + (U-R_a(x_U))*max(sigma_b*y,0)/ell_b
u_b(x) = R_b(x) + (V-R_b(x_V))*max(sigma_a*x,0)/ell_a.
```

Here `x,y` are actual coordinates relative to the corner planes and `ell_a,
ell_b` are positive represented sample distances. The residual vanishes on
each solid half-wall, interpolates the surviving sample and is continuous
across fluid-sector boundaries. Differentiation within each actual sector
gives exactly that sector's existing reflected coefficients applied to
`U-R_a(x_U),V-R_b(x_V)`. Therefore
`Cs_r=-sum_f E_rf R_component(f)(x_f)` for these **derived flat/corner** rows.
This formula is not used to define a lift for arbitrary rows merely to force
closure. It is a new Rheon derivation, not a stencil quoted from Batty.

Sector conductance remains `sum(v_sector)/delta^2`. Geometric area equality
requires a separate exact-product-volume premise. The construction does not
identify separately rounded products with exact geometry.

## Finite force, work, torque and closure

Let `s=Eu+Cs*xi+Co*eta`, `W=diag(w_r)` and
`Phi=(mu/2)*sum_r w_r*s_r^2`. In exact-real finite algebra,

```
f   = -mu E^T W s
g_s = -mu Cs^T W s
g_o = -mu Co^T W s
u dot f + xi dot g_s + eta dot g_o = -mu sum_r w_r*s_r^2.
```

Nonnegative weights and viscosity give nonpositive total power. The native
physical action fixes `xi=eta=0`, calls the unchanged `AlignedStrain::diagnose`
for fluid force/work and computes only the new transposes. The virtual-work
API pairs test twists with this stationary wrench and independently sums
`-mu*sum w*(Eu)*(Cs*xi+Co*eta)`; tests do not become physical boundary inputs.

For common rigid sample map `R_f`, the explicit extra premise
`E R_f+Cs+Co=0` implies `R_f^T f+g_s+g_o=0`. Native diagnostics report the
actual maximum row residual per mode and the assembled force/torque balance
defect. No exact conservation or floating-point enclosure is inferred from
small values. The fixed sealed outer walls make global rotating fluid
incompatible; use local compatible rotation patches to test reconstruction.

Changing reference from `c` to `c+a` gives
`tau_new=tau-a cross F`, `V_new=V+omega cross a` and preserves pairing.
Torque is world torque about that reference. Density is retained through the
borrowed mass identity; at fixed viscosity and supplied velocity it does not
multiply the stationary stress reaction.

## Arithmetic, bounds and qualification

Native arithmetic uses the existing checked finite/normal-or-zero kernel and
compensated reductions. Constructor lift coefficients use documented nearest
binary64 multiplication and sequential addition; no FMA is assumed. Exact-real
conditional Lean statements interpret stored coefficients as real constants,
and separately state geometric reconstruction/closure premises. They do not
prove IEEE implementation refinement or PDE convergence.

The constructor limit covers the existing operator's accounted payload plus
actual capacity of two six-column vectors per retained row, without enlarging
limits. Borrowed geometry, caller fields, stack, allocator metadata and process
RSS are excluded. Actions allocate no heap payload. Cancellation/arithmetic
failure leaves immutable owners unchanged; caller force output may be partial
as in the existing action API. No nearest-rounded force/work/closure diagnostic
or existing unenclosed stability estimate authorizes stepping.

Fresh qualification must compare actual Rust coefficients/actions with an
independently rebuilt geometry/Fraction oracle, including reflections,
anisotropy, nonmidpoint walls, cross components, affine strain, compatible
local rotation, empty rows and shifted torque references. Independent physical
controls must distinguish this variational wrench from a wall-point-only
traction rule. Source/binary/proof/receipt identities are frozen independently;
old pass counts are not evidence. The implementation review must state physical
stress-accuracy limits in addition to algebraic work results.

## Original research and exclusions

[Batty and Bridson (2008)](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf),
sections 4–5, derive symmetric-deformation dissipation with compatible samples
and volume weights; their rigid-rotation discussion motivates restoring both
shear derivatives. The reflected residual lift above remains a new derivation.

[Batty and Bridson (2010, revised 2011)](https://arxiv.org/pdf/1010.2832),
section 6.2, treats boundary velocity through consistent discrete boundary work.
This supports adjoint boundary work, not an assertion of exact traction accuracy.

Pressure trace/flux closure remains held: adjacent-cell wall pressure gives
twice the physical hydrostatic force on the unit center cube. Separate
affine-exact pressure research must derive its matched transpose/support and
metric without choosing a broad solver or changing this operator. Moving
geometry, pressure coupling, advancing liquid/solid dynamics, adhesion,
variable materials and arbitrary meshes remain outside this feature.
