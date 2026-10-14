# Findings and dispositions

| Finding | Evidence | Severity/owner | Disposition |
| --- | --- | --- | --- |
| Saved checkout stale | work at 6ba41c7; current main 8baf459 | process / validation | refreshed refs; preserved original; isolated current-base branch |
| Adjacent-cell pressure loads can be biased despite gauge/work checks | docs/broader-physics-milestone.md:39–43 | physical label blocker / solver owner via parent | no production body-load acceptance; require affine pressure force convergence |
| Elementary mechanisms do not qualify full turbulent/compressible/moving exterior flow | current bounded API documentation | acceptance / parent and solver owners | staged gates in report |
| NASA TMR legacy URL redirects | primary NASA landing page 2026-10-14 | source provenance / validation | use current NASA-linked TMR URL |
| Exact third-party geometry/data rights not established | case-by-case primary-source checks | data ingestion blocker / dataset owner | reference links only; no copied assets |
| Native S1 fields not yet available | reviewed first-slice and this worker's exporter proposal | pending acceptance dependency / parent + solver worker | tools/tested oracles ready; wait for format admission and clean native commit before actual capture; not denied |
| Required installed NumPy wheel includes copyleft runtime terms | installed 2.3.5 LICENSE.txt hash in S1 proposal | runtime-distribution policy / parent | use explicitly pinned existing test runtime; no GPL-free or packaged-runtime promise; no binaries redistributed |
