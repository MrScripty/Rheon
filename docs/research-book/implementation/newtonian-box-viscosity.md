# Constant-density Newtonian viscosity in the sealed box

This implementation companion adds a bounded Newtonian update to the existing transactional carrier/liquid facade. It supports a **fully filled sealed Cartesian box**, constant positive density rho, constant nonnegative dynamic viscosity mu, and stationary impermeable **free-slip** walls. Normal wall velocity is prescribed zero. Tangential viscous traction is zero; this is not a no-slip or adhesion model. Material units are rho in kg/m³, mu in Pa s = kg/(m s), and nu = mu/rho in m²/s.

`LiquidTransportSimulation::step_viscous(inputs, workspace, mu, cancel)` uses the same carrier, pressure workspace, phase owner, candidate fields and publication barrier as `step`. A separately capped `ViscosityWorkspace` owns three face-sized f64 vectors. Its payload is 8 times the sum of the three face counts. With nominal capacities, writing F for the sum of face counts and N for cells, the combined payload is 24F + 80N bytes in 23 arrays. The existing facade payload is unchanged; the step report exposes both capacities and their sum. Scratch holds the assembled stress action, then the checked stored-f32 candidate. The step allocates no arrays and retains no additional accepted velocity snapshot.

The facade rejects binary slabs, reconstructed columns, partial occupancy, phase sources, a mismatching workspace, and unequal represented/carrier densities. Viscous box-flux steps are not exposed. These admission conditions deliberately limit the material interpretation to the fully filled one-liquid model. Appearance tracer and prescribed body forces retain their existing meanings. Arbitrary fully filled flows can still fail the original pressure, raw fraction or conservation gates: rounded projection does not imply exact divergence, and no phase clamp has been added.

## Operator and boundary conditions

For symmetric strain D = (grad u + grad u transpose)/2, stress is tau = 2 mu D. At cell centers let e_a be the difference of the two a-oriented faces divided by h_a. At each interior ab edge let s_ab = delta_b u_a/h_b + delta_a u_b/h_a. The discrete dissipative power is

    P(u) = mu V [2 sum_cells,a e_a² + sum_interior_edges,ab s_ab²].

Normal strain rows have weight 2; engineering shear rows have weight 1. Each row scatters its transpose back to the **same** face degrees of freedom. In particular, a shear row couples both velocity components. Boundary shear rows are omitted to impose homogeneous tangential traction. Outer normal faces are fixed zero and are excluded from the unknowns. All interior face masses equal rho V. This is uniform-box quadrature, with no cut-cell or partially filled momentum mass.

Writing P = mu V u transpose L u gives L = E transpose W E/V in the dimensional quadrature convention, or simply E transpose diag(2,1) E when V is factored out. Here E includes inverse spacings. Thus L is symmetric positive semidefinite in exact arithmetic. The explicit viscosity-only stage is

    v = u - dt (mu/rho) L u.

This stage runs after advection and body forces and before pressure. It is neither an implicit viscosity solve nor a coupled Stokes solve. Its energy report covers the viscosity stage only, not total advection/forcing/projection work.

A conservative bound follows from (x+y)² <= 2(x²+y²): the strain quadratic form is bounded by twice the component gradient quadratic form. Each one-dimensional difference graph, including fixed normal endpoints and homogeneous tangential traction, has squared-difference bound 4/h_a². Hence lambda_max(L) <= 8 sum_a 1/h_a². The production gate

    r = nu dt sum_a 1/h_a² <= 1/4

ensures dt nu lambda_max <= 2 in exact arithmetic. The implementation rejects a larger requested stage step; it does not silently insert substeps or increase a solver tolerance. Existing carrier CFL selection still runs before the stage and can reduce dt independently.

With M = rho V I and delta = -dt nu L u, the exact viscosity identity is

    K(v)-K(u) = -dt P(u) + (1/2) delta transpose M delta.

The final stored-f32 velocity adds rounding epsilon. Its separately measured work is epsilon transpose M (v + epsilon/2). Compensated sums accumulate strain, kinetic energies, update energy and rounding work. The energy-identity and nonincrease gates retain their explicit 64 binary64-epsilon budget plus measured absolute rounding work. The 64-cell draft exposed a failure of ordinary diagnostic summation; compensated summation repaired that arithmetic without changing the gate. Nonzero subnormal f64 intermediates, multiplication/division underflow to zero, nonfinite values and nonzero subnormal stored-f32 candidates are rejected. This restrictive finite-scale policy is not a proof of IEEE accuracy.

## Transactions and checked behavior

The standalone workspace `update` checks shapes and every candidate before copying to its caller-provided output. Every callback occurrence can cancel, including the final acceptance callback. The facade prepares viscous carrier velocity, pressure and conservative phase before its existing final commit. Failure or cancellation preserves published velocity, pressure, tracer, fractions, times and both versions. Eight native contract tests exercise the hand-derived cross-component stencil, all three component pairs on an anisotropic 3D mode, spatial/time refinement, zero viscosity, unsupported domains and scale failures. The cancellation regression cancels **each occurrence** in the isolated stage and in the first coupled interval, then checks retry equality. A late phase-Courant rejection and an excessive-viscosity rejection preserve both accepted owners after nonzero history.

## Numerical evidence, distinct from physical validation

The isolated analytic mode is

    u_x = sin(pi x) cos(pi y),
    u_y = -cos(pi x) sin(pi y), u_z = 0,

on a 1 m by 1 m box, extruded over three Z cells. It is divergence free, has zero normal velocity and zero tangential traction at the walls, and decays as exp(-2 pi² nu t). Its discrete MAC eigenvalue is lambda_h = 8 sin²(pi/(2n)) n². Explicit Euler predicts amplitude (1 - nu dt lambda_h)^steps. The initial sampled mode is stored as f32, so small sampling and per-step rounding errors are measured separately.

At nu = 0.05 m²/s and T = 0.1 s, the spatial study uses stability policy r approximately 0.1, below the unchanged 1/4 gate. RMS velocity errors against continuum decay at n = 8,16,32,64 are approximately 2.247e-4, 4.720e-5, 1.129e-5, 2.371e-6. Time refinement at fixed n = 12 gives amplitude errors against exact **discrete** exponential decay of 1.367e-4, 6.826e-5, 3.419e-5 for 32,64,128 steps. These observations support second-order spatial and first-order temporal behavior for this smooth isolated mode; they do not establish general convergence of the complete liquid step.

The retained smaller-step study (`evidence/viscosity/refinement-limit`) is a completed native run that **fails the accuracy oracle**. At n = 64, 2053 f32 updates and r approximately 0.02, the maximum modal error is 1.644e-5 and maximum amplitude error is 1.997e-5, exceeding the unchanged 3e-6 modal oracle. Continuum RMS error is 1.382e-5, showing a rounding plateau. Smaller dt is not automatically more accurate in stored f32. This case is recorded separately from the 13 passing scenarios; it is not hidden by increasing a tolerance.

The coupled demonstration uses a deliberately tiny 2x2x1 filled box with an exactly symmetric dyadic force pulse, then eight unforced intervals. Both pressure implementations publish equal numerical fields, zero pressure, constant liquid volume 4 m³, and phase fraction exactly one. With mu = 0.125 Pa s at rho = 1 kg/m³, final accepted kinetic energy is 0.00222748 J, compared with 0.00691109 J at zero viscosity. Advection also damps this case; the comparison is not an isolated analytic decay or a large liquid property simulation. Native grayscale PNGs show the declared velocity slice, not a surface reconstruction or photorealistic material render.

The independent Python gate rejects nonfinite fields **before** maxima, derives modal decay, checks native fields and pixels, energy identities and budgets, phase balances, accepted time/version agreement and all reported capacities. Its adversarial tests cover 23 variants under normal Python and `python -O`. Complete source/evidence Git bindings also include nested receipts; only the root receipt is excluded from its own self-hash map.

## Conditional Lean claims and outstanding prerequisites

The new `BoxViscosity.lean` companion proves ten exact-real statements. Three concern the explicit unit-spacing 2x2x1 MAC patch: symmetric work, nonnegative dissipative quadratic form, and bilinear symmetry. The other statements are local rigid-rotation shear cancellation, affine-shear dissipation, exact coordinate explicit and rounding energy identities, energy nonincrease **given** its scalar identity and bound, scalar modal contraction **given** 0 <= r <= 2, and one modal step. All declarations are audited; injected `sorry` and extra-axiom variants must fail. The historical implicit finite-strain theorems and their inventory remain unchanged.

These statements do not prove assembly for arbitrary grids, the full mesh spectral bound, solver/IEEE refinement, divergence preservation after f32 storage, spatial/time convergence, or physical material calibration. Local rigid-rotation strain cancellation does not imply a rotating field is admissible at stationary box walls. No global angular-momentum or rotating-drop claim is made.

Moving free surfaces still require compatible velocity masses, strain-volume quadrature, interface geometry, normal/tangential viscous traction and pressure/stress coupling. The current column pressure geometry and vertical air-band copy do not provide those quantities. The full traction condition is essential, as discussed in the original [Batty–Bridson free-surface viscosity paper](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf); the [Basilisk viscosity source](https://basilisk.fr/src/viscosity.h) provides another primary coupled-stress implementation. Their validation does not transfer to this stencil. No-slip adhesion/Couette/channel tests, irregular solid contact, variable density/viscosity, implicit high-viscosity steps, surface tension, arbitrary interface reconstruction and calibrated physical liquid behavior remain separate requirements.

The branch composes frozen column source/evidence `1b4cc3bd684e1bb46012c6b8ef908b3cd18fddde` / `5968e8d15c8cc7874fb888a58f2d66678c0ba935` with the accepted Pillow-only CI change `96356512a3228ddf743c32ca917e4e53ab826aac` by a normal two-parent merge `0968860811cdaf599247cac9d16c88622f7bbdf7`. No frozen source, research chapter, PDF, proof inventory or evidence packet is rewritten.
