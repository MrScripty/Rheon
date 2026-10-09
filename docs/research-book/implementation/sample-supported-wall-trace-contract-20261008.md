# Sample-supported wall trace: reconstruction decision and conditional accuracy contract

Research decision, 2026-10-08. Base `02142c537363659ac48ac01193084360f5260664` remains unchanged on its original branch. This branch adds diagnostic derivations, a bounded Python laboratory and separate conditional Lean statements. It changes no Rust, pressure, mass, stored strain row, boundary lift, expected physical load, tolerance, timestep or coupling. All generated evidence is external. The held PR25 and geometry-band campaigns remain outside scope.

## Decision and physical target

Use the **first outward tangential sample and the prescribed wall trace** for the minimum supported wall-gradient reconstruction. Integrate force, torque and rigid virtual work from one positive surface basis using its exact mass and first moments. This resolves the reconstruction-class choice with quantities already represented. No independent wall-jet input, extra active layers, polynomial fitted to the manufactured fixture, or owner-selected tolerance is required.

The target is the viscous load of a fully wet, constant-viscosity, continuously incompressible Newtonian liquid on a stationary aligned rectangular solid. Write `sigma_v=mu(grad u+grad u^T)` and let `n_s` point from solid into fluid. Solid traction is `t=sigma_v n_s`. Pressure is excluded. For a C1 field up to a stationary planar no-slip face, tangential derivatives of the zero trace vanish. If `div u=0` continuously there, the normal derivative of the normal velocity vanishes too. Consequently `t_i=mu partial_r u_i` for each tangential component and `t_n=0`, where `r` increases into fluid along `n_s`.

**Those are continuum premises.** Neither point samples, the current finite strain operator nor a zero MAC incidence divergence prove them. The laboratory separately reports the general *kinematic symmetric-gradient normal-secant comparator* `2 mu U_n/delta_n`; this is not a full compressible constitutive law, which would require a bulk-viscosity model. The comparator exposes the discarded normal secant rather than silently calling it physical zero. The continuously solenoidal manufactured control can have nonzero point-sampled MAC divergence; that observation does not change its analytic continuum premise.

At meeting no-slip faces, C1 continuation with both traces implies zero limiting wall gradient along their shared edge. This does not justify overwriting endpoint data with zero. All represented edge/end samples remain in the surface basis. Generic sharp-box Stokes solutions can lack this regularity; the contract below applies only when its stated bounds exist. It is not an unconditional convergence theorem for sharp obstacles.

## Smallest alternatives and rejection reasons

| Choice | Required present support | Exactness and error | Decision |
|---|---|---|---|
| P1 wall-plus-first-center ray | Positive represented wall distance; one fluid-cell padding suffices for tangential centers | Affine normal profiles; first-order derivative error | Minimum supported diagnostic |
| P2 wall-plus-two-center ray | Two distinct outward active tangential centers | Quadratic normal profiles; second-order derivative error under C3 and sample-error bounds | Optional sensitivity diagnostic only; reject when second center absent |
| Multidimensional/divergence-constrained polynomial fit | Additional independent moments/samples, rank and compatibility conditions | No unique exact boundary jet follows from minimum samples | Not the minimum admissible feature |
| Independent supplied wall derivatives | Data not represented in the present discretization | Could bypass sampling ambiguity if independently certified | Not required or invented for this decision |

For actual distances `0<a<b`, the two ray rules are

```
D1 = (U1-B)/a
D2 = -(a+b)/(ab) B + b/[a(b-a)] U1 - a/[b(b-a)] U2.
```

These are derived interpolation derivatives. [Fornberg's original arbitrary-node differentiation paper](https://people.sc.fsu.edu/~jburkardt/m_src/differ_arbitrary/fornberg.pdf) provides the general interpolation framework. The formulas and Rheon surface basis here are newly derived, not rules claimed verbatim from Batty. At `a=h/2,b=3h/2`, D1 sample-error amplification is `2/h`; D2's two-sample amplification is `10/(3h)` with exact B. P2 thus needs more support and amplifies sample error more, even when its truncation order is higher. Minimum padding has no second center in the coarse fixture; the laboratory refuses P2 there. It never uses a known outer trace as a fictitious second active center.

The preserved sampling counterexample rules out exact recovery for arbitrary admitted smooth fields even by a nonlinear reconstruction: all 48 coarse samples and wall traces can equal rest while the true wall load differs. Selecting a consistent approximation class does not undo this information limit. Recovering this one control by recognizing its polynomial would be fixture fitting.

## Surface integration and torque

For wall-normal direction n and tangential velocity component i, present MAC points lie at grid planes along i and cell centers along the other tangent j. On each wall use continuous piecewise-linear nodal hats along i, and piecewise-constant midpoint slabs along j. Retain endpoints. Distances, widths and wall locations must be represented physical quantities; no snapping or substitution by nominal spacing is allowed in a general implementation.

Let a node have left/right hat widths `l,r>=0`, not both zero. Direct integration gives

```
hat mass m = (l+r)/2
first moment about node = (r^2-l^2)/6
hat centroid offset = (r-l)/3.
```

With slab width `h_j`, positive area weight is `a_q=m h_j`. Its surface centroid has wall-normal coordinate equal to the actual wall, hat coordinate equal to node plus `(r-l)/3`, and slab coordinate at the midpoint. Endpoint centroids shift by one-third of the adjacent width. These are integrated basis moments, not relocation of an old boundary trace or of a purported physical point force.

For each tangential-component basis coefficient `T_q=mu D1_q`:

```
F = sum_q a_q T_q e_i
Tau_c = sum_q a_q (s_q-c) cross (T_q e_i)
(V,omega) dot (F,Tau_c)
    = sum_q a_q T_q [V+omega cross (s_q-c)]_i.
```

These identities are exact for the reconstructed surface field. Reference change obeys `Tau_(c+d)=Tau_c-d cross F`. The laboratory assembles both sides separately in rational arithmetic. The Lean transpose-work statement abstracts the same matched matrix pairing.

A trapezoid rule can integrate `t(x)=x` on `[0,1]` exactly for force (`1/2`) while its lumped moment is `1/2` instead of `1/3`. Consistent hat first moments give `1/3`. The remaining slab midpoint direction has its own limitation: for `t(y)=y`, force is exact but its moment is `1/4` instead of `1/3`. Therefore affine force exactness never implies affine torque exactness. Both limitations are explicit diagnostic controls.

Geometric surface area equals the sum of these basis areas on the represented surface when endpoints/slabs exactly tile it. That is separate from stored sector volumes in the unchanged viscous energy. Replacing `sum(v_sector)/delta^2` with geometric `A/delta` still needs the extra exact-product-volume hypothesis; this research supplies no such hypothesis for general f64 geometry.

## Conditional accuracy, with units and separate force/torque budgets

For an exact wall trace B and a twice differentiable normal profile with `|partial_r^2 u_i|<=M2` on `[0,a]`, Taylor's theorem gives `|D1-partial_r u_i(0)|<=a M2/2`. With sample and trace errors bounded by `eta1,etaB`, add `(eta1+etaB)/a`. For P2, with `|partial_r^3 u_i|<=M3` on `[0,b]`, the truncation bound is `ab M3/6`, plus the absolute interpolation coefficients times the corresponding data-error bounds. A proof of the derivative remainder follows by subtracting the quadratic interpolant and the cubic polynomial matching its derivative discrepancy at zero; Rolle's theorem applied with a double zero at zero and zeros at a,b yields the factor `ab/6`. The laboratory checks its sharp cubic control.

Multiplication by mu converts the derivative error into traction error `e_q` in force per area. Do not infer M2/M3 or eta from finite samples. P1/P2 disagreement is a sensitivity indicator, not an enclosure. The symbolic manufactured laboratory bounds derivatives by the sum of absolute centered polynomial coefficients times interval powers, so its particular bounds are independently checkable upper bounds. They do not certify arbitrary computed velocities.

On one face/component patch, let A be surface area, `h_i,h_j` the maximum element widths, and `T` the exact smooth wall traction component. Define uniform face derivative bounds `M_ii>=|partial_i^2 T|`, `M_jj>=|partial_j^2 T|`, `M_j>=|partial_j T|`. Piecewise-linear interpolation and composite midpoint integration give a force cubature bound

```
Q_F <= A [h_i^2 M_ii/12 + h_j^2 M_jj/24].
```

For torque component k with i!=k, let a be the other axis and `R_a>=|s_a-c_a|` over the face. Since the lever is linear, its consistent-hat moment error obeys

```
Q_Tau_k <= A [R_a h_i^2 M_ii/12
             + h_j^2 (R_a M_jj + 2 M_j 1_(a=j))/24].
```

Torque component i from this traction component is zero. The mixed piecewise-linear/midpoint construction makes the second term essential. For nonuniform partitions these bounds use maximum actual widths; regularity bounds must cover the full patches. Add ray/data errors using the same positive weights:

```
|Fhat_i-F_i| <= sum_(q of i) a_q e_q + sum Q_F
|Tauhat_k-Tau_k| <= sum_(q of i!=k) a_q |(s_q-c)_a| e_q + sum Q_Tau_k.
```

The latter uses the exact basis centroid; the error coefficient is constant on that basis. These are componentwise absolute, dimensional budgets in force and force-times-length units. Near cancellation, relative error with an arbitrary unit floor has no physical meaning. A future qualification must compare both actual errors against an application-specified absolute budget, separately from arithmetic roundoff; this task introduces no new tolerance or pass gate.

For a hypothetical fixed physical domain/box with shape-regular aligned refinement, first distances comparable to h, uniformly bounded normal derivatives and C2 wall traction, exact samples give P1 force and torque error O(h), and supported P2 gives O(h^2). This is an analytic implication, **not a new refinement campaign**. With maximum sample/trace error eta(h), errors instead include O(eta/h). Eta=o(h) is necessary for this bound to imply convergence; eta=O(h^2) preserves the P1 rate and eta=O(h^3) the P2 rate. No assertion is made that the current velocity method satisfies those bounds. Fixed IEEE error does not vanish as h goes to zero. Lean proves a conditional real secant bound and finite positive-weight propagation; it does not prove Taylor's theorem, this surface cubature theorem, solver convergence or IEEE refinement.

## Existing bounded controls remain physical failures

The unchanged physical reference is `F=(0,-1/2,0)`, `Tau_center=(0,0,-1)`, for the existing tilted streamfunction control, mu=1, box `[1,2]^3`. The smooth cutoff discussed in the preserved investigation can keep the used sample collars and wall derivatives while satisfying sealed exterior conditions. This is not an unforced steady Stokes solution.

| Existing fixture / candidate | Fy | Tau_z | Physical qualification |
|---|---:|---:|---|
| Coarse native / P1 | 0 | 0 | Fails: all samples alias rest |
| Coarse P2 | unsupported | unsupported | Refused: missing second center |
| Bounded preserved native | -72225/65536 | +8775/8192 | Fails, including torque sign |
| Bounded P1 surface observable | -199125/131072 | -43875/32768 | Fails unchanged physical target |
| Bounded P2 surface observable | +26325/32768 | -22275/16384 | Rejected, including force sign |

The endpoint data happen to vanish on this analytic control, so consistent hat moments do not change its previous flat-wall moment result. Their necessity is established by the separate affine-traction counterexample, not hidden by the fixture. Coarse-to-bounded P1 force error worsens while torque error improves. Two fixed samples do not demonstrate an empirical rate. The manufactured derivative bounds contain these errors; a loose conditional bound is not evidence of useful physical accuracy. The lab records old native loads, new observables, true loads and their differences without redefining expected values.

## Energy, rigid work, and the residual obligation before any coupling

[Batty and Bridson (2008)](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf) derive viscosity from symmetric strain and dissipation, retaining cross-component terms and rigid-motion behavior. This supports retaining the current positive finite energy rather than repairing a load by an unrelated torque redistribution. Their discretization does not establish the new wall-load rule above.

[Batty and Bridson's variational Stokes research, section 6.2](https://arxiv.org/pdf/1010.2832), expresses prescribed-boundary work through stress/pressure and a volume residual. Its reported velocity/stress experiments do not provide a theorem for this reconstructed integrated wall load. In the present viscous-only continuum diagnostic, if a virtual extension b equals the solid test on its boundary and zero on all other boundaries, integration by parts with the stated normals gives

```
- integral_fluid sigma_v : grad b
    = integral_solidwall (sigma_v n_s) dot b
      + integral_fluid (div sigma_v) dot b.
```

The volume term generally does not vanish for the manufactured field. A weak boundary energy gradient under a particular extension therefore need not equal the physical wall traction. The current finite operator still has `Phi=mu/2 (Eu)^T W(Eu)>=0` and `u^T f=-2Phi` for stationary traces. Its finite boundary transpose diagnostics are qualified within that model. This new P1 observable is not the derivative of that same energy.

Required conditions for a future common operator are: one declared gradient/reconstruction G including all cross components and represented boundary traces; nonnegative quadrature W; matched fluid and boundary transposes; zero symmetric strain for every supported rigid virtual lift; exact reference covariance and translation/rotation work closure across fluid, solid and outer boundary; and a discrete Green identity with an **independently assembled** volume divergence/residual term. The identity must account for the chosen extension, actual surface moments, and both boundaries. Only a justified equilibrium/residual bound can remove that volume term or bound its effect. [Del Rey Fernández and Zingg's original generalized SBP analysis](https://arxiv.org/html/1410.5029v1) motivates such matched integration-by-parts/energy requirements, particularly compatibility for mixed derivatives; it does not supply a Rheon MAC stencil.

For any rigid test, the observable work gap is exactly the test paired with `(g_P1-g_old)`. This is a diagnostic algebraic identity, not an evaluation of the volume residual. Keeping old fluid action and old outer wrench fixes the solid wrench under exact closure, as already proved at the preserved base. Consequently replacing only that wrench cannot satisfy closure. The lab exposes this gap; it does not hide it in outer-wall torque or rename it residual work.

## Deliverables, status, and next research boundary

The recommended accuracy contract is **conditional consistency with explicit ray, data and cubature bounds**, together with exact surface moment/work identities. Present-data runtime loads remain **uncertified diagnostics**. This is a research-model decision, not a repaired physical-load candidate. No owner preference is needed to choose the minimum supported reconstruction.

`tools/research_sample_supported_wall_trace.py` captures only the two already admitted fixtures from source-identified frozen debug/release Rust binaries, validates them with the unchanged arithmetic checker, verifies analytic sample correspondence, and computes P1/P2 loads, bounds, normal comparators, MAC divergence and work/reference controls exactly. `research/sample-supported-wall-trace/ConditionalTraceContract.lean` proves conditional secant error, affine reproduction, positive-weight error propagation, matched transpose work and the observable work gap. Its separate checker validates all pinned dependencies and audits logical declarations, with real negative probes rejecting a custom axiom and sorry. The production proof inventory is untouched.

Independent review must challenge the recommendation and its premises before production work. A successful research review does not lift physical-load, pressure, coupling, stepping or publication gates. The remaining work is mathematical and evidentiary: establish actual velocity-error/regularity bounds and useful physical accuracy, then derive a common energy/Green operator before coupling. Existing data cannot certify those facts or eliminate the preserved coarse alias. No held campaign, additional resource allocation or external input is requested by this decision.
