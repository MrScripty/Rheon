import Mathlib.Data.Real.Basic
import Mathlib.Data.Real.Sqrt
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring

/-!
Exact-real contracts for the bounded material preview, not a verification of
`ParticleFluidSimulation`, its floating-point execution, neighbor enumeration,
time-step controller, or continuum/multiphase physics.

The pair coefficients below are the coordinate formulas in
`src/particle_fluid.rs`: reference densities `ri,rj`, estimated densities
`di,dj`, particle masses `mi,mj`, and the shared radial kernel factor. Their
symmetry is derived from those formulas, not assumed. The pressure statements
do not assert a compression-energy derivative across the 0.05 density floor.
Rust evaluates radial forces only for `1e-10*h ≤ distance < h`; the polynomial
below specifies that active-pair factor, not an extension of the force outside
its admitted support. Neighbor/support assembly is not proved here.
The viscosity energy bound is for one isolated pair without pressure, source,
gravity, cohesion, wall contact or changing coefficients. The actual controller
has additional conditions; these statements do not prove its global stability.

The nozzle profile/head and reservoir ledger describe the prescribed source.
They do not validate the approximate 0.43 disk-average factor, gas choking,
surface tension, temperature-dependent properties, or wall-pressure support.
-/
namespace Rheon.MaterialPreview
noncomputable section

def liquidPressure (c density reference : ℝ) : ℝ :=
  max (c ^ 2 * (density - reference)) 0

def gasPressure (density gasConstant temperature : ℝ) : ℝ :=
  density * gasConstant * temperature

theorem liquid_pressure_nonnegative (c density reference : ℝ) :
    0 ≤ liquidPressure c density reference := le_max_right _ _

theorem liquid_pressure_below_reference (c density reference : ℝ)
    (h : density ≤ reference) : liquidPressure c density reference = 0 := by
  apply max_eq_right
  exact mul_nonpos_of_nonneg_of_nonpos (sq_nonneg c) (sub_nonpos.mpr h)

theorem gas_reference_pressure (pressure gasConstant temperature : ℝ)
    (hr : gasConstant ≠ 0) (ht : temperature ≠ 0) :
    gasPressure (pressure / (gasConstant * temperature)) gasConstant temperature =
      pressure := by
  unfold gasPressure
  field_simp <;> ring

theorem reference_volume_contribution (reference volume neighborReference kernel : ℝ)
    (hn : neighborReference ≠ 0) :
    reference * ((neighborReference * volume) / neighborReference * kernel) =
      reference * volume * kernel := by
  field_simp <;> ring

def pressurePairCoefficient (pi pj ri rj di dj : ℝ) : ℝ :=
  pi * ri / (rj * di ^ 2) + pj * rj / (ri * dj ^ 2)

def pressurePairForce (mi mj pi pj ri rj di dj gradient displacement : ℝ) : ℝ :=
  -mi * mj * pressurePairCoefficient pi pj ri rj di dj * gradient * displacement

theorem pressure_pair_coefficient_symmetric (pi pj ri rj di dj : ℝ) :
    pressurePairCoefficient pi pj ri rj di dj =
      pressurePairCoefficient pj pi rj ri dj di := by
  unfold pressurePairCoefficient
  ring

theorem pressure_pair_action_reaction (mi mj pi pj ri rj di dj gradient delta : ℝ) :
    pressurePairForce mi mj pi pj ri rj di dj gradient delta +
      pressurePairForce mj mi pj pi rj ri dj di gradient (-delta) = 0 := by
  unfold pressurePairForce
  rw [pressure_pair_coefficient_symmetric pj pi rj ri dj di]
  ring

def poly6RadialGradient (coefficient h distance : ℝ) : ℝ :=
  -6 * coefficient * (h ^ 2 - distance ^ 2) ^ 2

theorem poly6_radial_gradient_nonpositive (coefficient h distance : ℝ)
    (hc : 0 ≤ coefficient) : poly6RadialGradient coefficient h distance ≤ 0 := by
  unfold poly6RadialGradient
  nlinarith [mul_nonneg hc (sq_nonneg (h ^ 2 - distance ^ 2))]

def viscosityPairCoefficient (mi mj mui muj di dj laplacian : ℝ) : ℝ :=
  mi * mj * ((mui + muj) / 2) / (di * dj) * laplacian

theorem viscosity_pair_coefficient_symmetric (mi mj mui muj di dj laplacian : ℝ) :
    viscosityPairCoefficient mi mj mui muj di dj laplacian =
      viscosityPairCoefficient mj mi muj mui dj di laplacian := by
  unfold viscosityPairCoefficient
  congr 1
  congr 1 <;> ring

theorem viscosity_pair_coefficient_nonnegative (mi mj mui muj di dj laplacian : ℝ)
    (hi : 0 ≤ mi) (hj : 0 ≤ mj) (hui : 0 ≤ mui) (huj : 0 ≤ muj)
    (hdi : 0 < di) (hdj : 0 < dj) (hl : 0 ≤ laplacian) :
    0 ≤ viscosityPairCoefficient mi mj mui muj di dj laplacian := by
  unfold viscosityPairCoefficient
  exact mul_nonneg (div_nonneg
    (mul_nonneg (mul_nonneg hi hj) (div_nonneg (add_nonneg hui huj) (by norm_num)))
    (le_of_lt (mul_pos hdi hdj))) hl

def viscosityForce (coefficient ui uj : ℝ) : ℝ := coefficient * (uj - ui)

theorem viscosity_pair_work (coefficient ui uj : ℝ) :
    ui * viscosityForce coefficient ui uj + uj * viscosityForce coefficient uj ui =
      -coefficient * (ui - uj) ^ 2 := by
  unfold viscosityForce
  ring

theorem viscosity_pair_work_nonpositive (coefficient ui uj : ℝ)
    (hc : 0 ≤ coefficient) :
    ui * viscosityForce coefficient ui uj + uj * viscosityForce coefficient uj ui ≤ 0 := by
  rw [viscosity_pair_work]
  have loss := mul_nonneg hc (sq_nonneg (ui - uj))
  nlinarith

def pairVelocity (mass dt coefficient ui uj : ℝ) : ℝ :=
  ui + dt * viscosityForce coefficient ui uj / mass

theorem viscosity_pair_momentum (mi mj dt coefficient ui uj : ℝ)
    (hi : mi ≠ 0) (hj : mj ≠ 0) :
    mi * pairVelocity mi dt coefficient ui uj +
      mj * pairVelocity mj dt coefficient uj ui = mi * ui + mj * uj := by
  unfold pairVelocity viscosityForce
  field_simp <;> ring

theorem viscosity_pair_energy_identity (mi mj dt coefficient ui uj : ℝ)
    (hi : mi ≠ 0) (hj : mj ≠ 0) :
    mi / 2 * (pairVelocity mi dt coefficient ui uj) ^ 2 +
      mj / 2 * (pairVelocity mj dt coefficient uj ui) ^ 2 -
      (mi / 2 * ui ^ 2 + mj / 2 * uj ^ 2) =
      -(dt * coefficient) *
        (1 - dt * coefficient / 2 * (1 / mi + 1 / mj)) * (ui - uj) ^ 2 := by
  unfold pairVelocity viscosityForce
  field_simp <;> ring

theorem viscosity_pair_energy_nonincrease (mi mj dt coefficient ui uj : ℝ)
    (hi : 0 < mi) (hj : 0 < mj) (hdt : 0 ≤ dt) (hc : 0 ≤ coefficient)
    (hstep : dt * coefficient * (1 / mi + 1 / mj) ≤ 2) :
    mi / 2 * (pairVelocity mi dt coefficient ui uj) ^ 2 +
      mj / 2 * (pairVelocity mj dt coefficient uj ui) ^ 2 ≤
      mi / 2 * ui ^ 2 + mj / 2 * uj ^ 2 := by
  have identity := viscosity_pair_energy_identity mi mj dt coefficient ui uj
    (ne_of_gt hi) (ne_of_gt hj)
  have factor : 0 ≤ 1 - dt * coefficient / 2 * (1 / mi + 1 / mj) := by
    nlinarith
  have loss := mul_nonneg (mul_nonneg (mul_nonneg hdt hc) factor) (sq_nonneg (ui - uj))
  nlinarith

def nozzleProfile (radialSquared verticalFraction : ℝ) : ℝ :=
  (1 - radialSquared) ^ 2 * (3 / 4 + verticalFraction / 4)

theorem nozzle_profile_bounds (radialSquared verticalFraction : ℝ)
    (hr0 : 0 ≤ radialSquared) (hr1 : radialSquared ≤ 1)
    (hv0 : -1 ≤ verticalFraction) (hv1 : verticalFraction ≤ 1) :
    0 ≤ nozzleProfile radialSquared verticalFraction ∧
      nozzleProfile radialSquared verticalFraction ≤ 1 := by
  have hs0 := sq_nonneg (1 - radialSquared)
  have hs1 : (1 - radialSquared) ^ 2 ≤ 1 := by nlinarith
  have ht0 : 0 ≤ (3 / 4 : ℝ) + verticalFraction / 4 := by linarith
  have ht1 : (3 / 4 : ℝ) + verticalFraction / 4 ≤ 1 := by linarith
  unfold nozzleProfile
  constructor
  · exact mul_nonneg hs0 ht0
  · nlinarith [mul_nonneg (sub_nonneg.mpr hs1) ht0]

def nozzleSpeed (head density profile : ℝ) : ℝ :=
  Real.sqrt (2 * head / density) * Real.sqrt profile

theorem nozzle_pressure_head_identity (head density profile : ℝ)
    (hh : 0 ≤ head) (hd : 0 < density) (hp : 0 ≤ profile) :
    density / 2 * (nozzleSpeed head density profile) ^ 2 = head * profile := by
  unfold nozzleSpeed
  rw [mul_pow, Real.sq_sqrt (div_nonneg (by linarith) (le_of_lt hd)), Real.sq_sqrt hp]
  field_simp <;> ring

theorem reservoir_mass_ledger (density credit flux dt parcelVolume count : ℝ) :
    count * (density * parcelVolume) +
      density * (credit + flux * dt - count * parcelVolume) =
      density * (credit + flux * dt) := by ring

end
end Rheon.MaterialPreview
