# S1 native export proposal for parent coordination

Applies to reviewed plan `67baa10576a2120f314ec8e56bd7b7129d18d9f0`, S1-E/F/G
and independent S1-D translation. Validation owns only `tools/verify_compressible.py`
and `tools/test_compressible_verifier.py`, plus this research/validation directory.
Solver worker owns native example/core/public Rust tests. Parent relays and admits
this exact interface before native fields can receive acceptance. No shared
solver-plan source is edited here.

## Proposed producer format v1

Native example invocation: `compressible_reference --output <fresh-directory>`.
It writes `suite.json` last, with `schema_version: 1`,
`method: "llf-euler-periodic-v1"`, `gamma: 1.4`, `gas_constant: 287.0`, and `cases`.
No dataset/archive/backward-compatibility promise; this is a disposable S1 fixture.

Each case has `id`, `kind` (`uniform`, `differential`, `acoustic`, `sod`),
`shape: [nx,ny,nz]`, `spacing_m: [hx,hy,hz]`, `origin_m: [ox,oy,oz]`,
`boundary: "periodic"`, `layout: "x-fast"`, `frames` and
`initialization` (specified below). Each accepted frame has integer `generation`
(initial=0), `time_s`, `requested_dt_s`, `accepted_dt_s`, and `file` relative to
the output root. Store every accepted step including initial; dt=0 initially,
subsequently `time_s - previous_time_s = accepted_dt_s`, with no skipped steps.
Finite decimal binary64 round-trip CSV columns are exactly
`rho,mx,my,mz,E`; one row per cell, `i + nx*(j + ny*k)`; no coordinates duplicated.
Fields are conservative cell averages, in SI kg/m³, kg/(m² s), J/m³.
Actual solver reports/controls may be exported as additional frame metadata;
independent acceptance always recomputes admissibility and totals from raw U.

Required cases:

- One `uniform` case: shape `[3,2,1]`, spacing `[0.2,0.3,0.4]`,
  origin `[0.25,-0.5,1.0]`, initialization `uniform-v1`, rho=1.7,
  velocity `[0.2,-0.4,0.3]`, pressure=0.9, final t=0.01.
- One `differential` case: shape `[3,4,2]`, same spacing/origin, initialization
  `exported-initial-v1`; a smooth, finite, nonuniform all-axis native initial
  field with transverse momentum, final t=0.001 (one accepted step). Verifier
  imports the separate educational LLF **only for S1-D**, explicitly maps x-fast
  to its `(nx,ny,nz,5)` layout, and checks actual dt under both bounds.
- Three `acoustic` cases at `[8,8,8]`, `[12,12,12]`, `[20,20,20]`, unit cube,
  zero origin, initialization `linear-conservative-cell-average-v1`, t=0.08.
- Three `sod` cases at `[24,2,2]`, `[48,2,2]`, `[64,2,2]`, unit cube,
  zero origin, initialization `sod-cell-average-v1`, t=0.08.

IDs may be any unique nonempty ASCII strings; cases are selected by kind/shape.
Manifest/file readers reject duplicate cases/frames, missing/nonfinite fields,
unsafe paths and incorrect time/geometry/initialization/units/layout.

## Acoustic convention requiring explicit agreement

Let rho0=p0=1, c=sqrt(gamma), epsilon=1e-6. For modes
`(k,w,phi)=((1,1,1),1,0)` and `((1,-1,2),0.5,pi/3)`, define
`s = epsilon*w*sin(2*pi*k dot x - 2*pi*|k|*c*t + phi)`.
The linear **conservative** continuum field is
`rho=1+sum(s)`, `m=c*sum(s*k/|k|)`,
`E=1/(gamma-1)+gamma/(gamma-1)*sum(s)`.
Cell averages replace each sine by its center value times
`product(sinc(pi*k_d*h_d))`, with sinc(z)=sin(z)/z. Native initialization
and analytic expectation use this convention; exact finite-amplitude Euler is
not claimed. Its O(epsilon²) nonlinear residual is disclosed separately from
quadrature/roundoff, and can be checked by amplitude refinement without changing
the predeclared accuracy thresholds. This differs from center-sampled primitive
initialization in the older educational fixture. If solver owner needs that
initializer, parent must explicitly replan the convention before acceptance;
do not silently compare mismatched references or tune thresholds.

## Sod reference convention

Initial jump x=0.5 aligned with cell faces. Root-match exact rarefaction/shock
relations at gamma=1.4; integrate **conservative** U over each cell, splitting
quadrature intervals at fan head/tail, contact and shock. Derive reference
pressure from averaged U, not averaged pressure. Compare full cells whose
centers satisfy 0.2<x<0.8; at t=0.08 their extents remain away from physical
periodic-image waves. Disclose quadrature/root uncertainty by doubled-order
comparison. Exact conservation of isolated-fan full-domain momentum is not
the periodic reference: S1-G uses actual periodic budgets only.

## Independent origin capture

Verifier `capture` will require a clean source checkout at the externally
specified native commit, build locked release core-only example through pinned
Rust 1.92, execute the fresh binary with fresh output, retain build/run logs,
and write a verifier-owned origin manifest. It binds source commit/tree and
input hashes, exact build/run commands, toolchain, executable hash, actual
exported times/controls through artifact hashes, and verifier source identity.
The capture prints the origin-manifest SHA-256; subsequent verification requires
that **external pin**. Do not treat a self-declared manifest or analytic accuracy
as proof of origin. Mutation tests distinguish mismatched origin pin, changed
artifact and wrong numerical state; copying correct same-case fields cannot be
detected by analytic accuracy alone. Generated logs/fields remain external.

Native clean source commit and format agreement are pending. Existing elementary
pressure/viscosity suite does not satisfy S1-E/F/G.
