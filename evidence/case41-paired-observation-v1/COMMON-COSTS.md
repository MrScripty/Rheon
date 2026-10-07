# Actual argument and lifetime obligations

The comparison is between these two paired wrappers, not the old reference
observer with extra point captures. The full fixed frames, absorbed equation
body, return storage, all crate callees, outer calls, linked library calls,
dynamic calls and panic branches are retained in the ledger. Positive source
operation differences are charged separately; shrinking frames and baseline
slack supply no credit. Unknown external costs have no numerical value and
cannot be counted as zero.

## Geometry storage and recursion

Both wrappers construct one FittedHeightWorkspace using exactly the frozen
case41 cap, bottom [0,0.5,1], width 1, density 3, viscosity 0.05 and default
settings. The constructor's 14 Budget::vector arguments at columns=2 are
enumerated by the scalar-only layout test, with actual native element sizes.
One geometry lives from construction until the wrapper returns. Reassembly
clears/resizes its existing vectors; there is no second geometry observation
copy. The allocator, constructor/drop frames and failed allocation branches
are retained, rather than assumed free.

Topology capacities are 19 geometric nodes, 4 macro triangles, 9 macro edges,
24 triangles, 16 embeddings, 72 pieces, 72 faces, 288 pressure terms, 22 pressure
columns, 528 rank scratch entries, 16 masses, 19 dual-motion entries, 16
diagnostics and 24 triangle scratch entries. The actual requested payload sum
comes from the native layout receipt. Budget charges Vec capacities, not lengths.
The pinned RawVec `grow_exact`/`set_ptr_and_cap` and Global allocator sources
are archived. They describe requested layouts and payload capacities, not a
bound on libc allocator internals or its simultaneous stack. Those costs stay
unknown in the complete preflight.

At columns=2, `split_position` recurses only for right-seam endpoints [2,5],
and its sole recursive argument is left seam [0,3]. The predicates are disjoint.
The left seam executes the nonrecursive intersection branch. Thus at most two
split_position frames coexist, on both construction and reassembly routes.
The static crate bound includes the second frame and its nonrecursive children.
No unbounded geometry recursion is asserted and no recursive frame is dropped.
The topology still has to be valid on numerical success; geometry errors retain
their original refusal paths.

## Sorting and callbacks

Partition receives the original 66-element f64 boundary buffer. Count starts at
2, is checked before insertion at 66, and sorting sees 2..66 elements. Both
equations use `f64::total_cmp`; this callback is an integer bit comparison with
no user callback, geometry mutation, allocation or numerical reevaluation.
The paired ELF reaches the same actual insertion-sort and driftsort entrypoints
on both sides. Each entry frame and its outgoing transfers remain in the ledger.

Pinned Rust 1.92 stable sort reserves AlignedStorage<T,4096>. For f64 this is
512 elements. Required scratch is max(n-n/2,min(n,1,000,000),48), at most 66
here. Therefore its heap-buffer branch is not taken for these actual lengths.
The 4,096-byte stack scratch is present in the full 4,160-byte driftsort_main
frame, not omitted because the sort is common. Drift has 66 run slots and 66
depth bytes. Stable quicksort's limit starts at 2*floor(log2(n|1)) <=12 and
decreases before a recursive call: at most 13 quicksort frames coexist. Its
limit-zero fallback calls drift in eager mode, which calls quicksort on <=32
elements and immediately small-sorts. At most two drift frames coexist.
An intentionally loose envelope sums every linked sort fixed frame sixteen
times, including unused instantiations; unrecognized frames or external
descendants leave a complete bound unknown. This envelope is not added to the
additional subtotal as though every library path were already paired.

Geometry sorts exactly nine macro edges using a fixed ends-key callback. Rust
1.92's <=20 insertion-sort branch applies, so its linked ipnsort/quicksort
branches do not execute for that sort. Both insertion-sort frames and even the
unreachable outgoing ipnsort edges are retained for independent inspection.

The first-boundary array-drain helpers map fixed arrays (accepted fixture
conversion and triangle positions/motions), not growing containers. Their
callbacks are statically instantiated fixed index/lookups. The drain owns a
ManuallyDrop array plus a slice iterator, calls its callback once, and allocates
no heap. The compiler's full helper frames include inlined callbacks: 8 bytes
for fixture conversion, 64/16 for point maps, and 112/16 for inspect maps.
Candidate point maps have separate addresses from reference but equal fixed
frames and the same bounded shapes. Their panic/unwrap transfers remain
explicit unknown-cost branches until topology/index invariant exclusions and
all remaining descendants are independently accepted. No callback is silently
assigned a zero stack cost.

## Serialization, outer calls and residual unknowns

Borrowed serialization occurs after the numerical boundary returns. It uses
the same field types, fixed array dimensions and Debug formatting on both
sides. Baseline rates alone persist between evaluations. At most fourteen
records are emitted: seven equation records, six column/column-unavailable
records, one completion. A baseline refusal emits only refusal and completion.
Each equation record has fewer than 10,000 numerical scalar tokens, each
finite binary64 Debug token fits 32 ASCII bytes, and fixed keys/separators fit
64 KiB; a conservative per-equation record envelope is 512 KiB. No equation
record is buffered or cloned by the wrapper. A later runner must bound journal
storage separately; no such runner or output allocation measurement exists yet.

The candidate outer test contains the entire 62,096-byte ChartWorkspace. All
remaining outer slots, pushes and return address are charged, and both outer
transfer lists are archived. The three fixed buffers receive an explicit
528-byte duplicate charge even though their storage is also included in the
full paired capture frames.

Dynamic memory/string/I/O/libc/allocator paths and linked panic/error descendants
still need a complete cost or an argument/lifetime-based common-operation proof.
Matching a name or sharing a code address alone does not establish equal live
memory. The known crate subtotal is provisional and cannot authorize equations.
