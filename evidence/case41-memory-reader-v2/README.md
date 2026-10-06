# Case41 linked-memory reader diagnosis, v2

Tooling only. This packet does not modify production Rust, rebuild the archived
binary, enable the finite-difference test, evaluate a candidate equation, create
an accepted owner, or change any numerical gate. The old failed reader and its
scalar/layout evidence remain intact in `case41-six-fd-columns-v1`.

The existing binary is compiled source c238111d8c8d159c157e489294a25998aaf36516,
tree c87e9c658cad27df708a34aa63bccf9868095598; its SHA-256 is
48f58e62577eef569e942d3e8e7139701613091076a736c015bee5256691ba72.
The reader checks actual objdump bytes against the original lossless archive,
ELF-sized function identity, actual installed libc, and closed scalar Result paths.
No independent acceptance of a complete case41 memory preflight has occurred.

The old reader required a standalone `Work::equation` frame on each route. Both
observers have absorbed that body and call `point` and `partition` directly. The
standalone equation symbols elsewhere in the ELF serve other routes. Removing
this mistaken requirement is safe only while retaining the entire observer
frames and checking both actual outgoing numerical calls. A one-sided absence
remains an error. This is an assembly/source diagnosis, not a compiler proof.

The new traversal retains all directly and GOT-reached crate callees, including
coefficient, cross, linear, geometry, heap-budget and scalar helpers. Conservative
register-origin dataflow resolves singleton GOT function pointers on all incoming
paths; unknown origins never establish a call target. Full fixed frames and every
external transfer are retained in the result. This is a targeted reader, not a
formal verifier of arbitrary x86 code or a complete linked process call graph.

A known crate-frame/storage subtotal is expected to be 66,816 bytes:
62,096 workspace + 96 entire conservative outer-frame remainder + 4,096 maximum
positive crate/kernel frame route + 528 explicit duplicate buffer charge.
The remaining 768 bytes are unallocated headroom, **not proof of a complete cap**.
Each source-operation difference is clamped independently. Unmatched crate
callees receive their full fixed frame; no shrinking frame or nominal baseline
slack is credited. Shared recursive geometry bodies are reported explicitly;
this packet does not establish a finite absolute stack bound for that recursion.

`BLOCKED_EXTERNAL_TRANSFER_ACCOUNTING` is deliberate. Dynamic libc operations
and linked non-crate callees are listed rather than omitted or assigned zero.
Some are common formatting/panic/allocator code; others are numerical array and
sorting helpers. Matching a symbol alone does not prove equal simultaneous
memory, arguments, heap behavior or callback costs. A complete numerical
certificate still needs those costs closed, or a justified common-operation
comparison. The subtotal must not be used to execute the six FD equations.

`preflight.py` invokes exactly three existing scalar/layout tests with the FD
allow variable absent, then the rational scalar/affine oracles and static reader
in normal and optimized Python. It has no FD runner. The preparation's existing
Rust formatting/release compilation/Clippy evidence is reused, not rerun.
The two first extended-traversal failures are preserved with their source and
stderr: a recursive geometry edge invalidated a naive acyclic absolute-stack
calculation. The current reader reports that recursion and refuses an absolute
stack certificate. All original Jacobi fixtures and numerical/proof limitations
remain unchanged. Independent acceptance remains required before any FD work.
