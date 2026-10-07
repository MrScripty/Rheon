# Forward clarification of the stable-sort recursion envelope

The first common-cost note bounds the regular stable-quicksort recursion at
13 frames. A limit-zero regular frame can call eager drift, which itself calls
one additional quicksort frame on at most 32 elements. That eager frame returns
from its small-sort branch without further recursion. Thus the full simultaneous
quicksort count, including fallback, is at most **14**, with at most two drift
frames. The static ledger's factor-sixteen envelope already covers this count;
it sums every linked sort fixed frame sixteen times and is unchanged.

The 761,088-byte envelope is deliberately loose common-library fixed-frame
accounting, includes unused linked instantiations, and excludes unresolved
external transfers. It is not an additional-memory charge or complete bound.
No complete memory certificate was accepted and no equation executed using
the earlier wording. The original frozen source note remains in Git history.
