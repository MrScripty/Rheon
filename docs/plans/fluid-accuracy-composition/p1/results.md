# P1 selection results: neither adaptation is production-admissible

The frozen campaign completed at source **39098e0e0b0b0f8c966ff7f811d548b6d396ec15**:
108 hydrostatic trajectories, 162 shear trajectories, 36 actual 3D cloud operator
studies and 18 planar isolation diagnostics. All 270 reduced trajectories reached
.1 s without a refused interval. Successful execution is not physical acceptance.
The fixed speed/pressure/traction/work criteria reject these adaptations. Return
P1 to method selection; do not change water defaults, normalize density alone,
or promote either candidate. The original preview and frozen failed rest
screens are unchanged.

The [protocol](protocol.md) fixes dimensions, coefficients, geometry, versions
and targets before execution. [Admission](admission.md) records authority and
exclusive write set; [review](review.md) records independent findings. Full P-W
production acceptance remains pending, including actual 3D native wall
trajectories, geometry and discrete timestep qualification.

## Matched planar results

Representative finest hydrostatic results below use n=32, h/dx=1.5, support
R=2h, dt=2.5e-5 s, c=32 m/s. Pressure error is RMS/(rho0*g*H), density error is
RMS relative to the compressed equilibrium at actual parcel positions. Targets
remain .01 m/s speed, 2% pressure and 1% density.

| Adaptation / cloud | Max speed (m/s) | Pressure error | Density error |
|---|---:|---:|---:|
| AHA dummy / ordered | .001558 | .1931% | .001843% |
| AHA dummy / perturbed | **.167893** | **19.0076%** | .3729% |
| SA gamma k=0 / ordered | **.062635** | **4.6447%** | .04506% |
| SA gamma k=0 / perturbed | **.190580** | **22.1342%** | .3218% |

Both methods pass all 54 density targets individually. AHA passes only 24/54
speed and 27/54 pressure targets; SA passes neither speed nor pressure target
in any of its 54 runs. Thus passing density RMS targets does not establish an
acceptable pressure/force method. The perturbed cloud is the prescribed seeded
10% displacement with fixed mass, not a uniform unpressurized fill.

At n=64, h/dx=1.5, dt=2.5e-5 s, reduced no-slip shear gives:

| Adaptation / flow | Velocity RMS error | Wall traction error |
|---|---:|---:|
| AHA / Poiseuille | .1483% | .06702% |
| AHA / Couette | .009288% | .1537% |
| SA / Poiseuille | .1221% | **29.3342%** |
| SA / Couette | .04883% | **30.3604%** |

Across all support/dt choices, all 18 AHA fine no-slip profiles pass the 2%
velocity and 5% traction screens. Its measured pairwise spatial orders range
.9733–2.1852. SA also passes fine velocity, but fails all 18 fine traction
comparisons (up to 35.87% error); orders range .5700–1.1444 and do not meet the
fixed .9 target throughout. A good velocity profile alone would conceal this
traction error. The largest SA momentum/source/reaction ledger defect is
3.3833e-4 kg/(m s), versus the fixed nonzero-scale tolerance 9.982e-12 kg/(m s).
AHA's largest absolute shear ledger defect is 5.28e-17 kg/(m s). Stationary-wall
mechanical work is exactly zero in the recorded Poiseuille/slip arithmetic.
Both nonzero free-slip constant profiles remain invariant (worst relative
roundoff 8.58e-17) with measured zero traction.

## EOS work and admission

Mass is fixed in both reduced formulations: all mass ledger defects are zero
at recorded precision. No accepted center crosses the bottom plane. Full
internal+kinetic+gravity energy is evaluated, with stationary wall work zero.
The independently accumulated finite-step kinetic/gravity kick,
kicked-continuity and nonlinear EOS chain terms close the arithmetic balance
to at most **7.25e-12 J/m²**. Independent reconstruction from all raw state exports
confirms the pressure/RMS/energy observations. That small closure error is
**not** a conservation pass: it accounts for an inconsistent method's defect.

Integrated pressure/EOS work defects range −.4248 to **+6.5868 J/m²** for AHA,
and −6.8197 to **+4.1195 J/m²** for SA. The plan's characteristic 1e-8 energy
scale here is 9.7890e-5 J/m². These large defects cannot be relabelled wall work
or physical dissipation. SA's gamma-dependent pair forces also lack exact
physical-mass momentum cancellation. The measured matched wall surface term
alone does not close that physical ledger; assigning the missing force to an
invented body reaction would hide the failing formulation.

The linear EOS reference follows dp/dy=−rho*g: rho=rho0*exp(g*(H−y)/c²),
p=rho0*c²*expm1(g*(H−y)/c²). Exact source-interval integrals supply reference
mass per area; no global rescaling or lattice multiplier is used. Its internal
energy e=c²*(log(rho/rho0)+rho0/rho−1) on the compressed branch satisfies
rho²*de/drho=p. The source includes branch, integral, derivative and invalid-input
checks, including the zero-gravity limit.

For 1 m and 1% allowed compression, c>=31.393633 m/s; c=20 would give 2.481963%
compression. The frozen c=32 meets that continuum criterion. The low-Mach
limit is U<=3.2 m/s and every retained trajectory stays below it. Neither fact
supplies a discrete acoustic stability bound. `production_dt` returns
`UnqualifiedAcousticOperator`; it does not disguise h/c as a proof. The reduced
hydro trajectories use fixed diagnostic timesteps, not admitted production
timesteps. Their kick/continuity/drift ordering differs from the original papers.
Native assembled shear matrices provide measured floating-point explicit
contraction intervals; actual and shortened-final timesteps are retained. This
linear constant-density check does not establish nonlinear pressure, geometry,
capillary or future variable-volume stability.

## F5 native 3D consistency findings

The cloud campaign evaluates actual native 3D kernel/viscous sums on 16/24/32
periodic cubes with ordered and norm-bounded perturbed clouds. It uses a
constant-density operator isolation, not the current preview's full pressure,
estimated-density and contact update. The poly6/spiky variant studies those
kernel shapes at the same declared supports; it is not a replay of the preview's
fixed R=2dx default. No production result is silently reinterpreted.

At n=32, Wendland/Morris initial shear decay-rate error is .5300–1.5897%,
passing the necessary 2% instantaneous rate screen in all six support/cloud
cases. Poly6/spiky errors range .8300–5.2362%, passing only two of six. All
observed pair kinetic powers are negative, and absolute closed momentum rates
are <=1.01e-22 N. This is an instantaneous rate/consistency study, not a full
native trajectory-decay gate.

At fixed h/dx=1.5 the ordered Wendland partition bias is .3141% and its
Laplacian moment bias .06193%; refining spacing with the same dimensionless
stencil preserves those biases. For the poly6/spiky support study the analogous
ordered Laplacian bias is 4.4068%. At h/dx=2.5 Wendland partition and Laplacian
biases fall to .01379% and .003500%. Increasing support reduces sampling bias
while broadening the physical kernel, so it is not interchangeable with spatial
refinement. No preset coefficient is calibrated to erase one measured moment.
This is useful direction for the next selection study, not a selected pressure/
wall method.

## Isolation and capability limits

All 18 infinite-slab diagnostic queries mask excluded density, pressure and
viscosity support factors exactly; 12 have nonzero unmasked kernels. Fifteen
volumetric support requests are explicitly refused, including zero thickness.
Angles 0/17/31 degrees and an independent local-coordinate segment oracle are
covered. The three thicker cases satisfy only this experiment's conservative
support envelope. These diagnostics do not execute a mesh-equipped production
particle solver, match wall loads, or qualify finite thin surfaces, corners,
curvature, free-surface/wall junctions or imported obstacles.

The candidate implementations are explicit adaptations of primary-source
families, not reproductions or counterexamples to the published papers. The
semi-analytical adaptation uses exact half-space/channel gamma and a native
near-wall slope law; it does not implement every boundary vertex, diffusion or
free-surface construction in the paper. The dummy adaptation changes EOS,
kernel, fixed-volume force policy and integrator. Results decide these frozen
versions only. No full mesh water, physical impact, GPU, native UI, capillary,
turbulence or experimental water validation is claimed. No new Lean source was
added and no historical proof receipt was refreshed.

## Next decision

P1 remains in method selection. AHA's reduced ordered shear behavior is useful,
but its perturbed-pressure and EOS work defects preclude production selection.
Do not relabel gamma-induced pair defects as wall force without deriving and
verifying matched wall traction. The next bounded direction must derive pressure/continuity as a
single physical-mass adjoint pair with matched wall reaction, then establish
quadrature consistency on perturbed clouds and derive the actual staggered
pressure/viscous timestep bound. If that changes equations or wall quadrature,
freeze a new explicit version and protocol before evaluation. Preserve this
campaign. P2 follows a selected coherent quadrature; G1/C1 remain independent.
