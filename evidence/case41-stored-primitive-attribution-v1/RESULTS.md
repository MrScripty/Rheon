# Bit-bound archived force/donor attribution

## Frozen provenance and new scope

Analysis source/input precheck: `83cfe02a5857cf7465cc043a4a5cc11268bc2dd8`, tree
`6cdf48928bc86ec7c7a6adcc3548d2b40e7fdf92`. The two initial external readers
failed exact reconstruction because supplied helper inputs mixed floats and
Fractions. Their source, protocols and failed outputs are preserved in
`attempt-01/02`. The corrected source explicitly converts both velocity and
provided donor arrays to exact Fractions; rational-output checks and complete
vector reconstruction pass. No numerical refusal or native retry occurred.

Current numerical source is unchanged
`8cc37c4473c723c3436e6be5d859af90695c2c37`; original preflight
`fb710371326d552fe91bafff2995340758d14ed0`, execution
`c2c583180f23316c6ac2ac89da434bf313121980`, result
`455af0e35c25037d2ff904bb92fd71b5d6b1f5c9`. Its stdout SHA-256 is
`8a7ec54e86e0f6ec4686bc5eeca93856e54a1b8a7972eafc82cc295061371287` and
ELF SHA-256 is
`c483c840f34d60c58eeb01fbaeec7e745edf817e84ab02655e44b7e2900a3a07`.
The ELF is neither needed nor executed by this analysis.

Archived E2 native source is `cbeaa72b403cb1f3af4f95727cc29ec660c99ca3`, preflight
`c2ee6acc8a783161a844e3dee0566af71732628d`. Its retained `candidate-fixed.log`
SHA-256 is `6b4d8a9fc1a0adf1503f191e7b9b6b60f53e3904b2066d9e481bf4c06aa06894`.
`protocol.json` binds both complete raw journals, original inputs, equation source
and every imported helper. The result lists original line numbers and raw-line
hashes (UTF-8 bytes excluding newline) for the baseline and 20 E2 records.

The complete original fixture matches bit-for-bit. Twenty-one retained equation
fields match the current baseline. The E2 start chart/velocity/mass/q/eta and end
chart/velocity/mass/force/q/eta/D match exactly; the forty-face topology matches
at all retained points. Sixteen donor quadrature samples are present and their
geometry/known-coordinate arithmetic matches the original graph. Native force,
donor and stored rate arithmetic replay bit-for-bit externally. No new point,
force evaluation in the native program, quadrature or equation occurs.

Existing ranks and conditioning are not recomputed. This packet adds primitive
force/donor attribution in the original frozen dual-mass pressure decomposition,
and separates it from endpoint-counterfactual effects.

## Actual stored primitives

Hold all stored endpoints, masses, geometry and sampled fluxes fixed. Reassemble
strain/pressure from retained triangle areas/gradients and pressure terms with
exact rational products/sums and the actual stored endpoint velocity. Integrate
only the retained sixteen flux/factor samples exactly. This preserves the actual
stored state and observes differences in primitive arithmetic; it does not
recover unrounded geometry or an intended real-arithmetic finite equation.

The native residual is reconstructed **exactly** as:

    native rate = exact stored-primitive residual
                + native force assembly gap
                + native donor quadrature gap
                + native stable accumulation gap.

All 22 vector identities, exact pressure-complement orthogonality and weighted
Pythagoras pass. W=M^-1/(27/8) and P=B(B^TWB)^-1B^TW use the unchanged baseline
masses, embedding and rounded pressure columns. Signed alignments use the exact
stored stable complementary residual, not a new acceptance criterion.

| Residual/contribution | Raw force norm | W complementary norm |
|---|---:|---:|
| Native baseline vector (exact squared norm display) | 1.0688520423549863e-13 | 9.861671359583165e-14 |
| Exact replay of stored aggregate fields | 1.0688253531784157e-13 | 9.861378297182208e-14 |
| Exact stored-primitive residual | 1.0674470208181306e-13 | 9.861488961750221e-14 |
| Force assembly gap | 2.28609278878686e-15 | 2.733040931763096e-17 |
| Donor quadrature gap | 1.9231593425037044e-20 | 6.0989045035653775e-21 |
| Stable final arithmetic gap | 8.98769663910027e-18 | 5.652255921999323e-18 |

The exact stored-primitive residual still exceeds the unchanged 1e-13 Newton
gate, verified by rational squared comparison. Its complementary norm is
essentially unchanged. These measured aggregate/primitive differences do not
explain the dominant complementary residual or overturn the original refusal.
The original native sequential norm remains 1.0688520423549861e-13; the last-bit
display difference above is just the norm calculation.

The force gap splits exactly into strain assembly, pressure assembly and final
combined-force addition. Their complementary norms are respectively
1.3833123352357002e-17, 2.499314260035083e-17 and
1.7472885250722962e-18. Pressure assembly dominates the **raw** force gap
(2.283551377497765e-15); most of it lies in the pressure range. Norms do not add;
exact cross terms and signed alignments are archived.

The donor gap splits into sample-product rounding and sequential accumulation.
Complementary norms are 7.028627138636735e-22 and 5.777438696055716e-21.
Flux samples themselves remain captured rounded values: their formation error,
unsampled path and quadrature truncation are not bounded by this comparison.

The consistent primitive terms retain the large chart inertia, body force,
strain force and donor cancellations previously observed. Pressure is now split
from strain: primitive pressure-force complementary norm is
2.832733758512014e-15. Exactly the same complement is obtained from primitive
pressure force minus the frozen rounded B*Pi (raw reconciliation norm
4.354269507170467e-15). This describes the difference between two retained
operator constructions; it changes neither B nor the pressure projector and is
not a physical divergence error budget. Large opposite signed term alignments
are cancellation-sensitive algebra, not percentages of error.

## Endpoint rounding stays separate

The actual archived 106-bit working endpoint parts are decoded exactly. Their
dependent RN64 conversions match the stored endpoint. Public known eta values
retain their native binary64 graph, which differs from the working affine
values. Projecting these observed differences through M/h gives:

| Endpoint contribution or counterfactual | Raw norm | W complementary norm |
|---|---:|---:|
| Known public-versus-working inertia contribution | 3.3055060856997156e-14 | 2.29031323422258e-14 |
| Dependent working-to-stored conversion inertia | 1.5096092790306673e-14 | 6.710123273788983e-15 |
| Primitive residual with latent inertia only | 8.166901455713057e-14 | 8.030904507742744e-14 |

The exact native reconstruction additionally checks both endpoint contributions
plus the latent-inertia primitive residual and all actual arithmetic gaps.
Their signed alignments to the stored complement are approximately 0.180257,
0.017189 and 0.802564 for the two contributions and latent primitive remainder.
They are not independent physical error shares.

The latent-inertia residual falls below the scalar Newton target only in this
counterfactual. Stored donor velocities, force inputs, masses, geometry and
flux samples remain fixed while inertia changes. It is not a consistent
nonlinear state, successful correction or accepted endpoint. This distinction
already existed in E2; the new result binds it to the present baseline and adds
primitive/weighted attribution. Most of the complementary alignment remains
in the counterfactual remainder, so endpoint rounding alone does not establish
the cause of refusal or an arithmetic floor.

## Limits and replay

Exact rational identities/classifications are independent of display precision.
80/120-digit displays and normal/optimized Python agree; binding-negative
controls and clean cache-free reproduction are retained. No rank, new equation,
controller correction, owner advance, trajectory or reference integration runs.
No derivative, root distance, real-map error, arithmetic floor or relaxed gate
is established. The 66,368/67,584 memory certificate remains incomplete; all old
failed evidence and main/production are unchanged.

Use the external-output commands in `REPLAY-INSTRUCTIONS.md`. The separate cache
replay instruction fix is source `934e079f7503ac83a129de2c0eb8e348ebba9ff8`, result
`d1f2309d1e74775dd26237edb0655e3fb20df50f` on
`research/case41-external-cache-replay`; it preserves tracked old replay results.
