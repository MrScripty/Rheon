# Outcome: one changed chart candidate crosses the fixed residual threshold

The specified comparison is complete and numerical work has stopped. No search,
Newton correction loop, accepted owner, third-component solve, publication,
tolerance change or production adoption occurred. All original five refusals,
the 220 refused neighbors and strict temporal-band failures remain unchanged.
Production `src/lib.rs` was restored after the archived test-only inclusion.

| Fixed comparison | 16-point rate norm | 32-point rate norm | Interpretation |
| --- | ---: | ---: | --- |
| B0 native original | 1.1895444217920825e-13 | 1.1895028437230134e-13 | Above unchanged Newton target 1e-13; reproduces frozen fields. |
| A0 exact arithmetic on B0 binary fields | 1.1895842873679072e-13 | Not a new native equation | Exact squared norm remains above the exact binary threshold squared. No hidden original pass. |
| E1 changed chart solution, full native downstream recomputation | 9.978631585490488e-14 | 9.979078763725605e-14 | Both measured native norms below the unchanged target on a different candidate. |

B0 independently matches **1,935** captured floating values by binary bits,
including signed zero, plus exact face topology. The actual Rust test also
checks its full coarse arrays, norm and transfers. The B0-only stage completes
before the normal/optimized A0 reader; E1's command binds that completed A0
artifact and the scalar/memory preflight before execution. E1 repeats B0 as
an input guard and then evaluates only the same pinned terminal unknowns.

E1 changes 11 of 22 endpoint coefficient bits and 11 nodal velocity components,
27 force components, 19 positive and 19 negative face transfers, and all 22
stable/direct rate components. q, eta, R, old/new mass, end D and end B remain
bitwise identical. The terminal alpha/pressure inputs and accepted mass/velocity
are unchanged. Native qB/cap/geometry policy and downstream checked arithmetic
are retained; forces and transfers are recomputed rather than borrowed from B0.
The additional exact diagnostic on **E1's changed fields** has norm
9.978522789647268e-14; it is distinct from A0 and is not the Newton norm.

The isolated native planar qualifier passes its unchanged gates: fine stable/
direct norms 9.979078763725605e-14 / 9.812543087080354e-14, constraint maximum
2.3862205614233467e-15, local GCL maximum 1.0197456121774123e-16, quadrature
difference 2.2234614865425384e-20, residual work -2.9414482819713763e-17 and
ledger error 7.523007824333794e-17 against allowance 4.5502795188855507e-14.
Signed GCL work and physical viscosity, BE and donor mixing remain separate.
These are complete 22-row planar equation/physical diagnostics, not full
three-component/public-step qualification or a reclassification of B0.

This establishes a concrete **finite-arithmetic candidate sensitivity**: a
106-bit ephemeral increment solve, followed by stored-endpoint recomputation,
can produce a different candidate below the fixed threshold. It does not show
that the original controller was defective under its native arithmetic contract,
that E1's Newton iteration will converge, that any public call will advance, or
that subsequent/other/finer/reversed trajectories pass. The small margin to the
threshold is an observation, not a robustness certificate. No floor is claimed.

## Source and policy freezes

First prototype was frozen/pushed before compilation at
`5270e91c1ee1e14b936d66d750ce8a631c078cfb`, tree
`8193a9a5ba7ecb1cb41b1296a4ad5bb8334b0b01`. Its immutable
`prototype-policy.json` binds source and inherited policy. A later additive
execution wrapper at `78b7365513ccf317e554d36f968da36b55f04065`, tree
`35770b5a8cbab53a24232298961eac4eeca5c801`, adds a B0-only stopping point so A0
executes before E1; it changes no scalar/equation/geometry/input arithmetic.
Both source milestones and all earlier artifacts remain unchanged.

The staged policy SHA256 is
`7b4328c3087b4cc8e6c1adbe6bb65e589bdb014c0b12e0ceab11e77cad98c2ec`.
The exact staged native test binary SHA256 is
`6ce8f08803c74654433093559e8ca0ce2c02586aff1999ffaf7bf40a59b744d0`.
Rust/Cargo 1.92.0, core-only release, locked dependencies and dedicated targets
were used. Actual Rust formatting checks and the targeted preflight/comparison
unit tests pass. No unrelated full matrix is rerun or claimed.

## Actual additional workspace qualification

The compiled Scalar is 32 bytes and ChartWorkspace is **62,096 bytes**, including
all 1,934 scalar slots, fixed accepted-coefficient extraction and descriptors.
The Work borrow adds 8 bytes. Linked x86_64 machine-code frame/call analysis
gives a conservative additional kernel stack bound of **1,920 bytes**, including
128 bytes of red-zone allowance and return addresses per function. The compiled
simultaneous additional bound is **64,024 bytes**, below the frozen 65,536 limit.
The 2,048-byte kernel ceiling reserves **64,152 bytes** in total. Native baseline
forced nominal reservation remains **474,768 bytes** (including its step/third/
force reservations); combined baseline plus additional reservation is **538,920**.

The successful numerical kernel allocates no heap, recurses nowhere and uses no
dynamic stack size. The actual default libc memset branch graph is a leaf with
no stack use; it is bound to the installed library/CPU variant. Private normalized
scalar and fixed-index invariants exclude generated panic-bounds paths. Ordinary
range/division errors return the documented failure. This is a bound for this
compiled research kernel and environment, not a portable production allocation
or restart proof. Host logging, exact Fraction oracles and archived evidence
are separate research infrastructure, not engine workspace or hidden state.

The first compiled scalar suite passes 23 assertion groups and the independent
Fraction oracle checks all 36 emitted operation probes exactly, including
rounding, signs/zeros and range/underflow refusals. Pivot/tie and endpoint
conversion checks are part of the actual Rust suite. The staged source repeats
that conformance before B0/E1. This finite suite is not a formal proof of every
possible 106-bit operation.

## Retained qualification issues

The stable compiler/link path did not yield an ELF `.stack_sizes` section, even
with retention requested. Both completed preflight logs and section inventories
are preserved. The actual linked disassembly instead provides the stack proof;
full disassembly is losslessly archived with fixed gzip timestamps and raw SHA256.

The first memory reader inspected an adjacent fortified libc wrapper and rejected
the branch-span check. Its source, failed output and traceback remain unchanged
as `check_memory-first.py` and `memory-preflight-first*`. The corrected audit
follows only code reachable from the actual default memset entry and passes.
No scalar implementation, candidate or arithmetic policy was changed to address
this reader issue. No genuine scalar conformance failure occurred.

The final source/evidence receipt and post-Git verification bind completed logs,
native B0/E1 captures, normal/optimized readers, frozen source and full preservation.
The independent arithmetic/proof/physics limitations of the original contract
and supplement remain in force. This is the requested stopping point; future
controller/lifecycle/third/replay/adoption work requires a separate coordinated
decision and evidence.
