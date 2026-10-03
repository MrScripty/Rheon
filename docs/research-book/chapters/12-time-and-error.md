# 12 Time stepping and error budgets

Physical time and display frames are separate clocks. A frame can contain substeps; a slow frame must not silently enlarge the physical step beyond its accepted regime. Decide whether overload slows simulation, drops presentation frames or changes a declared quality tier. Record the policy in replay metadata.

## Restrictions

Directional Courant numbers are \(c_x=|u|\Delta t/h_x\) and analogues. Multidimensional positivity depends on the update and can involve their sum. Characteristic transport may remain bounded without that restriction, but travel limits still help trajectory accuracy, collision detection and narrow-band validity.

Explicit diffusion depends on \(\nu\Delta t\sum_dh_d^{-2}\). Surface tension introduces a capillary scale proportional to \(\sqrt{\rho h^3/\sigma}\), with method-dependent constant. Acceleration and moving walls add constraints. Return a step and its limiting reason, not an unexplained number. If required substeps exceed a cap, return a typed outcome rather than label a partial interval complete.

## Error accounting

A conceptual budget is

\[
E_{\mathrm{total}}\lesssim E_{\mathrm{model}}+E_{\mathrm{space}}
+E_{\mathrm{time}}+E_{\mathrm{solve}}+E_{\mathrm{round}}
+E_{\mathrm{surface}}+E_{\mathrm{render}}.
\]

This is an accounting guide, not a universal inequality across incompatible norms. Residual controls one constraint error, not advection displacement or mesh extraction. Machine-precision pressure with a coarse staircase obstacle may overspend on a subdominant error.

Refine space at fixed Courant number for smooth transport. Refine time with sufficiently small spatial error for temporal order. Simultaneously changing both mixes effects. For interfaces measure volume, surface distance and silhouette displacement; small average error can hide catastrophic filament loss.

## Manufactured solutions

A periodic stream function \(\psi=\sin x\sin y\) gives velocity \((\partial_y\psi,-\partial_x\psi)\) with zero continuous divergence. Sampling it does not automatically give zero discrete divergence; a discrete curl construction can ensure cancellation. Distinguish those tests.

Choosing discrete pressure \(p\), computing \(b=Ap\) and solving tests the solver for that operator, not the operator's physical accuracy. Add independent analytic and stencil checks. Expected values should not reuse the same indexing routine whose correctness is in question.

## Long horizons and outcomes

Track volume or tracer amount, energy, divergence, surface position and rejection count after sources stop. Include rest, translation, rotation and eventually liquid and obstacle cases. Sensitive flows may not match pointwise indefinitely; use bounded time windows and defined geometric or statistical observables. Exact replay remains a separate implementation property.

Outcomes should distinguish converged, iteration limit, invalid input, nonfinite state, incompatible flux and resource limit. A nonconverged preview may be permitted explicitly but must not masquerade as accepted scientific state. Candidate and accepted buffers need clear ownership so cancellation cannot corrupt future steps.
