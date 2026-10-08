# Fixed arguments, simultaneous lifetimes and remaining linked costs

This is targeted preflight for one frozen case41 wrapper. No generic hardening
or whole-program memory certificate is claimed. A common symbol is insufficient
to establish common simultaneous memory. The values below come from the actual
arguments, compiled layouts, and frozen construction/reassembly source.

Both setup paths construct one two-column workspace from three cap and bottom
entries, density3, width1, viscosity0.05. Its 14 requested vector capacities are
19,4,9,24,16,72,72,288,22,528,16,19,16,24. Actual element layouts are
24,24,40,80,64,56,24,24,8,8,8,32,96,16 bytes. Requested payload is23,584,
not zero. Geometry stays live in Work throughout all observations, numerical
calls and serialization; one immutable old velocity/mass fixture stays live
alongside it. Candidate additionally borrows the62,096-byte chart in the outer
frame. Reference is a memory specimen; its unused chart is never credited as
baseline slack. Three176-byte buffers remain explicitly charged even though
their slots are also included in conservative capture frames.

For c=2, construction/reassembly generates 6 cap/bottom nodes,4 macro-centres,
9 edge nodes,24 triangles and16 periodic nodes. Each triangle contributes at
most3 face pieces (72) and12 pressure terms (288); pressure columns22 and
rank scratch24*22=528 are fixed. Reassembly clears and reuses capacities.
Constructor Budget checks actual vector capacity bytes against the default
64MiB limit, including excess capacity. This proves a finite source limit,
not the actual allocator's overhead, transient calls or a cancellation of
candidate/reference allocator costs. Those linked costs stay unclosed.

The only geometry recursion is split_position's right seam[2,5] calling
left seam[0,3]. For c=2 these differ. Left cannot re-enter the right branch:
at most2 active split frames, each320 actual bytes (640 absolute bytes before
callee costs). Both paths retain the same fixed topology; changing coordinates
or a geometry refusal cannot increase this depth. Recursion is never free.

The geometry edge sort receives9 MacroEdge elements. Rust1.92's ordinary
sort_unstable entry uses insertion sort up to20; ipnsort's large-input branch
is source-infeasible for this construction. Its first-boundary target remains
in the assembly ledger until argument/CFG exclusion is independently qualified.
The key callback reads two usize edge endpoints; it neither allocates nor
recurses. No pressure or topology arrays change size in the FD roster.

Partition's stable f64 sort receives count2..66 after explicit overflow guards.
Its total_cmp callback operates on float bits, without allocation or recursion.
Rust1.92 requests max(ceil(n/2),min(n,1,000,000),48) scratch elements: at most66
for these arguments, within the512-f64/4096-byte stack buffer. Thus this
specific successful sort path needs no heap scratch. Its full4096-byte buffer
and descendant frames still cost memory. We do not credit that cost as common
merely because the same sort symbol appears on both graphs: candidate/reference
sign-root counts and branch choices have not been established for all seven
future probes. Bound/comparison closure remains necessary. The official
installed sort source and actual specialized assembly are archived by the
supplement reader; the reader makes no whole-stdlib claim.

Candidate-specific array drains implement two three-index triangle maps:
positions from19 nodes and motion derivatives from19 motion entries. Their
closures are inlined in the actual helpers. Valid-path fixed frames are64 and16
bytes, with no heap and no outgoing callback; source fixed-topology indices
are bounded below19. Bounds-panic transfers remain explicit in the linked
ledger; no panic/format/allocator convention is silently assigned zero.

Cancellation closures in setup, point, inspect, partition and equation capture
no state and always return false. Their values and bounded loop inputs are
fixed, but every outlined helper or indirect transfer still needs a compiled
closure or common-lifetime proof. Current preflight withholds acceptance for
all unresolved linked/dynamic paths and complete outer/setup/error lifetimes.

Serialization occurs only after the numerical boundary returns and borrows
the one equation Result slot. It retains the same original required FD fields.
The retained slot, chart, geometry and fixed buffers stay counted while it
runs. Formatting/error-string allocations and process output costs are not
numeric kernel authority; they remain an explicitly unclosed observation
scope rather than a zero-byte global bound. At most7 equation records,6 column
records and1 completion record (plus honest refusals) can be emitted. No large
native observation collection, complete equation copy or new point call is
introduced. Immutable fixtures and external output storage are outside the
additional numerical-buffer cap, not excluded from the explanation.
