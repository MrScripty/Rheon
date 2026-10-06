# Guard-qualified diagnosis; numerical preflight remains blocked

Guard-tested tooling source **87fcfa4e26c85a087c700d9de95ec5fc269f5e24**,
tree **ce4d7ec44018347b9c3c4f63789bd41dc17879d5**. The first source freeze
is ee9f5332512ca2230ecf16d705e6dd64b3597ee1; the next forward commit binds
the original blocked receipt explicitly. Neither commit rewrites old evidence.
Compiled numerical source and ELF remain the identities stated in README.

Exactly three existing native tests passed: scalar conformance (23 groups and
36 independent exact rational probes), actual type layout, and six affine
probes. No FD-enable variable was present. Both normal and optimized Python
produce identical reader and verification results. Four controls reject a
one-sided missing point, a missing absorbed-body edge, and an undersized outer
frame; an unknown indirect call stays present and blocks acceptance. Actual
`nm`, `readelf` and `objdump` output matches the existing compiled archives.

| Operation | Candidate frame | Reference frame | Charged positive difference |
| --- | ---: | ---: | ---: |
| Capture shell | 9,680 | 9,216 | 464 |
| Observer with absorbed equation | 183,472 | 184,064 | 0 |
| Partition | 74,128 | 74,080 | 48 |
| Point | 27,936 | 26,048 | 1,888 |
| Coefficients / cross / linear | 384 / 8 / 2,160 | 384 / 8 / 2,160 | 0 |
| Closed scalar Result-path peak | 1,696 | New arithmetic | 1,696 |

The maximum known positive crate route is 4,096. All other reached crate
callees remain in the graph and frame ledger; unmatched callees get their full
frame, with no credit from a smaller observer or unused baseline reservation.
The numerical `linear` helper appears in the native constraint-chart fallback;
its presence is not evidence that a new Newton correction was performed.

The outer test frame is 62,192, including the entire 62,096 workspace. Its
remaining **96** bytes replace the old reader's assumed 8-byte descriptor
charge; the reader does not discard the remaining outer slots, pushes or return
address. Adding three explicit 176-byte buffers yields the known subtotal
**66,816**. This is not an accepted bound. It leaves **768** unallocated bytes
under the unchanged 67,584 cap, pending complete external-callee accounting.

There are 46 candidate and 52 reference reached crate functions. Conservative
CFG register-origin analysis resolves all indirect calls on those graphs, but
319 candidate transfers go to linked non-crate functions and 60 to dynamically
bound functions (reference: 576 and 145). The complete ledger retains them.
Many targets occur on both sides; that fact alone proves neither equal
simultaneous stack/heap use nor equal callback arguments. Four candidate-only
first-boundary targets include two array-drain instantiations, one sort
instantiation and a panic instantiation. Readable array/sort assembly is supplied.
Recursion in the shared geometry split function is also retained explicitly;
this packet does not prove a finite absolute stack bound for it.

**Remaining blocker:** close the non-crate numerical/library paths and justify
any common-operation comparison before asserting a complete cap. No complete
memory certificate or independent preflight acceptance exists. The six FD
columns remain forbidden. Formatting/panic/allocator conventions, heap costs,
callback costs and the common recursive geometry path cannot be silently
relabelled zero to obtain a pass. This is a read-only tooling diagnosis,
not a cap-overrun finding or a numerical refusal finding.

No production change, Rust rebuild, candidate equation, owner construction,
owner advance, finite-difference column, Newton correction, public publication,
new replay/render, or Lean claim occurred here. Existing research equations,
original Jacobi fixtures, thresholds and proof/numerical limitations remain
unchanged. The earlier failed reader, two new failed absolute-graph attempts,
case47 evidence and all frozen predecessor bytes are preserved.
