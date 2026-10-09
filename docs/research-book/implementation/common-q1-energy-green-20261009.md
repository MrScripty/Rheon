# A common Q1 energy/Green operator, and what it cannot repair

This is a new bounded mathematical candidate from preserved `7fd70976e88d244c6e07a167b5f9d39f06cad425`. Its active unknowns, sample positions, stored masses and two native input fixtures are unchanged. The candidate replaces the strain reconstruction and its action **in the research laboratory only**. Production Rust, pressure, stepping and coupling are untouched.

The result is constructive: the existing samples can determine a continuous piecewise-trilinear velocity, a positive symmetric-strain energy, its matched fluid/boundary transposes, and an exact broken-cell Green identity. Thus there is no general same-samples obstruction to a common energy formulation. There is an obstruction to identifying its weak boundary reaction with surface traction without residual work, and the fixed coarse sampling ambiguity still prevents physical acceptance. This candidate is rejected as an accuracy repair on both unchanged fixtures.

## Why the isolated P1 wall-ray energy is insufficient

Let a flat wall point be p, its outward fluid sample q=p+delta*n, tangent e_i, positive area a, and stationary trace B_i. The scalar energy `Phi=mu*a*delta/2*((U_i-B_i)/delta)^2` yields the desired P1 surface force `a*mu*(U_i-B_i)/delta`. Under a common rigid rotation, however, `U_i-B_i=delta*(omega cross n)_i`. The energy is positive for a nonzero supported rotational component. Fluid force at q and solid force at p leave a torque defect `-delta*n cross t`. This energy cannot supply the required common-rigid kernel.

Restore symmetric shear `S=(U_i-B_i)/delta + partial_i B_n`. For rigid B, `partial_i B_n=-(omega cross n)_i`, so S annihilates common rotation. Differentiating the restored energy against a solid twist yields

```
a*mu*S * [R_i(p) + delta*(omega cross n)_i] = a*mu*S*R_i(q).
```

The boundary reaction therefore has the sample lever q, not the wall lever p: its extra moment is `delta*n cross t`. Replacing that reaction with wall-only P1 work deletes a required derivative. This is a precise isolated-row incompatibility; it is not a proof against all richer reconstructions using the same samples.

A local shared-field example makes the missing term explicit. On `0<=x<=d`, `-h/2<=y<=h/2`, use actual tangential offsets U=u_y(d,0) and normal offset N=u_x(H,0), with wall twist `(Vx,Vy,omega)`:

```
u_x = Vx-omega*y + (x/H)^2*(N-Vx)
u_y = Vy + (x/d)*(U-Vy)
gamma = (U-Vy)/d-omega
Phi = mu*h*[4*d^3/(3*H^4)*(N-Vx)^2 + d/2*gamma^2].
```

It has an exact common-rigid kernel and wall traction `(0,mu*gamma)`. Its weak solid reaction is `(8*mu*h*d^3/(3*H^4)*(N-Vx), mu*h*gamma, mu*h*d*gamma)`; the wall moment about the patch center is zero. In the rotational test, bulk divergence work vanishes, but lateral patch flux supplies the extra moment. The independent local symbolic lab verifies the complete volume and patch-face identity. A local gap cannot all be called a bulk-volume residual: interfaces matter.

## Concrete global reconstruction using existing data

For velocity component i, start with its native MAC lattice: face planes along i, cell centers along the other axes. Base nodal values are the active samples; missing solid/solid-boundary component values are the rigid solid trace; sealed exterior normal values are the exterior rigid trace. Inside-solid values are only part of a declared interpolation extension, not added fluid data or unknowns. Add the retained box planes and exterior planes as component-grid knots.

At every augmented knot p on or inside the closed solid, set the component to `R_s(p)`. Else use base tensor interpolation with tangential coordinates clamped to their original center ranges, plus

```
R_o,i(p)-R_o,i(p_clamped).
```

This exterior correction preserves common affine rigid motion. With zero exterior virtual coefficients, tangential components are constant in their exterior half-cell collar, and the exterior normal trace is zero. Consequently exterior shear traction is zero. For a general exterior rigid virtual trace, the correction gives `partial_n u_t=-partial_t R_o,n`, again preserving zero shear traction. The solid trace is exactly stationary no-slip when its virtual coefficients vanish. The twelve solid/exterior twist coefficients are derivative probes; no moving-body state is introduced.

Take the common overlay of all component knot planes and interpolate the augmented fields tensor-linearly. Components are continuous on shared faces and trilinear within each fluid integration box. Retained solid planes are exact overlay faces, so omit precisely the solid integration boxes. No unexplained partial or missing box is accepted. Original active values are interpolated exactly. The two existing sampling grids remain unchanged; the overlay only partitions their reconstructed integrals. It has 208 and 1664 fluid integration boxes, respectively, with the same 48 and 504 active scalar unknowns.

If every active value and both virtual boundary traces come from one rigid field R, all augmented knots equal R(p). Tensor interpolation reproduces affine R, so the symmetric gradient is identically zero. The laboratory verifies every coefficient of all six rigid fields at every element vertex. It also checks continuity of each boundary-test extension across every integration interface, no-slip solid traces, free-slip exterior stress, and connectedness of the fluid integration graph.

## Energy and actions from that same field

Let L map active and virtual coefficients x to element vertex velocities. For reconstructed velocity v=Lx, constant mu>=0, define

```
S = grad(v)+grad(v)^T
sigma = mu*S
Phi(x) = mu/4 * sum_K integral_K S:S
       = mu/2 * sum_K integral_K [2*sum_i (partial_i v_i)^2
                                + sum_(i<j) (partial_j v_i+partial_i v_j)^2].
```

All derivative polynomials have degree at most one in each coordinate. Their products have degree at most two in each coordinate. Tensor Simpson nodes `(0,1/2,1)` and positive weights `(1/6,2/3,1/6)` integrate them exactly. There are six retained strain families at each of 27 quadrature points per element; no family or zero row is silently dropped. With positive represented widths, the normal-row weights are `2*mu*V*w_q`, and engineering-shear weights are `mu*V*w_q`.

Equivalently each element has a symmetric 24-by-24 Gram matrix K. The laboratory assembles K twice: once by exact polynomial moments and once by all 162 positive strain quadrature rows. For component-node pairs `(i,a),(k,b)` its independent integral formula is

```
K_(ia,kb) = mu * integral_K [1_(i=k)*sum_j partial_j N_a*partial_j N_b
                           + partial_k N_a*partial_i N_b].
```

The operator is fully specified by the prolongation and these local matrices:

```
H = sum_K L_K^T K_K L_K
(f,g_s,g_o) = -H x.
```

Stationary boundaries give `u dot f=-2*Phi`, `Phi>=0`, and exact common-rigid resultant closure `R_f^T f+g_s+g_o=0`. These are the candidate's own forces and reactions; they do not preserve the old action. Changing the reference c to c+d changes each torque by `-d cross F`, directly from the declared rigid basis `V+omega cross(p-c)`.

There is also a finite positivity argument beyond a generic semidefinite claim. A trilinear field with zero symmetric gradient in one box is affine rigid: matching its polynomial coefficients eliminates the mixed terms and leaves a skew linear part. Neighboring rigid fields equal on an open shared face must have identical rotation and translation. Connectedness therefore leaves only a global rigid kernel. With stationary no-slip solid trace that kernel vanishes; preservation of active point values makes the stationary-fluid Gram positive definite for mu>0. This is a finite reconstruction argument, not a claim about a full liquid PDE. Independent exact element matrices at the two integration lengths have rank 18, nullity six, and the six declared rigid vectors in their kernel.

A conservative exact rational fluid row bound is computed as

```
U_j = (1/m_j) sum_K,a |L_(a,j)| sum_b |K_(a,b)| sum_(k fluid) |L_(b,k)|.
```

Triangle inequality bounds the absolute fluid Gram row sum per stored mass by U_j. It includes mu. The observed maxima are `11231/288` and `137609/576`. These are research bounds for this candidate, not the old B, not IEEE enclosures or timestep authorization. No step is performed.

## Genuine discrete Green identity, including stress jumps

Let b_s be the reconstructed derivative with respect to a solid virtual twist. It equals that rigid test on the solid wall and zero on the exterior. On an internal face oriented from K_minus to K_plus, define `J=(sigma_minus-sigma_plus)*n_minus`. Because b_s is continuous, its face trace is shared. Element integration by parts gives

```
g_s = g_surface + r_bulk + r_jump
r_bulk = sum_K integral_K div(sigma_K) dot b_s
r_jump = -sum_internalfaces integral_face J dot b_s.
```

Solid surface traction is `sigma*n_s` with n_s pointing into fluid. Its force and torque are integrated exactly from the same reconstructed field. The exterior has the analogous identity: exterior tangential traction vanishes, so only its prescribed rigid normal trace enters boundary work. The laboratory derives bulk divergence by differentiating each polynomial, independently evaluates both limits of every internal stress flux, and integrates their products with the test. It never defines a residual as `g_s-g_surface`.

The equality of a weak reaction and wall traction requires the corresponding **combined** bulk and interface work to vanish, or be bounded by a justified physical residual budget. A pressure-free stationary sampled manufactured field does not establish that. A stress derived from a continuous Q1 velocity generally jumps across interfaces; treating only cell-interior divergence as the residual is incorrect.

The candidate field is not continuously divergence-free. All normal stress is retained; this surface operator is not the prior conditionally incompressible P1 tangential observable. Added tangential endpoint values also change the surface interpolation from the previous hat-times-midpoint rule. No continuum regularity, corner C1 property or velocity-error guarantee is inferred from point data.

## Unchanged acceptance evidence and quantified failure

The analytic target remains `Fy=-1/2`, `tau_z=-1` for the same stationary, solenoidal manufactured field and unit solid. No target, arithmetic budget or physical tolerance changes.

| Bounded candidate or diagnostic | Fy | tau_z | Error Fy | Error tau_z |
|---|---:|---:|---:|---:|
| Preserved native full strain | -72225/65536 | +8775/8192 | -39457/65536 | +16967/8192 |
| Scalar tangential P1 wall-ray / prior surface observable | -199125/131072 | -43875/32768 | -133589/131072 | -11107/32768 |
| Restored symmetric flat-row subset | -199125/131072 | -37125/16384 | -133589/131072 | -20741/16384 |
| Previously rejected P2 wall observable | +26325/32768 | -22275/16384 | +42709/32768 | -5891/16384 |
| Common Q1 reconstructed surface stress | -207675/32768 | -124425/32768 | -191291/32768 | -91657/32768 |
| Common Q1 weak energy reaction | -23528925/2097152 | -164025/16384 | -22480349/2097152 | -147641/16384 |

The Q1 surface normal part is `(Fy,tau_z)=(-512325/131072,-22275/16384)`; its tangential part is `(-318375/131072,-79875/32768)`. The target normal viscous traction is zero. Both parts therefore expose accuracy problems; zero normal stress is not silently imposed on this non-solenoidal reconstruction.

For the bounded Q1 candidate, the directly integrated solid residual components are

| Term | Force-y test | Rotation-z test |
|---|---:|---:|
| Cell-interior divergence work | +386775/32768 | +1221975/131072 |
| Signed internal stress-jump work | -34991325/2097152 | -2036475/131072 |
| Combined residual | -10237725/2097152 | -203625/32768 |

Adding the last row to the surface stress gives exactly the weak reaction. The bounded energy is `173503597288249580625/2199023255552`; its fluid work is exactly minus twice that. Large bulk/interface terms arise from reconstructing the actual sampled field, not from a steady unforced momentum solution.

In the coarse fixture all active samples are zero. All linear candidates above give zero energy and load, leaving errors `(Fy,tau_z)=(1/2,1)`. P2 remains unsupported there. The zero continuum field has the same sampled and stationary-trace inputs as this nonzero-load control. Any rest-consistent function of those inputs must fail one of them. Extra quadrature or a new transpose cannot recover absent information. This preserved information obstruction alone excludes a physically qualified candidate on both fixed acceptance fixtures under the present data; it does not exclude a consistent approximate operator with conditional refinement assumptions.

## What is established

`tools/research_common_green_q1.py` provides the concrete bounded candidate, actual frozen Rust inputs, and exact action/energy/Green diagnostics for only the two existing fixtures. Original research motivates symmetric strain and variational boundary work: [Batty and Bridson 2008](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf) and [their variational Stokes work, section 6.2](https://arxiv.org/pdf/1010.2832). This Q1 prolongation, quadrature and broken-interface construction are new derivations, not stencils attributed to those papers.

`research/common-green-q1/SharedEnergyGreen.lean` proves nonnegative weighted energy, gather/transpose work, conditional rigid-kernel work, the Green equality criterion, and the scalar-ray rotational obstruction. Geometry, interpolation, exact integration and element Green identities are supplied by the derivation and independent bounded labs; the Lean leaf does not prove them, physical correctness or IEEE behavior. Its source-bound audit uses pinned existing dependencies and actual custom-axiom/sorry rejection probes.

The useful result is a shared formulation with explicitly measurable residual work, plus a sharp reason that wall-only P1 reaction cannot be substituted into the original symmetric stencil. It remains rejected for production accuracy. A future physical proposal must improve or enrich the velocity/trace reconstruction and qualify its errors while preserving the common formulation; declaring the current failures passing or absorbing residuals into an outer wrench is not permitted.
