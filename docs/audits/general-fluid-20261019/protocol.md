# Independent general-fluid audit: pre-evaluation protocol

Baseline: remote main bc2f472787060d9a69cb619ed2c2fea10fd754b6 fetched 2026-10-19.
Audit branch audit/general-fluid-physics-20261019. No production repairs.

Water at 293.15 K, near 0.1 MPa: rho=998.2 kg/m³, dynamic mu=0.001002 Pa s,
kinematic nu=mu/rho=1.0038068523342015e-6 m²/s, sigma=0.0728 N/m,
g=9.80665 m/s². Rounded constants deliberately match the preset; reference
correlations and their uncertainties will be recorded in the report.

Acceptance criteria fixed before running audit numerical probes:

1. Existing native CPU contracts: all selected solver/mesh/physics contracts pass;
   failures retained verbatim, not silently accommodated. This is code regression
   evidence, not validation of continuum accuracy or historical claims.
2. MAC hydrostatic manufactured column, wet height 0.5 m, g downward, atmospheric
   p=0 at top, sealed sides/bottom. n=8,16,32 water layers, dt=0.001 s. Both available
   pressure implementations. Maximum pressure error <0.01 Pa, corrected speed
   <1e-6 m/s, actual wet divergence <1e-5 /s. Include f32 predictor roundoff.
3. Fully developed no-slip water channel, gap H=0.001 m, driving acceleration
   a=0.01 m/s², periodic/invariant tangents, lower solid boundary. Starting from
   analytic steady Poiseuille profile, backward Euler dt=1 s until 20 s on 8,16,32
   fluid layers. Compare to u=a*y*(H-y)/(2 nu): RMS relative error <2% at n=8,
   spatial error ratios <0.4 per doubling, mass flux/energy ledgers finite. Separate
   unforced sine decay at fixed n=32, t=0.05 s, 8/16/32 timesteps: errors relative
   to the semidiscrete exact mode must decrease with ratios <0.65; energy must
   decrease. Report continuum and temporal error separately.
4. Water symmetric-strain MAC viscous mode: 0.001 m square, n=8,16,32,
   free-slip outer box, div-free sin/cos vortex, t=0.01 s, stable dt proportional
   to dx². RMS continuum error must decrease with ratio <0.5; kinetic energy
   nonincreasing within implementation rounding allowance. Retain actual errors.
5. Particle preview initially resting water: closed 1 m box, 8/12/16 samples per
   side, no source, t=0.1 s, max requested dt=0.001 s. Record mass, pressure,
   density, kinetic/potential energy, speed, and occupied height. Physical-rest
   screen: speed <0.01*sqrt(g*1m), density RMS deviation <1%, and pressure RMS
   error <5% of rho*g*1m after gauge alignment to hydrostatic depth. Initial
   condition is unpressurized uniform fill: failure diagnoses initialization/
   equilibrium suitability, not time convergence to a prescribed equilibrium.
6. Internal mesh tests: static rotated convex box submerged in filled water;
   static thin oblique two-sided barrier; free-surface-intersecting rotated box
   in half-filled water (seed full box, remove upper-half centers before t=0,
   then attach obstacle). 8/12/16 particle samples, t=0.1 s. Zero accepted center
   penetrations/crossings, relative retained mass drift <1e-12 after initial
   exclusion. Report excluded initial mass separately; not a conservation error.
   100 deterministic normal sweeps at three geometry scales: clearance >=0,
   normal velocity <=1e-10 m/s. Open mesh as solid must refuse. Thin barriers
   intentionally do not partition SPH interactions: compare densities against
   independently summed all-neighbor and same-side-only kernels.
7. Dam-break/obstacle: assess a primary published experimental benchmark before
   selecting any quantitative comparison. Only call experimental validation if
   dimensions, initial fill, sensor locations, observation data and uncertainty
   are available and represented. Otherwise retain the exact blocked comparison
   and run a public-API CPU release/obstacle feasibility specimen; mass drift
   <1e-12 and zero centers in solid are geometric/conservation criteria only.
   No fabricated pressure/front data and no visual-plausibility acceptance.

Generated output goes outside Git, under /workspace/rheon-fluid-audit-evidence.
Authored probes/protocol/report and compact hash manifest go in Git. Exact
commands and actual logs retained externally and archived durably if possible.
No CUDA, GUI, Lean-kernel or scientific validation claims without execution.

Pre-MMS extension (before the added pressure MMS is evaluated; the original
probe campaign has run and its criteria remain unchanged): pressure MMS on
sealed 0.5 m Neumann interval, p=1000*cos(pi*y/0.5) Pa plus arbitrary gauge, zero
wall-normal derivative. Continuum source -lap(p)/rho integrated over each cell
is assembled directly (never via the production operator). n=8/16/32/64,
both pressure implementations. Require RMS error ratio 0.22–0.28 per doubling;
compare gauge-subtracted analytic values, not pressure magnitude.
