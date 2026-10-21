# PR59 analyzer CLI exit-status delta review

2026-10-21. Read-only static review; no analyzer/data execution, numerical/harness run, repository test, build, cloud action or binary mutation. AST parsing and in-memory syntax compilation passed. The initial review remains unchanged.

**Decision: approved; no blocking defect found in the exact two-line delta.**

After exclusive report creation and decision printing, `main()` returns zero only when `performance_admission == 'meets-frozen-rule'`, otherwise 2. The module entry now calls `sys.exit(main())`. Thus adverse/inconclusive cohorts retain their written `blocked` report and produce exit status 2; only both per-case classifications meeting the frozen rule can produce status zero. Input/serialization/filesystem exceptions remain nonzero failures rather than admission. This closes the earlier reporting-CLI exit-status limitation without changing statistics, thresholds, cohort shape, data handling, pooling, waiver behavior or overwrite protection.

Removing the new return line and reverting `sys.exit(main())` to `main()` in memory reproduces the exact initially reviewed script SHA256, proving no other analyzer bytes changed. No file was reverted or modified for this check.

- Previous analyzer SHA256: `676c7166e693c99d75c3a2add120c9f2ef0022e9bebadae18bba72678e69413d`.
- New reviewed analyzer SHA256: `e42977351b65c2886f2b5a59429e1e1418651662c9c21c43a0c7206134e2b1f9`.
- Preserved initial report SHA256: `02437e088dacb90b28069d3705de38c35aa9e874da6aeffb04f8dba8f5493a86`.

All other scope/provenance/assumption limitations in `/tmp/pr59-controls-analyzer-review.md` remain applicable, except its statement that a blocked valid cohort exits zero is superseded by this delta review. Process success now reflects the summary's strict statistical rule; it still does not certify external build/source/manifest custody or final lead admission. No timing data was processed, no cohort extended, and the historical +2.20% evidence remains unaffected. No denied or pending operation arose.
