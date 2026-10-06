# Exact constant-parcel remap correction

Frozen source c36d99df1b5ee9dfea07f21d4204caab143e307f and evidence
6b0249aef6b6c7ecdbdf1a4f0f717064b27dfdad are retained. Their ordinary-scale
constant-field rejection is reproduced by two actual Rust regressions in
`original-native-rejection.log` (exit 101), then corrected in a successor.

When every positively weighted donor in a target row has exactly the same f32
value, its mathematical weighted mean equals that value. The implementation
copies it without a redundant rounded division. This neither clips a candidate
nor changes a tolerance. Row-mass partition, all extensive ledgers, their
measured residuals, scale rejection and final cancellation remain checked.
The independent Python oracle uses the same exact-real constant property.

The two tests use (1) spacings [1,0.1,0.1], Y height 2.25→2.6 cells, density 1,
both components 3f32; (2) unit spacings, Y height 2.25→2.7 cells, density 1,
first component 0.1f32 and second zero. Both retain zero mixing and rounding
work. Before correction, the two rounded averages respectively lie just above
and just below their singleton donor hull, despite unchanged f32 stores.

Qualification uses the actual Rust feature matrix, normal/optimized Python,
pinned positive/negative/restored Lean audits (unchanged twelve conditional
identities), and fresh native replays. `legacy-replay/receipt.json` binds complete
byte equality against the retained frozen fixtures; redundant fresh outputs
were produced under /tmp and are not duplicated in this packet.

The model and all limitations in the frozen column-momentum book companion
remain. No MAC mapping, pressure compatibility, varying-height strain or liquid
simulation claim is added by this correction.

    python3 evidence/column-momentum-constant/verify.py --negative-self-test
    python3 -O evidence/column-momentum-constant/verify.py --negative-self-test

Add --evidence-commit COMMIT for root receipt and full Git inventory binding.
Only the root receipt is omitted from its own SHA map; nested receipts remain.
