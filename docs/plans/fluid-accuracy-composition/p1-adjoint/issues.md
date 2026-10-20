# Unresolved selection blockers

1. Gamma adjoint map: genuine perturbed sampling E0 and wall-volume/traction
   inconsistency, despite exact work. Reject for production; no new trajectories.
2. Exact-partition reference: cell-mean versus site hydro inconsistency, endpoint
   wall response and an interior alternating-pressure operator defect. Reject.
3. Both: no qualified pressure/geometry acoustic Hessian or timestep bound. An
   h/c heuristic and semidiscrete work identity do not supply one.
4. Both: native 3D wall/viscosity, mesh geometry and full fixed P-W campaign remain
   pending method selection. Planar static results do not discharge those gates.

Next candidate to derive: shared positive volume partition with explicit boundary
representation, complete derivatives and source-integrated volume/first-moment/
gravity quadrature. Choose site/centroid meaning explicitly and differentiate
the same total potential. It must
first survive E0, linear/cell-mean hydro, physical wall-load, translation and
acoustic spectrum gates. No implementation admission for that candidate is made.
Parent owns canonical plan changes, stacking/integration and eventual merges.
