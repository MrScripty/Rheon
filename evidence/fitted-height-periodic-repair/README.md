# Corrected periodic seam evidence

[Geometry/rank erratum and limits](../../docs/research-book/implementation/fitted-height-periodic-seam-repair.md).
Correct neighboring-centroid intersections across the periodic seam and their
true derivatives. Research `2db90e8c` and native predecessor `c995671c` remain
frozen; their rank-33 mesh is coherent periodic P1 algebra, but does not qualify
the intended periodic Powell–Sabin seam.

`diagnose_seam.py OUTPUT` actually rejects the frozen midpoint seam, checks the
correct location/motion and exact rank32, and shows small nonzero seam
perturbations restore rank33. It makes no GCL/time claim for those perturbations.
`reference.py OUTPUT` runs all corrected rational geometry, full-vector transport,
pressure, strain, traction and energy gates.
`python -m unittest test_reference.py` (also with `-O`) checks seam geometry,
singularity, original arithmetic/operator contracts and finite energy counting.
`qualify_rust.py OUTPUT` and `qualify_native.py OUTPUT` run actual feature-matrix
and release tests, builds and independent native validation.
`verify_native.py JSON --negative-self-test` also rejects restored midpoint seam
coordinates. `make_figure.py` draws corrected native instantaneous arrays only.
`verify.py --negative-self-test --evidence-commit COMMIT` binds source, all nested
receipts and complete historical bytes; only its root receipt is self-excluded.

Conditional historical Lean algebra is preserved, with no new proof claim.
Pressure rank is finite numerical/exact-reference evidence, not uniform stability.
No pointwise traction accuracy, physical time integrator, advancing solver or
native evolving-liquid render is claimed; the old refusal is unchanged.
