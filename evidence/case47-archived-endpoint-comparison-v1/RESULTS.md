# One archived pure-E2 endpoint against the original paired references

The comparison passes with identical normal/optimized Python results. Reader
source is b7af5b24fb020a0b6eafd0f6d546976f6845be66, tree
aafbbb27ccbdf55132d1ecab56b71e0abb8df8ad. The initial source guard failure at
d5b2d7863410879c34ab7f791360ad6ed8a18964 is retained: the sole helper difference
was the already reviewed seven-line E2 arithmetic selector addition. The reader
now requires exactly that addition and identical remaining helper bytes. No
physical-model mismatch or endpoint metric preceded the guard repair.

Pure-E2 compiled source: e37b0bb34ace62f16dc931488146d6a396b27be0; tree
5e3fef4bdb87087acbeb7bdf56303fa9669b2b2a. Existing execution source:
529707429fb092beb4df9a278d978bf72e8612cf; tree
5badc6cb9d3a792d37a5d2a391e0d66126f0c49a. Tested archived result:
c20aa84e2de27a6537d4685c102c640d8f87dba4; tree
a45d25d85d634fd18a14f95f14e3fe2ff9f798af. ELF SHA-256:
6e6ea0505c81cbf8624b177c6c782ff8cb29cc0830c0524e79088e8307a10989.

Paired DOP853 reference publication: 40c25f9eca37b66e7e24bee5a8d5f6aa995e2196;
tree 3da767af9d6dcbed92760b049b917e85457b7d04. Reference driver SHA-256:
f0200ac9c82c7b5d7b041a8c6f2790a17946732057b3b30f80b52b2256f27b53.
Reference JSON SHA-256:
72dcf8080b10a9c2d7679041c65203e53faf9b9879dbb6fe2d1938e15ad7c725.
The archived authorization binds the driver, original physical sources,
protocol and parameters. It does not record a Git execution commit for the
reference driver; the publication identity is supplied without inventing one.

Both references are pressure_state/nonconstant/reversed with acceleration
(-0.0625,0.125,-0.03125), duration 0.1, density 3, viscosity 0.05, extrusion width
1 and the same periodic Powell–Sabin microgeometry. Coarse: rtol=1e-10,
atol=1e-12, max_step=0.00625, 386 RHS calls. Fine: rtol=2e-12, atol=2e-14,
max_step=0.003125, 734 RHS calls. These are preserved old counts, not new calls.

The native constructor is bitwise identical to the original accepted case47
state. Reference canonical q/eta/third coefficients match exactly. Independent
nodal reconstruction differs by at most 1.753805434212552e-15 in velocity and
1.1102230246251565e-16 in position, within the original consistency gates. This
is the same prescribed canonical state, not bitwise equality of independently
reconstructed nodal arrays. The nominal endpoint is 128 × 0.00078125 = 0.1;
actual repeated-addition native clock is 0.09999999999999977. Neither endpoint
is retimed or interpolated.

| Original metric | Against fine reference | Against coarse reference |
| --- | ---: | ---: |
| Full velocity lumped L2 error | 6.288313524301006e-05 | 6.288313569841868e-05 |
| Third velocity lumped L2 error | 6.288070229134095e-05 | 6.288070260070558e-05 |
| Maximum cap-coordinate error | 2.2147831915675376e-08 | 2.2149030540208337e-08 |

All velocity metrics use the original fine-reference endpoint lumped masses,
with sum 3.375. Geometry uses x=(q0,q2,q1,2.25-q2), with no normalization.
The native cap coordinate vector is
(0.09916428379543038,0.9998624902169613,0.6240111578541151,1.2501375097830387).

The coarse/fine full-velocity gap is 2.4582511771244623e-11, reproducing the
original stored gap exactly. The unchanged strict check requires this to be
less than the fine endpoint full error / 1000 = 6.288313524301005e-08; its signed
margin is 6.285855273123882e-08. The coarse/fine geometry gap is
1.1986245329609346e-12 and third lumped L2 gap is 1.432683296870489e-11; those
are diagnostics without added gates.

No new trajectory, integration, FD equation, instantaneous momentum equation,
native invocation or build occurred. Only original static geometry/chart algebra
reconstructed the initial reference fixture and endpoint mass weights. The
four mismatch controls stop before endpoint comparison. The result is one
endpoint-error comparison, not temporal-order qualification, a new convergence
band, broader geometry/material qualification, repair of eight old geometry
band failures, production adoption or a Lean theorem. The accepted case47
67,424/67,584 supplement remains within its existing additional-memory scope;
this endpoint analysis contributes no memory claim. Case41's 66,816 subtotal
and its external-cost blocker remain preserved.
