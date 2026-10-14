# Findings and dispositions

| Finding | Evidence | Severity/owner | Disposition |
| --- | --- | --- | --- |
| Saved checkout stale | work at 6ba41c7; current main 8baf459 | process / validation | refreshed refs; preserved original; isolated current-base branch |
| Adjacent-cell pressure loads can be biased despite gauge/work checks | docs/broader-physics-milestone.md:39–43 | physical label blocker / solver owner via parent | no production body-load acceptance; require affine pressure force convergence |
| Elementary mechanisms do not qualify full turbulent/compressible/moving exterior flow | current bounded API documentation | acceptance / parent and solver owners | staged gates in report |
| NASA TMR legacy URL redirects | primary NASA landing page 2026-10-14 | source provenance / validation | use current NASA-linked TMR URL |
| Exact third-party geometry/data rights not established | case-by-case primary-source checks | data ingestion blocker / dataset owner | reference links only; no copied assets |
| Native S1 fields not yet available | reviewed first-slice and admitted exporter proposal | pending acceptance dependency / parent + solver worker | parent admitted format at 4621d69; await clean native commit before actual capture; not denied |
| Required installed NumPy wheel includes copyleft runtime terms | installed 2.3.5 LICENSE.txt hash in S1 proposal | runtime-distribution policy / parent | parent permits separately installed validation tool; no runtime distribution proposed/authorized, no GPL-free promise, Apache-2.0 source unchanged |
| R1: sandbox markers rejected valid capture output | review at 4621d69 | P1 / validation | actual-checkout detection repaired; independent closure confirmed |
| R2: ignored compilation input acquired source identity | review clean-commit Cargo probe | P1 / validation | tracked-only export and build-input admission repaired; independent closure confirmed |
| R3: first-mode speed gate does not separately bound mode two | review synthetic 20%-slow second mode | scope / parent | explicit report/test; no threshold change; replan for broader guarantee |
| R4: Cargo build Rust flags bypassed initial guard | independent repair follow-up | P1 / validation | guard/negative tests passed; independent focused closure confirmed |
