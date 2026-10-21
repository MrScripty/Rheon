# 36 Reference FEM algorithms and verification program

This chapter assembles the preceding derivations into implementable solver algorithms. The result is an **end-to-end core specification in pseudocode** for a fixed-topology hyperelastic solid and a separately selected mixed Newtonian fluid, with bounded nonlinear solution, reactions, diagnostics and failure. Contact/remeshing algorithms remain extensions from [Chapter35](35-contact-coupling-and-ale.md), not hidden capabilities of the core. No program was executed to produce the hand examples, no new Lean theorem is supplied, and no Rheon general FEM implementation is asserted. The [research status report](../lagrangian/research-status-report.md) identifies incomplete compressible-fluid/ALE/coupling coverage and remaining numerical selections; this chapter does not close those gaps.

## Model contract and data ownership

Choose one branch before constructing state. The solid uses 3D affine P1 tetrahedra, positive constant element reference density, the exact Chapter31 logarithmic neo-Hookean law, consistent mass and backward Euler. A separately labelled explicit option uses positive P1 lumped mass and the central-difference algorithm. The fluid uses continuous P2 motion/velocity, P1 pressure, consistent mass, constant reference density and viscosity, and the Chapter34 endpoint-volume residual. Its pressure space must be qualified for the actual mesh/boundary family. Neither branch includes fracture, plasticity, heat, shocks, capillarity or contact by default.

The baseline supports body acceleration independent of trial geometry and dead reference tractions, with fixed linear essential/periodic constraints. Geometry-dependent loads require their derivatives; current follower pressure can use the cofactor formulas already given. Use a domain and boundary classification consistent with the selected physics. For the fluid, a free boundary is the natural zero relative-traction boundary; an isolated fluid has no arbitrary pressure gauge. Closed prescribed boundaries require compatibility and appropriate gauge handling.

| Owned data | Contents | Lifetime and invalidation |
| --- | --- | --- |
| Reference mesh | X, tetrahedra, oriented boundary faces, material IDs, physical units | Fixed until explicitly conservative remesh |
| FE spaces | Local-to-global maps, basis/gradient tables, pressure map if needed, constraint embedding | Fixed topology/order/constraints |
| Material | Solid moduli and specified energy, or fluid density/viscosity | Immutable during one attempted step |
| Quadrature | Rule identity, weights, reference gradients and measures | Fixed for residual/tangent of the attempt |
| Accepted state | x or d, v, clock, history, accepted pressure where applicable | Changes only at successful publication |
| Candidate state | Newton iterates, trial geometry, constitutive scratch, residuals, diagnostics | Disposable on rejection/cancellation |
| Linear solve | CSR graph or operator action, preconditioner, bounded Krylov/direct scratch | Rebuilt when required by actual operator |
| Output | Selected accepted state and its model/source/input identity | Derived after acceptance; never a competing state |

CSR preallocation follows element support and constraints. A dense pivoted direct solve is a useful bounded small-system reference, not a scalable default. Preserve pressure nullspaces explicitly; a zero pivot may indicate a missing boundary condition or spurious pressure mode and must not be repaired by inventing diagonal stiffness. Allocate all required arrays within stated limits before stepping. If limits cannot support the requested mesh/order/iteration space, fail construction.

## A conservative geometry guard that is implementable

General large-deformation geometry certification is a substantial subsystem. The following sufficient guard provides a concrete, deliberately restrictive baseline without claiming that sampling proves positivity. Assume the reference body is convex and initially unstrained, with affine reference tetrahedra; for fluid P2 reference edge nodes are exact midpoints. Fix a dimensionless admissibility parameter \(0<\theta<1\). On every tetrahedron bound \(\|\nabla_Xd\|_F<\theta\) everywhere, where \(d=x-X\).

For P1 the gradient is constant. For P2 on an affine reference tetrahedron it is affine, so its value is a convex combination of its values at the four reference vertices. Convexity of the Frobenius norm bounds it by the largest vertex norm. This is a bound over the whole element, unlike determinant sampling at arbitrary quadrature points. Compute rigorous outward bounds if this is ever used as a software certificate; ordinary rounded comparisons alone are numerical diagnostics.

**Conditional proof.** Continuity and the piecewise gradient bound on a convex body give \(|d(X)-d(Y)|\le\theta|X-Y|\) by integrating along the straight segment. Consequently \(|\chi(X)-\chi(Y)|\ge(1-\theta)|X-Y|\), proving injectivity. Singular values of \(I+\nabla d\) are at least \(1-\theta>0\); continuity from the identity proves positive determinant. The same bound holds along any convex interpolation between two admissible displacement fields. It therefore covers Newton line segments and the baseline linear-in-time accepted mesh path. It also rejects many valid large rotations/shears and nonconvex bodies. Replacing it with interval/Bernstein determinant certificates and continuous global intersection checks is a separately qualified extension, not a reason to weaken it silently.

This guard is a pedagogical choice, not a Rheon threshold, and is never written into a production manifest. Its exact-real proof does not certify floating-point implementation. It allows a concrete reference solver while retaining a clear failure result for unsupported geometry.

## Construction and initialization

~~~text
construct(model, reference_mesh, boundary_data, budgets):
    verify units, finite parameters, density>0, material-domain assumptions
    verify affine reference tetrahedra have positive orientation
    verify matching shared faces, outward boundary faces, convex reference body
    choose solid P1 or fluid P2/P1 spaces; build global DOF maps
    build fixed essential/periodic embedding R and prescribed data map
    choose quadrature rules; cache reference gradients and weights
    assemble consistent M0 with adequate mass quadrature
    retain solid lumped mass only for explicitly selected explicit mode
    analyze actual rigid/pressure nullspaces and boundary compatibility
    allocate candidate, sparse graph, solver vectors and output within budgets
    initialize x0=X, compatible v0 and clock; fluid volume moments start at zero
    initialize solid acceleration from constrained force balance if needed
    require admissible initial geometry and state
    return reference data + one accepted state + separate workspace
~~~

Boundary geometry/topology validation and parameter validation precede allocation where possible. Mesh orientation repair, if offered, is a separately recorded import operation; do not reinterpret negative deformation determinants as harmless node ordering. For fluid initial velocity, either supply a compatible field or explicitly perform a qualified constrained initialization and record the changed field. Initial pressure is a constraint multiplier obtained from the corresponding acceleration/boundary problem when needed, not a material energy variable to advect as if it were density.

## Solid implicit step

~~~text
attempt_solid_step(accepted, h, bounds):
    require finite h>0 and advancing representable clock
    d_pred = d_n + h*v_n
    z = bounded initial free-coordinate guess satisfying essential data
    repeat at most max_newton:
        d = d_D(t_n+h) + R*z
        require whole-mesh geometry_guard(d)
        assemble Psi, fint, Kint by chapter32 and material A of chapter31
        assemble dead/reference loads at t_n+h
        r_full = M0*(d-d_pred)/h^2 + fint - fext
        r = transpose(R)*r_full
        K = transpose(R)*(M0/h^2 + Kint - Kext)*R
        if scaled residual and required increment criteria pass: break converged
        solve K*s = -r within linear iteration/memory/time limits
        recompute true linear residual; reject failed solve
        compute derivative of fixed scaled residual merit; require descent
        bounded_backtracking with geometry guard and Armijo decrease
        reject if no feasible acceptable trial or cancellation requested
    reject if not converged within cap
    v = (d-d_n)/h; a = (v-v_n)/h
    independently recompute final full residual, energy, reactions and geometry
    require finite state and declared algebraic/diagnostic gates
    cancellation barrier; publish x=X+d, v, time and diagnostics atomically
~~~

Do not accept solely because the loop exhausted its count. Re-evaluate residual at the actual final trial; a line-search residual can be stale after a state update. For prescribed components, \(r_{\mathrm{full}}\) is the support load required by balance under the internal-positive convention; the applied support reaction is that value, with its opposite acting on the support. Body momentum accounting includes these forces. Derive velocity boundary values from the same discrete displacement update, or supply displacement data constructed to match desired integrator-compatible velocity.

For the explicit branch, use the Chapter33 half-step update, evaluate forces at the specified accepted stage and check geometry/CFL/contact conditions before publication. A failed candidate leaves both accepted displacement and half-step velocity unchanged. Variable-step central difference needs its own derivation; simply substituting a new h into a stored fixed-step half velocity is not the stated scheme.

## Fluid implicit step and element action

~~~text
attempt_fluid_step(accepted, h, bounds):
    choose compatible constrained velocity guess and pressure coordinates
    converged = false
    for at most max_newton:
        v = v_D + Rv*z_v; p = pressure_embedding(z_p)
        x = x_n + h*v; d = x-X
        require geometry_guard(d)
        rv = M0*(v-v_n)/h; g = zero_pressure_vector
        for each tetrahedron and quadrature point:
            compute N_P2, gradX_N_P2, Q_P1, reference weight
            F = sum_a outer(x[a], gradX_N[a]); J=det(F)
            L = (sum_a outer(v[a], gradX_N[a]))*inverse(F)
            D = sym(L); p_q=sum_alpha Q[alpha]*p[alpha]
            Pf = J*(-p_q*I + 2*eta*D)*inverse_transpose(F)
            rv[a] += weight * Pf * gradX_N[a]
            g[alpha] -= weight * Q[alpha]*(J-1)/h
        subtract body/reference-traction loads from rv
        apply velocity constraints and actual pressure nullspace treatment
        form tangent action using every delta formula in chapter34
        check BOTH separately scaled momentum and volume residuals
        if both residual gates and required increment criteria pass:
            converged = true; break
        bounded block/GMRES solve, true residual check, feasible merit line search
        reject failed linear/nonlinear solve, invalid arithmetic or cancellation
    reject if not converged within cap
    recompute final geometry, momentum, volume, density and work diagnostics
    require all declared gates; keep weak/local volume errors visible
    cancellation barrier; publish x, v, p, clock and diagnostics atomically
~~~

The element loop uses the **same candidate geometry** for stress, determinant, gradient, volume constraint, boundary load and tangent. Fluid reference mass remains M0. Quadrature identities in Chapter34 are statements for exact integration; actual quadrature defines the implemented residual and needs separate consistency checks. The constraint scaling \(-1/h\) is included in its derivative and its residual units. If rank/stability or geometry conditions are unavailable, the outcome is unsupported/unqualified, not an automatically stabilized answer.

A useful fixed-geometry block hand example separates algebra from full motion: with dimensionless scaled unknowns take \(A=\mathrm{diag}(2,2)\), \(B=(1,-1)\), \(f=(2,0)^T\). Solving \(Av-B^Tp=f\), \(-Bv=0\) gives \(v=(1/2,1/2)^T\), \(p=-1\). The Schur complement is \(-BA^{-1}B^T=-1\). Changing both off-diagonal signs changes the multiplier convention, not the constrained velocity. This two-velocity toy is not a stable FEM pair, a physical pressure calibration or execution of the nonlinear moving-fluid algorithm.

A physical fluid hand check uses a vertical container with bottom and side no-slip walls and a horizontal top at height \(H_0\), natural zero relative traction. With \(x_n=X\), \(v_n=0\), \(b=-g_0e_3\), \(g_0>0\) in m/s\(^2\), set \(v=0\), \(J=1\), \(p=\rho_0g_0(H_0-X_3)\). Pressure is linear and exactly represented by P1; \(D=0\). Its gradient is \(-\rho_0g_0e_3\), so \(-\nabla p+\rho_0b=0\). For tests zero on the walls, integration by parts gives

\[
 -\int_{\Omega_0}p\nabla_XN_a\,dV_0
 =\int_{\Omega_0}N_a\nabla_Xp\,dV_0
 =-\int_{\Omega_0}\rho_0g_0N_ae_3\,dV_0.
\]

The top boundary term vanishes since p=0; wall tests vanish. Thus the fluid momentum residual and every volume moment are zero in exact arithmetic with adequate quadrature. At \(\rho_0=1000\) kg/m\(^3\), \(g_0=9.81\) m/s\(^2\), \(H_0=1\) m, relative bottom pressure is 9810 Pa. The free traction fixes the pressure level; subtracting its mean breaks this equilibrium. This is a manufactured exact static state for the stated discrete spaces/geometry, not an executed hydrostatic measurement or a moving free-surface convergence result.

## Open-source architectures worth studying

The [source map](../lagrangian/source-map.md) pins versions, source paths and license evidence. The comparisons below are source-guided architectural observations, not comparative benchmarks or adoption decisions.

| Project and inspected version | Common architecture and lesson | Tradeoff/domain boundary |
| --- | --- | --- |
| [MFEM v4.7 ex10](https://github.com/mfem/mfem/blob/v4.7/examples/ex10.cpp) | FE forms, mass and hyperelastic operators feed a separate ODE solver; reduced implicit nonlinear operator | Compact mechanics example; its reference diffusion damping is not a full moving-fluid viscous stress |
| [DOLFINx v0.9.0 hyperelasticity](https://github.com/FEniCS/dolfinx/blob/v0.9.0/cpp/demo/hyperelasticity/hyperelasticity.py) | Symbolic energy/form differentiation generates residual and Jacobian; assembly, boundary constraints and PETSc own numerical execution | Reduces hand-derivative maintenance; generated derivatives still depend on correct form, domains and quadrature |
| [deal.II v9.6.2 step44](https://github.com/dealii/dealii/blob/v9.6.2/examples/step-44/step-44.cc) | Explicit finite-strain material/quad-point state, mixed blocks, optional static condensation | Makes material and algebra inspectable; three-field conventions/spaces must remain consistent |
| [Laghos v3.1](https://github.com/CEED/Laghos/blob/v3.1/laghos_solver.hpp) | Continuous kinematic and discontinuous thermodynamic spaces, shared force operator, full/partial assembly | Compressible Euler miniapp; high-order tensor-product action can reduce global matrix traffic, with different preconditioning/work costs |
| [Kratos PFEM pinned 9452ba2](https://github.com/KratosMultiphysics/Kratos/blob/9452ba205627aa4a182368b2fbcbc41bb68b62ab/applications/PfemFluidDynamicsApplication/python_scripts/pfem_fluid_solver.py) | Python orchestration, ModelParts and separate pressure/velocity strategies with moving-mesh settings | Production application infrastructure is larger than an element loop; remeshing/strategy/stabilization details are formulation-specific |
| [FEBio v4.7 solid domain](https://github.com/febiosoftware/FEBio/blob/v4.7/FEBioMech/FEElasticSolidDomain.cpp) | Elements/material points and domain-level internal-force/stiffness assembly separated from analysis/solver state | Constitutive history and mechanics domain are first-class; broader mechanics coverage is not proof of this reference fluid branch |

Full assembly stores global sparse operators and enables established factorizations/AMG. Partial assembly stores quadrature/operator data and applies local basis maps; high-order tensor products can reduce work/storage. Neither label alone determines performance. MPI ghost accumulation must gather trial values before element evaluation and sum shared residual contributions afterwards; matrix-free and assembled paths must implement the same constrained action. Quadrature histories need accepted/candidate semantics even when a framework offers convenient mutable material objects.

A future Rheon implementation could adopt an existing library or implement a narrow operator set. That decision needs dependency/license/build/resource review and actual equation fit. The book imports no third-party code and does not prescribe a Rust wrapper or framework registry.

## Verification ladder and meaningful failures

A complete exposition is not completed scientific qualification. Establish evidence in increasing dependency order, with independent oracles and explicit bounds:

1. Check basis partition/linear reproduction, affine maps, mass entries, traction signs and objective rigid motions against Chapter32's exact hand calculations. Test deliberately reversed orientation and zero-volume input.
2. Check material stress/energy/tangent directional consistency and assembled residual derivative using independent symbolic or finite-difference references over admitted positive-J states. A finite-difference comparison needs a scale sweep to expose truncation/cancellation; it is numerical evidence, not proof.
3. Check constrained free/support force balance, internal resultant/torque and fixed-topology mass. Reject perturbed load area, omitted tangent block, pressure nullspace and swapped coordinate gradients at the intended boundary.
4. Check linear axial/elastic waves and manufactured smooth solutions with independently derived source/boundary data. Separate spatial refinement, timestep refinement and quadrature refinement so one error does not mask another. Report L2/H1 displacement or velocity, pressure modulo only a legitimate gauge, stress/traction, volume and work errors.
5. For fluid, include fixed-domain Stokes, rigid translation/rotation, hydrostatics under correctly signed boundary traction, viscous decay, free-surface motion and the endpoint-divergence counterexample. Measure total and local J/density errors, actual mixed conditioning, pressure/traction convergence and artificial work.
6. Qualify contact, remap and FSI only after their own equations are implemented: gap/multiplier complementarity, frictional loss, continuous path feasibility, overlap balances, rigid-motion transfer, interface work, added-mass sensitivity and rollback. Breakup/merging need new domain-volume/transfer evidence.
7. Test rejection/cancellation at nonzero accepted state, iteration/memory limits, restart identity if supported and stale-output protection. Native/cloud/GPU timings or accuracy require runs in each claimed environment with exact source/input and resource bounds.

For a smooth stable conforming elliptic problem with coercivity constant \(\alpha\), continuity constant \(C\), exact integration and Galerkin orthogonality, Céa's argument yields \(\|u-u_h\|_V\le(C/\alpha)\inf_{w_h}\|u-w_h\|_V\). Proof: test the error relation against \(u_h-w_h\), use coercivity and continuity, then divide by the error norm (zero error is immediate). Interpolation estimates on shape-regular meshes can give order-k energy errors and, with extra regularity/duality assumptions, higher L2 orders. This conditional **linear** result is not a convergence proof for finite-strain contact, moving mixed free surfaces, mesh degeneration or the proposed nonlinear timestep.

A future bounded qualification should start with one owned small mesh, one exact case, explicit maximum DOFs/steps/iterations, wall/CPU/memory/output limits and an independently reviewed oracle. Broader parameter sweeps, adversarial corrupted outputs and native performance follow only through a separate admission. The [review and qualification guide](../lagrangian/review-guide.md) names those pending needs. No existing numerical threshold or proof receipt is broadened by these chapters.
