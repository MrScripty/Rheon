# CV2 findings

- CS-I1: square fullM cond6.533e11 and minMii1.7014e-14 retained. Not equivalent
  to A/KKT condition. Independent R1/R2 solution errors decide this gate.
- CS-I2: scaled/backward-stable solve cannot repair rounded-operator/load error.
  Both exported-target and exact-integral-target accuracy required; no relabeling.
- CS-I3: explicit Schur Gram squares weighted-divergence conditioning. Before
  coding/data select HouseholderQR; no afterdata fallback to QR/highprecision.
- CS-I4: native lowerfactor is only an approximate preconditioner when actualfull
  A has rounding asymmetry; exact originalfull residual/refinement/oracle retained.
- CS-I5: pressure/cut stability, oblique wall lifting, transport and PDE accuracy
  remain outside this static finite-corpus solver; no essential syntheticinputmissing.

- CS-I6: observed baseline fails unchangedvelocitycriterion all6squaresteps, max
  2.55e-9 despite backwardresidual9.93e-12 passingbudget. Retainednegativeevidence.
  Selectedmethodpasses bothR1/R2all24; no fallback/basis/support/criterion changes.
- CS-I7: selectedR1errors<=1.95e-16; R2v<=1.862e-12,p<=3.355e-13. Assembly/load/
  RHSgapdominatesremainingerror, belowfixed1e-10 here. Futurefinergrids/loads need
  newconditioning/approximation evidence; currentresultisnotuniversalerrorenvelope.
