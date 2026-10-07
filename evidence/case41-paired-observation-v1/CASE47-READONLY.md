# Independent archived case47 check

A parallel read-only worker inspected the fetched comparison branch at
**24e231d8544bc1f981704acdafe090f24d7687f4**. It verified all 94 archived input
bindings and 20 result-artifact hashes, normal/optimized reader parity, and
identity of the native log/receipt against accepted trajectory publication
**26154d9a32e193b35d9cda0b02b2d7c5f770cfeb**. No file was changed and no native
runner, reference integration or new reference computation ran.

Original pressure_state/nonconstant/reversed model, canonical initial state,
native initial bits and nominal endpoint 128*0.00078125=0.1 match. The retained
native clock is 0.09999999999999977; no interpolation or retiming was applied.
The original fine-endpoint mass weights and original metrics are retained.

| Original metric | Fine reference | Coarse reference |
| --- | ---: | ---: |
| Full velocity lumped L2 | 6.288313524301006e-05 | 6.288313569841868e-05 |
| Third velocity lumped L2 | 6.288070229134095e-05 | 6.288070260070558e-05 |
| Maximum cap-coordinate error | 2.2147831915675376e-08 | 2.2149030540208337e-08 |

Coarse/fine full-velocity reference gap is 2.4582511771244623e-11, reproducing
the original value exactly and passing its strict comparison against full
endpoint error/1000 = 6.288313524301005e-08.

Reader source: b7af5b24fb020a0b6eafd0f6d546976f6845be66,
tree aafbbb27ccbdf55132d1ecab56b71e0abb8df8ad.
Native compiled source: e37b0bb34ace62f16dc931488146d6a396b27be0.
Native preflight/execution source: 529707429fb092beb4df9a278d978bf72e8612cf.
Archived tested numerical result: c20aa84e2de27a6537d4685c102c640d8f87dba4.
ELF SHA-256: 6e6ea0505c81cbf8624b177c6c782ff8cb29cc0830c0524e79088e8307a10989.
Paired reference publication: 40c25f9eca37b66e7e24bee5a8d5f6aa995e2196.
Reference execution Git commit was not recorded; driver/data hashes provide
provenance without inventing an execution identity.

The accepted outer supplement 30a312eb77b3f3a5b5aebf8af1cfb848b7257d82 is
67,424/67,584 under its existing additional-memory scope. Historical 67,088
accounting remains incomplete. Endpoint error is not temporal order. Eight
original geometry-band failures, broader refinement and retry coverage remain
unresolved.

The underlying archived packet is at
[case47 comparison](https://github.com/MrScripty/Rheon/blob/24e231d8544bc1f981704acdafe090f24d7687f4/evidence/case47-archived-endpoint-comparison-v1/RESULTS.md).
