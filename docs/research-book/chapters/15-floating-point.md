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

## A unit lost in a reduction

Take A = 2²⁴ and sum A, 1, −A. The browser explicitly applies Math.fround to
each input and addition in its binary32 mode. The trace becomes (A, A, 0),
so the executed sum differs from the exact result 1. In binary64 mode the same
finite terms give 1. [ECMAScript's Math.fround specification](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.fround)
defines conversion through binary32 with ties-to-even rounding. Python's
independent oracle uses struct conversion to check every trace element.

## Cancel the large pair first

Change only the order to A, −A, 1. The binary32 trace is (A, 0, 1). Inspect
both bars at the same final-sum scale. This is a concrete counterexample to
transferring the Lean exact-real associativity identity to floating-point
reductions. Parallel reductions may choose other orders; compiler flags,
precision, exceptional values and the executed reduction tree are part of
the implementation contract. These finite checks do not bound a complete
solver's accumulated error or license reassociation of production code.

<details><summary>Practical study proof scope</summary>

[PracticalStudy.lean](../../../proofs/Rheon/PracticalStudy.lean) states the
exact real-arithmetic premises. Kernel qualification for these new statements
is pending. The independent finite oracle checks the browser formulas; it does
not prove a Rust refinement, IEEE solver, continuum prediction or runtime speed.

</details>
