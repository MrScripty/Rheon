# Additive memory audit of the frozen B0/A0/E1 binary

The narrow static audit establishes a **65,456-byte aligned additional bound**
for the frozen chart evaluator and measured point/partition callers, under the
existing native baseline reservation. This is below the unchanged 65,536-byte
cap by **80 bytes**. No build, scalar probe, E1 evaluation, candidate search or
trajectory advance occurred. All numerical results and earlier evidence bytes
remain unchanged. This audit supplies missing evidence; it does not certify
that the incomplete earlier preflight established this bound before execution.

| Simultaneously charged addition | Bytes |
| --- | ---: |
| Actual ChartWorkspace, including accepted extraction and padding | 62,096 |
| Work's additional borrow descriptor | 8 |
| Positive compiled Work::point frame delta | 1,808 |
| Positive compiled Work::partition frame delta | 0 |
| Actual reachable chart-kernel stack peak, including return addresses | 1,536 |
| Subtotal | 65,448 |
| Conservative rounding of aggregate to 16-byte alignment | 8 |
| Aligned bound | **65,456** |

The previous 64,024-byte claim omitted the point caller delta. Simply adding
that delta to the earlier blanket-red-zone bound gives **65,832 bytes**, which
does not prove the cap. Keeping the original 2,048-byte kernel ceiling also
gives **65,960 bytes** after that delta. Neither old expression is promoted to
a complete proof. The present bound uses the frozen instructions themselves;
no compiler flag, binary, precision, workspace object, arithmetic policy or cap
was changed to make it fit. The original evidence retains its historical claims.

## Kernel calls and outgoing tail jumps

`audit.py` indexes ELF-sized functions by **address**, rather than overwriting
duplicate demangled names. It classifies every call and conditional/unconditional
branch by numeric destination, including indirect and outgoing tail jumps. It
walks all reachable kernel instruction paths with stack depth, checks balanced
returns/joins, and rejects unrecognized stack mutations, unresolved transfers,
recursion and reachable traps. It follows all reachable branches of the frozen
default-libc memset entry as well. An outgoing edge is resolved and charged by
the kernel reader or rejects qualification; the caller audit rejects an
unresolved/new outgoing edge. **No outgoing jump was found** in the actual
seven kernel functions, four point variants, two partition variants or enclosing
comparison function. Internal branches and generated bounds-panic calls are
recorded explicitly, rather than confused with tail calls.

The maximum path is solve -> factor -> mul. Their actual stack bounds, including
their return addresses, are **736 + 672 + 128 = 1,536 bytes**. Solve/factor/mul
use no memory below their current rsp. Div is a leaf with 48 saved-register
bytes, 26 bytes below rsp and an 8-byte return address: **82 bytes**. Add is
48 bytes including its return address; conversions are 8 each. The frozen
default memset is an 8-byte-return-address leaf with a closed 99-instruction
reachable graph. Blanket 128-byte allowances on every non-leaf function are
unnecessary for these observed instructions. Positive indexed local accesses
retain the frozen private fixed-array/normalization invariants. Impossible
private bounds-panic paths are excluded on that same source basis; this remains
a finite source/machine-code audit, not a formal proof of arbitrary invalid
private scalar representations, panics, unwinding or asynchronous interrupts.

## Caller frames and simultaneous reservations

All four frozen point variants have a **27,856-byte** frame bound. All four
point variants in the preserved native diagnostic binary have **26,048-byte**
bounds: a **1,808-byte** positive delta. The diagnostic overlay leaves the
original point and partition source bodies unchanged. The prototype's point
body differs only at the archived private chart hook; partition source is
identical. These source equalities and hashes are replayed by the auditor.

The frozen partition variants have **74,048-byte** frames; native reference
variants have **74,080-byte** frames. No 32-byte shrink is credited. No credit
is taken for replacing the native linear-solve frame, nor is kernel memory
charged into unused baseline slack to evade the additional cap.

The enclosing frozen comparison has a **330,336-byte** fixed frame, including
LLVM's bounded stack-probe loop. The single workspace begins at rsp+237,232 and
ends at rsp+299,328 inside that frame. The direct initialization and pointer
stored into Work are archived instruction bindings. No additional workspace
copy or accepted-state tail is introduced. The maximum enclosing arithmetic
prefix is comparison -> partition -> point -> chart kernel: **433,776 bytes**.
After removing the explicitly charged additions, its baseline portion is
**368,328 bytes**, within the unchanged **392,416-byte** baseline step-stack
reservation with 24,088 bytes remaining. The actual workspace in the enclosing
frame is counted once, not again as a separate allocation in this prefix.
Other unchanged native helper paths retain their existing baseline reservation;
this is not a fresh proof of the whole production controller or host logging.

The existing forced nominal baseline remains **474,768 bytes**, including
baseline step/third/force reservations. Reserving the full authorized additional
65,536-byte region alongside it gives **540,304 bytes**. The previous combined
538,920-byte expression omitted the caller growth and is not the complete new
simultaneous reservation. Controller, third-component, lifecycle/restart,
cancellation/retry and future compiler/platform qualification remain pending.
An 80-byte margin is not a portable robustness guarantee.

## Frozen identities and provenance limits

The audited E1 binary is the existing file at
`/workspace/.rheon-tools/increment-comparison-staged-target/release/deps/rheon-6fcb0e06fe041f73`,
SHA256 `6ce8f08803c74654433093559e8ca0ce2c02586aff1999ffaf7bf40a59b744d0`.
Its source freeze is `78b7365513ccf317e554d36f968da36b55f04065`; numerical
evidence is `4326926f826bcda9ff7b483605a86a11becb6c98`, with receipt SHA256
`e32b08711a94747b93c706ae04cf5d7eb11fa94b75aa81a86b1369c327ed75c0`.

The native reference is the preserved release binary identified by the frozen
arithmetic diagnostic logs, `rheon-ab0162a1a0153f51`, now SHA256
`aabec4cec12ed659d954a6c1e214e6623924b48094a069b671b92c7604db0b50`.
Its hash is **first recorded in this audit**; a historical pre-execution binary
hash is not invented. The exact inspected reference bytes and source mapping
are bound here. This comparison establishes the measured frame delta between
these two retained binaries, not compiler equivalence across all builds.

The audit packet binds ELF sizes, relocations, exact relevant disassembly,
instruction stack-depth records, all transfer edges, source mappings, actual
normal/optimized reader outputs and additive post-Git checks. Negative controls
cover an outgoing tail jump, unresolved indirect jump, hidden callee, increased
stack frame, unbalanced return, dynamic stack mutation and an omitted caller
delta. All earlier tracked files are preserved. B0/A0 failures, changed-field
E1 results, original five refusals and temporal-band failures keep their prior
classification. No public step, adoption or new numerical qualification follows
from this audit.
