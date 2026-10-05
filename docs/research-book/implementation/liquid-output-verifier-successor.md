# Successor fresh-output transport verifier

The frozen composition and coupled packets remain byte-preserved. The new
tools/verify_liquid_transport.py strengthens checks on fresh output without
rebinding the historical evidence or changing the accepted solver source.

Explicit finiteness precedes pressure maxima. Native ledger balance and the
64-epsilon reduction budget are independently recomputed. The initial amount,
per-step volume carry-forward, boundary/source conditions and cumulative ledger
are checked separately. Negative fixtures cover NaN pressure, infinite velocity,
999-valued before/inward/outward fields, altered balance/budget and broken
carry-forward. All checks remain active under Python -O.

The committed 240 ledgers still pass. The demonstration is X-directed slab
translation stored on a 3D grid; it does not qualify multidirectional transport
or free-surface accuracy. Conservation is within the stated rounding convention,
including the tiny recorded boundary outflow tail. The composition verifier uses
assert and is not claimed to qualify standalone optimized-Python execution.
The complete Git packet binding supplement uses explicit checks.
