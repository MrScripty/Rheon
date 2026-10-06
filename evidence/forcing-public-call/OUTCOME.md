# Public-call qualification stops at the new binary's memory gate

The new controller prototype is frozen and compiled. Scalar conformance and
the native public-call baseline pass. The candidate E1 public call and its
cancellation/retry controls were **not executed**: the rebuilt binary's
conservative aligned additional bound is **65,552 bytes**, above the unchanged
65,536-byte cap by **16 bytes**. Numerical candidate work has stopped.

This is a failed cap certificate, not a measurement that an actual run overran
memory. No cap enlargement, compiler/parameter search, alternative precision,
baseline-slack credit, replaced-native-solver credit or production adoption was
used. The earlier frozen comparison's 65,456-byte certificate is preserved and
does not qualify this rebuilt controller. Its fixed-candidate E1 result remains
a separate observation; actual E1 controller convergence and publication remain
unknown.

| Added simultaneously charged component | Bytes |
| --- | ---: |
| Actual fixed ChartWorkspace, including accepted extraction | 62,096 |
| Additional owner/Work borrow descriptor | 8 |
| Positive Work::point caller frame delta | 1,872 |
| Positive public-call caller frame delta | 32 |
| Positive seed/equation/partition frame deltas | 0 |
| New binary's reachable chart kernel peak | 1,536 |
| Subtotal | 65,544 |
| Aggregate rounding to 16-byte alignment | 8 |
| Aligned additional bound | **65,552** |

The reference/candidate public-call frames are 145,952 / 145,984 bytes.
Their actually reached point frames are 26,048 / 27,920 bytes; constructor point
variants with a different cancellation closure do not supply the comparison.
Matched seed, equation and partition frames are 56,272, 121,344 and 74,096 bytes
on both paths. The chart's solve -> factor -> mul peak is freshly computed from
the new instructions: 736 + 672 + 128 = 1,536 bytes. All call/branch edges,
including outgoing tail jumps, are classified by ELF-sized function address.
No outgoing jump was found in the relevant controller caller chains. The same
installed default libc leaf is rechecked; private impossible bounds-panic paths
retain the frozen source-invariant exclusion and are not a panic/unwind proof.

The candidate arithmetic stack prefix is 370,880 bytes; the original native
step-stack reservation remains 392,416 bytes. No unused baseline reservation is
reassigned to the additional cap. The existing per-owner nominal reservation
is 474,768 bytes. The allowed full-cap single-owner total is 540,304 bytes; this
certificate would require 540,320 bytes under its accounting. Two serial-test
owners have separate original native reservations totaling 949,536 bytes.
Host snapshots and stage counters are fixed diagnostics, respectively 1,696
and 112 bytes per object, rather than growing rollback/history buffers. This
blocked gate is not a completed whole-harness or lifecycle memory certificate.

## Actual baseline and scalar evidence

The exact original pressure_state/nonconstant/forward initial publication is
used at h=0.00078125 with whole-domain acceleration (0.0625,-0.125,0.03125),
original settings and stamp id 131. A native first public call passes all
ordinary planar, third-component, combined-work and publication gates. Its
published velocity, positions, mass, pressure, time and stamp reproduce the
original accepted prefix bitwise; all 22 reconstructed accepted coefficients
also match. This creates the local verified prefix, not a new initial/restart
approximation.

The original prefix owner is cloned through test-only derives on the existing
fixed-topology owners. Full owner fields, work geometry, existing workspaces,
pressure and clock are cloned. At most the original prefix and one active clone
are retained. The original stays unchanged after prefix construction. Native
fixed-call conformance checks the clone's accepted state on refusal. The
adapted clone with its chart **disabled** independently reproduces that same
controller call. Both reach the exact original terminal result after seven
corrections and 50 equation evaluations: norm **1.1895444217920825e-13**, above
the unchanged 1e-13 target, and return IterationLimit without publication. Their
initial second-step Newton-check records also match the frozen original.

The actual Rust scalar suite passes **23 assertion groups**. The independent
exact Fraction reader checks all **36** emitted probes. The scalar source is
byte-identical to the accepted 106-bit policy. Rust formatting, scalar/type
layout/native-baseline tests and normal/optimized static-memory readers pass.
No unrelated test/Clippy matrix or E1 full-call success is claimed.

## Source freezes and retained first preflight

Prototype `21c2d01828a5ca9f03cbc79ef3dec2e38eeba9ea`, tree
`ff805e9cf32bef02f7d61e87f10321c24c677eb6`, was committed/pushed before
compilation. Its first scalar preflight passes and its original native prefix
and fixed-call refusal match. Its adapted clone is stopped by the original
entry memory guard: the cloned allocated_bytes copied 474,768 while the new
owner descriptor requires another eight bytes. This was a test clone accounting
error before candidate arithmetic. The failed test, layouts, source, binary
hash and diagnosis remain unchanged at evidence milestone
`619aad93905d373b0bf8c61e68f088fb3a74e4f7`, tree
`5091579e575add42778847c5453b3b7b1a255e29`.

The additive v2 source at `77962729f8de389c25b3f595708ab50d4b3362c0`, tree
`b578331d1dd8e768b2ec794689f5906acb71bb56`, registers the eight-byte descriptor
in the cloned counter. Installing E1 would register the unchanged full 65,536
additional region, counting that descriptor once. No settings, loading,
arithmetic, correction budgets or acceptance/publication gates change. This v2
source and policy were committed/pushed before compilation in a **new** target;
the first prototype binary remains intact.

The v2 policy SHA256 is
`e211db43b50ca34cdd45c91d0b3064468ae0f03b43a319d6e1cf752fa75b3184`.
The new binary SHA256 is
`c37e2345ee5bae08c594f906bdb49dd3edbce37e46baab99a9f10f761a64af6f`,
at `/workspace/.rheon-tools/public-call-v2-target/release/deps/rheon-beaa99042d1278c3`.
Rust 1.92, locked core-only release and the frozen flags were used. The only
common-source overlays derive Clone under cfg(test); those and the private
test-module inclusion were restored byte-for-byte after preflight.

Controller/public-call E1 convergence, third-component/combined-work success,
publication, cancellation/retry and lifecycle qualification remain pending.
Original five refusals, neighbor failures, temporal-band failures, Jacobi
fixtures and numerical/Lean limitations retain their original classifications.
No fresh numerical run or production feature follows from this failed gate.
