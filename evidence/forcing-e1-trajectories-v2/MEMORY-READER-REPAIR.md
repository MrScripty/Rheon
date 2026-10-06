The frozen native binary and numerical source are unchanged. The first metadata
auditor rejected a constructor name that LLVM had inlined into the candidate
wrapper. Its empty outputs and exception logs remain preserved. The additive
closed reader follows the actual reference and candidate constructor graphs,
including a chart solve at every possible constructor caller. Positive stack
deltas are charged; the candidate's 32-byte constructor-shell shrink earns no
credit. Constructor added peak is 3,392 bytes, below the public-call 3,424-byte
peak. Together with the 62,096-byte workspace and 8-byte descriptor, the aligned
additional bound is 65,536 bytes, leaving 2,048 of the 67,584-byte allowance.

The first closed-reader metadata used integer graph keys. JSON serialization
made them strings, so direct replay comparison failed despite equal numerical
bounds. That reader and both failed qualification attempts are preserved under
reader-v1 names. The corrected additive reader canonicalizes its returned
metadata through JSON. Normal and optimized readers and fresh qualifications
now agree exactly. These repairs change evidence readers only, before any E1
trajectory execution; they do not alter the binary or acceptance criteria.
