# Captured linear predictions; no nonlinear step

## Reused evidence, not a new conditioning result

The seven-equation result is frozen at
`455af0e35c25037d2ff904bb92fd71b5d6b1f5c9`, published at
`7f7f9b62d16ff3c13a0112aac46a29ad4ddeec15`. Its model reader source
`95f620f5a47b7c9c9afd6eab4d5e9940049f09cf` already established exact captured
ranks A=6, B=16, J=22; raw condition 2109.073370010061; primary physical-scaled
condition 2109.048606567285; dual-mass condition 2196.3137756715005; and
pressure-complement acceleration condition 3.8273784368345427. The existing
80/120-digit normal/optimized diagnostics agree. This packet imports those
results and does not repeat SVD, rank or singular-direction analysis.

Unchanged numerical source is `8cc37c4473c723c3436e6be5d859af90695c2c37`, original
protocol/preflight `fb710371326d552fe91bafff2995340758d14ed0`, single execution
`c2c583180f23316c6ac2ac89da434bf313121980`. ELF SHA-256 remains
`c483c840f34d60c58eeb01fbaeec7e745edf817e84ab02655e44b7e2900a3a07`;
raw stdout SHA-256 is
`8a7ec54e86e0f6ec4686bc5eeca93856e54a1b8a7972eafc82cc295061371287`.
Neither ELF nor an equation is executed here. All original result-inventory
hashes and the imported exact-algebra helper hashes are checked before analysis.

## New analysis and matrix

New captured-only analysis source/input precheck is
`fb31d368b1cd35178b299bfd7bd685828b2b8ee5`, tree
`0c165ff919a88f55828edccb9221ac7b7e34205b`. Earlier successful read-only displays
from source `1d2e4ea589f2b9282dc3b180bb60a6ec743c7aa2` are retained under
`pre-enclosure-readonly`; the forward amendment adds exact rational enclosures.
It changes no captured input or numerical program.

`matrix.json` exports all 484 original binary64 entries as hexadecimal bits,
coordinate scales, nominal denominators and actual/nominal offset ratios.
The analysis reconstructs A from the six emitted native columns and B from the
baseline endpoint `end_b` transpose, then checks equality with the published
model. Every denominator remains the original nominal delta. The declared
scales remain C=diag(1 six times, 27/8 sixteen times), F*=27/8,
Jhat=J C/F*, rhat=r/F*, shat=C^-1 s.

The new calculation solves **only the exact captured rational linear system**
J s=-r outside the numerical program. It neither calls the native `linear`
routine nor constructs/re-evaluates a native candidate. The exact inverse and
zero predicted linear residual are checked rationally; this zero is an algebraic
identity, not nonlinear convergence.

| New projection diagnostic | Value |
|---|---:|
| Scaled displacement norm | 2.390532565550451e-13 |
| Acceleration displacement norm | 2.3501664580452406e-13 |
| Pressure-coordinate displacement norm | 1.476397854217691e-13 |
| Model residual after rounding displacement alone to binary64 | 2.2137885935829813e-29 |
| Componentwise backward error of that rounded mathematical projection | 2.765105484488647e-17 |
| Model residual using forecast representable coordinate increments | 4.2915669718609846e-15 |
| Componentwise coordinate-quantization bound, 2-norm display | 5.4528491538762554e-15 |

The backward-error number belongs to rounding the exact projection, **not** the
original native LU solve or a new controller correction. Forecast increments
are round64(x_i + round64(s_i)) - x_i, computed independently by exact binary
rational arithmetic with correctly rounded conversion. They are never supplied
to an equation. Projection magnitudes range from about 4.20 to 109713 coordinate
ULPs, so no unknown's projection is wholly below its coordinate spacing.
The ideal q displacements from h^2 s[0:3]/2 are approximately
(-3.9841e-20, -5.8082e-20, 2.1073e-21). Ideal known-velocity changes h s[0:6]
are archived. Those are frozen-formula predictions; geometry/endpoint rounding,
chart sensitivity, force reconstruction and nonlinear remainder remain absent.

## Original refusal versus stored-field contributions

The original native baseline norm remains 1.0688520423549861e-13, above the
unchanged 1e-13 Newton gate and below the 1e-11 physical rate gate. Norm displays
below instead round the exact rational sum of squared captured components; its
native vector norm displays as 1.0688520423549863e-13. This last-bit difference
comes from the norm calculation and does not change classification.

| Captured residual or isolated contribution | Force norm | Predicted scaled displacement norm |
|---|---:|---:|
| Native stable baseline | 1.0688520423549863e-13 | 2.390532565550451e-13 |
| Exact stable replay of stored binary fields | 1.0688253531784157e-13 | 2.3903344472404234e-13 |
| Native stable minus exact stored replay | 8.98769663910027e-18 | 7.293311955791525e-17 |
| Selected-constraint inertia contribution | 6.498663179170984e-14 | 6.775925250853049e-14 |
| Exact replay minus that inertia contribution | 1.0476852236284189e-13 | 2.5116530097411543e-13 |
| Exact stable minus exact direct stored replay | 3.7682219008410597e-14 | 8.249193960353791e-14 |

The gap between native stable accumulation and exact replay of **already stored**
fields is small relative to the 6.8852e-15 gate excess. It excludes errors in
upstream chart/force/donor construction and cannot be called the error of the
intended real-arithmetic equation. Exact stored replay itself still fails the
gate, verified by an exact squared comparison. Native direct norm remains
1.0732717109648156e-13; exact direct replay has norm 1.391658801300386e-13.
This distinction illustrates the sensitivity of direct stored momentum
cancellation; it does not replace the stable equation or supply a tolerance.

For the constraint diagnostic, hold captured D and the seven known endpoint
values fixed; reconstruct the exact solution of its fifteen original selected
rows. The stored endpoint differs by norm 2.234533127017323e-16. Its inertia-only
contribution is (M/h)(z_stored-z_selected_exact). All force, transport, mass and
other fields are held fixed. The original native maximum constraint observation
is 1.7587233075043104e-15; exact endpoint D*z maximum is 1.1420029359003704e-15.
They are distinct diagnostics. Selected exact rows become zero; unselected rows
still have maximum 9.992032370004936e-16, so full exact constraint satisfaction
is not claimed. The adjusted frozen-field residual remains above 1e-13.

Contribution norms are not error shares: vector cancellation matters. The
script checks exact residual and projected-displacement decompositions and
weighted pressure/complement Pythagoras, and archives the exact inertia cross
term. In the existing dual-mass metric the native pressure complement is
9.861671359583165e-14, versus pressure-range 3.3249153282522226e-14. Its stable
stored replay complement is 9.861378297182208e-14; removing the selected-row
inertia contribution leaves complement 9.781353428946825e-14. Constraint
reconciliation alone does not explain or remove the refusal in these frozen
comparisons. These values do not justify a new stopping criterion.

## Precision and rounding limits

All inverse/identity/constraint calculations use exact rational arithmetic.
80/120-digit mpmath is used for norm displays, whose binary64 outputs agree;
normal/optimized Python outputs agree. Existing SVD numbers are reproducible
high-precision displays, not interval-certified singular values.

The scaled Frobenius gap from native FD arithmetic to exact differences of the
captured native rate floats is 2.5721885209618395e-17. The gap to FD differences
of exact replayed stored fields is 2.0939715446148845e-10. Using the exact inverse
Frobenius squared norm yields a conservative captured-model inverse bound
132.6968633326093 and Neumann product approximately 2.7786345587813412e-8.
Exact rational upper enclosures (grid 2^-80) verify the product is below one.
This certifies perturbation control between **these two captured matrices only**.
It does not bound the missing derivative of a real smooth map.

Changing only A to the exact stored-field FD model changes the projected scaled
displacement by 3.181847610623441e-22; a conservative bound displays as
6.642416585099188e-21. Evaluating that alternative linear matrix at the native
projection predicts residual 1.0166339356885916e-22. Neither comparison supplies
FD truncation, upstream evaluation, smooth-neighborhood, nonlinear remainder,
root-existence, arithmetic-floor or physical-state error bounds. The forecast
4.2916e-15 residual is therefore not a successful nonlinear step. Full rank and
moderate complementary conditioning do not overturn the original exhausted
correction budget or authorize an eighth correction.

## Smallest next experiment proposal (not authorized or run)

After independent review accepts the captured packet, the smallest direct test
of this missing local response is **one separately frozen fixed-candidate
order16 observation** at the archived binary64 coordinate-lattice forecast.
Keep the original accepted inputs, equation, h, geometry/load/materials,
nominal-FD model and gates. No new Jacobian, Newton iteration, owner or
publication belongs to it. Compare its full stable/direct rate and constraint
observations with the linear forecast, retaining every mismatch or refusal.
This is a new diagnostic fixture; it cannot be inserted into the historical
seven-correction run or described as rescuing that refusal.

A separate source/ELF and input-bit binding would be needed: the current frozen
ELF has fixed baseline-plus-six inputs and must not be coerced into this job.
Freeze a single-observation protocol and either an independently accepted full
memory preflight or separately authorized measured-resource protocol. Any
baseline mismatch, crash, allocation failure, truncation or numerical refusal
ends it with no retry. This packet authorizes and performs **zero** native
observations; the proposal is not an execution request. If performed later,
one sample could test the forecast locally but still would not establish a
smooth derivative, root neighborhood or general arithmetic floor.

The old 66,368/67,584 resource certificate remains incomplete. Main, production,
all old evidence and all original failure outcomes are unchanged. No PR is
created. Reproduce with `python3 analyze.py 80`, `python3 -O analyze.py 80`, and
both forms at 120 digits; `verify.py` checks frozen bindings and display parity.
