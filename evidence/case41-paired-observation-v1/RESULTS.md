# Paired wrapper qualified for scalar/layout only; complete preflight blocked

Compiled numerical source **8cc37c4473c723c3436e6be5d859af90695c2c37**,
tree **b8af864c8f5b4e944d2a086a36f8a791d8b7bfa8**. Dedicated ELF SHA-256
**c483c840f34d60c58eeb01fbaeec7e745edf817e84ab02655e44b7e2900a3a07**.
Formatting, release compilation and Clippy passed in an isolated restored
worktree. Three preceding preparation failures are retained separately.

Four native scalar/layout tests passed at guard source
**92e825acfdbb7c40c3a05af917c4a754b0f2f10a**: 23 scalar assertion groups,
36 independent exact-rational scalar probes, six affine probes, existing type
layout and the new pure paired/allocation layout. The native tuple layout
record caused a Python JSON parser failure after those tests; it is preserved.
The corrected reader resumed without repeating any native test.

Completed read-only preflight source **ec984ded052a2792ffc666e0bc67b007a69c706a**,
tree **30da5e6b26743e7a837ea0cc656c268e813fa42a**. Normal and optimized Python
memory ledgers are byte-identical. The frozen inputs, original E2 residual bits
and equations are unchanged. Neither candidate nor reference roster ran.

| Operation | Candidate frame | Reference frame | Positive difference |
| --- | ---: | ---: | ---: |
| Paired capture, including return slot | 40,816 | 40,816 | 0 |
| Numerical boundary, absorbed equation | 121,136 | 121,120 | 16 |
| Borrowed serializer, outside numerical call | 640 | 640 | 0 |
| Partition | 74,128 | 74,080 | 48 |
| Point | 27,936 | 26,048 | 1,888 |
| Closed scalar Result-path peak | 1,696 | New arithmetic | 1,696 |

The maximum known positive crate route is 3,648 bytes. The entire candidate
outer frame is 62,192, including the 62,096-byte workspace and 96 bytes of
remaining storage/pushes/return address. Adding the explicit 528-byte duplicate
buffer charge yields the new **66,368-byte known subtotal**, leaving **1,216**
unallocated bytes below the unchanged **67,584** cap. No shrinking-frame or
nominal-baseline credit is used. This is not a complete bound or a cap pass.

Actual layouts are Work 7,808/7,800, Equation 30,976 on both sides, Result
30,984, Point 14,552 and each fixed FD buffer 176 bytes. Both full capture frames
remain in the audit. The numerical boundary's crate graph reaches no borrowed
serializer or stdout print operation. Equation-return storage is reused and
observation functions borrow it; no full observer Equation/Point copy is added.

The common geometry constructor's 14 requested capacity payloads sum to
**23,584** bytes. Actual argument shapes/lifetimes, fixed topology, pinned Rust
allocator sources, callbacks and fixed sort storage are described in
COMMON-COSTS.md and its forward recursion clarification. Geometry split recursion
is explicitly bounded at two frames, including the second frame's children.
The bounded absolute crate-only route is 266,184 bytes; it excludes linked
library costs and is not an absolute process bound. The paired stable-sort
entrypoints are the same actual linked addresses; their complete first-boundary
fixed frames, callback transfers and library frame inventory remain archived.

The candidate crate graph retains 309 linked non-crate transfers and 63 dynamic
transfers; reference has 297 and 61. Both outer transfer lists are additionally
retained. These counts do not include transitive closure of all linked libraries.
Unknown costs are explicit, not zero. No complete library/allocator/panic/I/O
closure or independently accepted common-operation certificate exists.
**FD execution remains forbidden.** Closing those costs and independent complete
preflight acceptance are required before the first baseline. That later baseline
must reproduce all 22 frozen E2 rate bits exactly before perturbations begin.

This is a separately published observation candidate, not a numerical result,
cap-overrun finding, numerical refusal or solver repair. Production, main,
equations, thresholds, cap, iteration limits, seven-evaluation roster and nominal
FD denominators are unchanged. Zero equations, corrections, owner constructions,
owner advances, integrations or automatic numerical retries occurred. All old
failed evidence and the historical incomplete subtotal are retained.
