# 15 Exact mathematics and finite arithmetic

Lean real numbers are not IEEE floating values. Exact cancellation does not imply identical bits from reordered reductions. Goldberg is a foundational reference for rounding and exceptional values [S18]. The proof-to-runtime gap requires explicit analysis and tests.

## Rounding

For suitable finite normal values, rounding assumptions and no exceptional range behavior, a common model is

\[
\operatorname{fl}(a\circ b)=(a\circ b)(1+\delta),
\qquad |\delta|\leq\varepsilon.
\]

Zero results require absolute-error interpretation; underflow, overflow and NaNs need separate treatment. For n-term accumulation, \(\gamma_n=n\varepsilon/(1-n\varepsilon)\) is a standard bound factor when \(n\varepsilon<1\). Cancellation can make final relative error large despite small absolute error.

Pressure subtracts nearly equal velocities near rest. Report absolute divergence and physical energy scales alongside relative residual. Every normalization needs a denominator and zero policy.

## An executed arithmetic comparison

The companion's binary64 residual identity discrepancy is approximately \(2.49\times10^{-15}\). Replaying the algebra with binary32 inputs and operations gives \(1.70\times10^{-6}\). These are fixed-problem observations, not universal bounds.

The binary32 replay rounds the binary64 pressure; it does not run a binary32 CG solver. Its purpose is to demonstrate arithmetic discrepancy in a formally exact identity. Production binary32 needs its own convergence, breakdown and residual-drift qualification.

## Invalid values and finite range

Validate finiteness explicitly before range checks for spacing, time, density, viscosity, cameras and initial data. NaN ordered comparisons are false. Negative zero may affect serialization or hashes even when harmless numerically; define canonicalization only if the replay contract needs it.

Do not silently replace nonfinite state with zero. Report stage and version, preserve accepted state and use only an explicitly authorized degraded-preview policy. Otherwise the intervention hides the cause.

Natural-number indexing omits machine range. Check dimensions, face counts, byte counts and offsets before allocation, then enforce resource limits. Huge finite world coordinates can overflow integer conversion or lose subcell precision. Clamp only under a declared boundary sampling policy; otherwise reject out-of-domain.

## Tolerances

Pressure convergence, geometry classification, interpolation bounds and test comparisons need separate tolerances with units. A global epsilon cannot serve all. A tolerance changes the accepted mathematical set and should be documented.

Use exact rational or higher-precision small oracles. Test both sides of thresholds, zero step, minimum shapes, empty liquid, blocked domains, tiny coefficients and near-contact geometry. Derive tolerances from error models or accepted geometric budgets; never enlarge them merely until a visible defect passes.
