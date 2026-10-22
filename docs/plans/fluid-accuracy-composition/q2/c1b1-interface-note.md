# C1-B1 interface coordination

Parent relay: PR128, tested sourcef161d1a and documentation43a084a, stacked on
PR125. Read-only inspection of PR128 file list and actual projection.rs atf161d1a
confirms no shared write/API conflict with Q1/PR126 or this Q2/PR127 slice.

C1-B1 adds composition/projection.rs, projection_tests.rs, its module declaration
and c1b1reports. Consumed PressureOperator/ProviderWorkspace/GridGeometry and
declared SessionIdentity APIs are unchanged. Q1's only existing numerical write is
src/liquid_volume.rs; Q2 adds private reference examples/tests/docs only. Neither
changes C1-B1's pressure/velocity/representation contracts or pressure formulas.
Native Y-face x-fastest indexing in Q2 matches GridGeometry and the C1 metadata.

C1-B1 sealed stationary pressure projection does not gain periodic Q1 transport,
free-surface/obstacle/moving-wall support or future coupled Stokes applicability.
Its supplied immutable array witness is not candidate runtime execution, Session
adoption or custody. No combined-branch build, C1-B/R trajectory or workflow claim
is made by this interface inspection. Lead owns dependency/CI/integration and
canonical ledger reconciliation; no C1/G1 files were edited or branches merged.
