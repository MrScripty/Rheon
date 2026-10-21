import Rheon.Discrete
import Rheon.Transport
import Mathlib.Algebra.Order.BigOperators.Ring.Finset
import Mathlib.Tactic.FieldSimp

/-! Exact-real finite-volume Euler contracts. Shared face fluxes are single values
with balanced incidence. Positivity is conditional on admitted states and explicit
convex/CFL coefficients. No Riemann solver, reconstruction, mesh assembly, IEEE
arithmetic, entropy inequality or continuum convergence is verified here. -/
namespace Rheon.Compressible
noncomputable section
open scoped BigOperators

def faceUpdate {n f k : ℕ} (B : Fin n → Fin f → ℝ)
    (amount : Fin n → Fin k → ℝ) (flux : Fin f → Fin k → ℝ)
    (dt : ℝ) (i : Fin n) (c : Fin k) : ℝ :=
  amount i c + dt * Rheon.Discrete.incidence B (fun e => flux e c) i

/-- One shared flux per face conserves each extensive Euler component. -/
theorem shared_face_conservation {n f k : ℕ} (B : Fin n → Fin f → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0)
    (amount : Fin n → Fin k → ℝ) (flux : Fin f → Fin k → ℝ)
    (dt : ℝ) (c : Fin k) :
    (∑ i, faceUpdate B amount flux dt i c) = ∑ i, amount i c := by
  exact Rheon.Discrete.conservative_update B balanced (fun i => amount i c)
    (fun e => flux e c) dt

def kineticDensity {d : ℕ} (rho : ℝ) (momentum : Fin d → ℝ) : ℝ :=
  ∑ j, (momentum j)^2 / (2 * rho)

def internalDensity {d : ℕ} (rho : ℝ) (momentum : Fin d → ℝ) (energy : ℝ) : ℝ :=
  energy - kineticDensity rho momentum

def pressure {d : ℕ} (gamma rho : ℝ) (momentum : Fin d → ℝ) (energy : ℝ) : ℝ :=
  (gamma - 1) * internalDensity rho momentum energy

def Admissible {d : ℕ} (rho : ℝ) (momentum : Fin d → ℝ) (energy : ℝ) : Prop :=
  0 < rho ∧ 0 < internalDensity rho momentum energy

theorem ideal_gas_pressure_positive {d : ℕ} (gamma rho : ℝ)
    (momentum : Fin d → ℝ) (energy : ℝ) (hg : 1 < gamma)
    (state : Admissible rho momentum energy) :
    0 < pressure gamma rho momentum energy := by
  exact mul_pos (sub_pos.mpr hg) state.2

theorem admissible_iff_pressure_positive {d : ℕ} (gamma rho : ℝ)
    (momentum : Fin d → ℝ) (energy : ℝ) (hg : 1 < gamma) :
    Admissible rho momentum energy ↔ 0 < rho ∧ 0 < pressure gamma rho momentum energy := by
  constructor
  · intro h
    exact ⟨h.1, ideal_gas_pressure_positive gamma rho momentum energy hg h⟩
  · rintro ⟨hr, hp⟩
    exact ⟨hr, (mul_pos_iff_of_pos_left (sub_pos.mpr hg)).mp hp⟩

/-- Positive endpoint values stay strictly positive, including t=0 and t=1. -/
theorem blend_positive (t a b : ℝ) (ht0 : 0 ≤ t) (ht1 : t ≤ 1)
    (ha : 0 < a) (hb : 0 < b) : 0 < Rheon.Transport.blend t a b := by
  unfold Rheon.Transport.blend
  by_cases h : t = 1
  · simp [h, hb]
  · have ht : t < 1 := lt_of_le_of_ne ht1 h
    have hp : 0 < (1 - t) * a := mul_pos (sub_pos.mpr ht) ha
    have hn : 0 ≤ t * b := mul_nonneg ht0 (le_of_lt hb)
    linarith

/-- The exact density-weighted Cauchy remainder; valid for signed momentum. -/
theorem kinetic_blend_remainder (t r s m n : ℝ) (hr : 0 < r) (hs : 0 < s)
    (ht0 : 0 ≤ t) (ht1 : t ≤ 1) :
    (1-t) * (m^2 / (2*r)) + t * (n^2 / (2*s)) -
      (Rheon.Transport.blend t m n)^2 / (2 * Rheon.Transport.blend t r s) =
    t * (1-t) * (s*m-r*n)^2 / (2*r*s*Rheon.Transport.blend t r s) := by
  have hb := blend_positive t r s ht0 ht1 hr hs
  unfold Rheon.Transport.blend at hb ⊢
  field_simp [ne_of_gt hr, ne_of_gt hs, ne_of_gt hb]
  ring

/-- Convexity is proved from the nonnegative square remainder, not assumed. -/
theorem kinetic_density_convex {d : ℕ} (t r s : ℝ) (m n : Fin d → ℝ)
    (hr : 0 < r) (hs : 0 < s) (ht0 : 0 ≤ t) (ht1 : t ≤ 1) :
    kineticDensity (Rheon.Transport.blend t r s) (fun j => Rheon.Transport.blend t (m j) (n j)) ≤
      Rheon.Transport.blend t (kineticDensity r m) (kineticDensity s n) := by
  have hb := blend_positive t r s ht0 ht1 hr hs
  unfold kineticDensity Rheon.Transport.blend
  rw [Finset.mul_sum, Finset.mul_sum, ← Finset.sum_add_distrib]
  apply Finset.sum_le_sum
  intro j _
  have hi := kinetic_blend_remainder t r s (m j) (n j) hr hs ht0 ht1
  have hn : 0 ≤ t * (1-t) * (s*m j-r*n j)^2 /
      (2*r*s*Rheon.Transport.blend t r s) :=
    div_nonneg (mul_nonneg (mul_nonneg ht0 (sub_nonneg.mpr ht1)) (sq_nonneg _))
      (le_of_lt (mul_pos (mul_pos (mul_pos (by norm_num) hr) hs) hb))
  unfold Rheon.Transport.blend at hi hn
  linarith

/-- The ideal-gas admissible set is convex in conservative variables. -/
theorem admissible_blend {d : ℕ} (t r s E F : ℝ) (m n : Fin d → ℝ)
    (ht0 : 0 ≤ t) (ht1 : t ≤ 1)
    (left : Admissible r m E) (right : Admissible s n F) :
    Admissible (Rheon.Transport.blend t r s)
      (fun j => Rheon.Transport.blend t (m j) (n j)) (Rheon.Transport.blend t E F) := by
  refine ⟨blend_positive t r s ht0 ht1 left.1 right.1, ?_⟩
  have hk := kinetic_density_convex t r s m n left.1 right.1 ht0 ht1
  have hp := blend_positive t (internalDensity r m E) (internalDensity s n F)
    ht0 ht1 left.2 right.2
  unfold internalDensity Rheon.Transport.blend at *
  linarith

def normalVelocity {d : ℕ} (rho : ℝ) (momentum normal : Fin d → ℝ) : ℝ :=
  (∑ j, momentum j * normal j) / rho

def eulerMassFlux (rho velocity : ℝ) : ℝ := rho * velocity

def eulerMomentumFlux {d : ℕ} (momentum normal : Fin d → ℝ)
    (p velocity : ℝ) (j : Fin d) : ℝ := momentum j * velocity + p * normal j

def eulerEnergyFlux (energy p velocity : ℝ) : ℝ := (energy + p) * velocity

/-- Stationary slip means zero normal momentum. Pressure still transfers momentum. -/
theorem stationary_slip_wall_flux {d : ℕ} (rho energy p : ℝ)
    (momentum normal : Fin d → ℝ) (slip : (∑ j, momentum j * normal j) = 0) :
    eulerMassFlux rho (normalVelocity rho momentum normal) = 0 ∧
    eulerEnergyFlux energy p (normalVelocity rho momentum normal) = 0 ∧
    ∀ j, eulerMomentumFlux momentum normal p (normalVelocity rho momentum normal) j = p * normal j := by
  simp [normalVelocity, slip, eulerMassFlux, eulerEnergyFlux, eulerMomentumFlux]

/-- A donor-density update with explicit outflow coefficients. This definition
is a convex density contract, not a general Euler Riemann-flux positivity claim. -/
def cflDensity {f : ℕ} (rho : ℝ) (coefficient donor : Fin f → ℝ) : ℝ :=
  (1 - ∑ e, coefficient e) * rho + ∑ e, coefficient e * donor e

theorem cfl_density_nonnegative {f : ℕ} (rho : ℝ) (coefficient donor : Fin f → ℝ)
    (hr : 0 ≤ rho) (hc : ∀ e, 0 ≤ coefficient e)
    (hd : ∀ e, 0 ≤ donor e) (cfl : (∑ e, coefficient e) ≤ 1) :
    0 ≤ cflDensity rho coefficient donor := by
  exact add_nonneg (mul_nonneg (sub_nonneg.mpr cfl) hr)
    (Finset.sum_nonneg (fun e _ => mul_nonneg (hc e) (hd e)))

/-- Strict CFL leaves a positive contribution from the current density. -/
theorem cfl_density_positive {f : ℕ} (rho : ℝ) (coefficient donor : Fin f → ℝ)
    (hr : 0 < rho) (hc : ∀ e, 0 ≤ coefficient e)
    (hd : ∀ e, 0 ≤ donor e) (cfl : (∑ e, coefficient e) < 1) :
    0 < cflDensity rho coefficient donor := by
  exact add_pos_of_pos_of_nonneg (mul_pos (sub_pos.mpr cfl) hr)
    (Finset.sum_nonneg (fun e _ => mul_nonneg (hc e) (hd e)))

end
end Rheon.Compressible
