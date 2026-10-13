# Issues and dispositions

- Arbitrary visual mesh cannot establish valid solid volume. Explicit surface
  barriers are distinct from validated convex watertight solids.
- Existing grid carrier/pressure solver admits no arbitrary obstacle topology;
  feature is restricted to the native particle material path.
- Particle-center collision does not resolve particle radius or kernel wall
  support; no calibrated traction or two-way force claim.
- Display tools were not initially present; inspect/install usable local native
  rendering support before claiming screenshot evidence.
- Independent review found path-length contact retreat could leave a grazing
  solid hit within the exclusion tolerance. Fixed to use normal clearance;
  regression covers oblique entry and tangential continuation.
- Independent review found endpoints entering the numerical contact band before
  the exact plane could reject a later step. Sweep now intercepts the contact
  band consistently with solid admission; regression covers near-face endpoints.
- FIFO/special-file import could block the UI before byte limits applied. Unix
  nonblocking opens plus preflight and held-handle regular-file checks now reject
  these inputs, including buffer resources. Regular-file I/O latency is not bounded.
