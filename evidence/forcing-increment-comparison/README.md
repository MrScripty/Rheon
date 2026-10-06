# Frozen prototype for one bounded comparison

This experimental branch preserves `704842c368953e47df279728935e20cfda89f0c5`
and all prior failures. Source is frozen before compilation/conformance, as
requested. `prototype-policy.json` binds the test-only native copy, exact-bit
inputs, 106-bit fixed scalar, borrowed workspace and inherited comparison
policy. No production Rust, owner, acceptance or trajectory is changed.

`native.rs` is a private test-only copy of the existing evaluator with one
optional borrowed chart workspace. Disabled mode retains the original chart
solve. Enabled mode solves the increment against accepted coefficients using
106-bit nearest-even scalar operations, stores one binary64 coefficient target,
then recomputes native geometry/embedding/inspection/transfers/forces/rate/norm.
The differentiated acceleration solve stays native. Only the first 15-variable
chart solve is substituted. The accepted qB path/geometry policy is unchanged.
The selected absolute block and full measured constraint gates remain required.

`scalar.rs` uses two fixed u64-equivalent integer limbs for the significand,
an exponent/sign and explicit padding in a 32-byte Scalar. All nonzero results
must fit the binary64 normal magnitude envelope. There is no precision fallback,
clipping, heap multiprecision or retained tail. The actual ChartWorkspace owns
all 1,934 reserved scalar slots plus fixed accepted-coefficient extraction and
descriptors. The native Work borrows it; no second physical state owner exists.

Preflight must pass scalar ties/range/cancellation-zero/underflow/pivot/endpoint
checks, an independent exact Fraction oracle, actual sizeof/layout, and compiled
stack metadata for the additional scalar/chart kernel under the declared
2,048-byte stack ceiling. Baseline native reservations remain required. No E1
may execute before this evidence passes. A genuine conformance failure stops
the task without a repair/search/retry for a passing case.

The sole allowed comparison pins pressure-forward h=.00078125, accepted version
1/time .00078125, and the original terminal unknowns/norm1.1895444217920825e-13.
B0 checks all captured floats with `to_bits`, including signed zero, before E1.
A0 is exact arithmetic on identical frozen binary fields; E1 changes the chart
solution and fully recomputes the downstream native equation. Their results
must remain separate. Planar qualification is an isolated diagnostic, never
a third-component solve, owner advance, public-call acceptance or override.

Apply the archived library test overlay only in this isolated checkout, use a
dedicated minimal release target, and restore `src/lib.rs` byte-for-byte after
capture. The overlay and native copy are research artifacts; production is
preserved. Formatting initially rejected missing integer parts on temporary
Rust literals; those syntax errors were corrected before this first source
freeze. No numerical conformance or native comparison had run at that point.

The preflight/comparison outcome, actual memory proof and post-Git bindings are
added later without rewriting this source milestone or previous evidence.
No numerical floor, general liquid/3D/surface/material claim or new Lean theorem
is implied. Original refusals and temporal-band failures remain authoritative.
