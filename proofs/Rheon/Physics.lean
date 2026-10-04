import Mathlib.Data.Real.Basic
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Fin
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.FieldSimp

/-! Finite exact-real research contracts. These hypotheses are not established
by mesh queries, a pressure solver, floating-point execution, or a material fit. -/
namespace Rheon.Physics
noncomputable section
open scoped BigOperators

theorem force_work_identity (m u a dt : ℝ) :
    m / 2 * (u + dt * a)^2 - m / 2 * u^2 =
      m * dt * a * u + m / 2 * dt^2 * a^2 := by ring

theorem hydrostatic_increment (u g dt dp rho h : ℝ)
    (hr : rho ≠ 0) (hh : h ≠ 0) (balance : dp = -rho * g * h) :
    u - dt * g - dt * (dp / (rho * h)) = u := by
  rw [balance]
  field_simp
  ring

theorem positive_transmissibility (area rho distance : ℝ)
    (ha : 0 < area) (hr : 0 < rho) (hd : 0 < distance) :
    0 < area / (rho * distance) :=
  div_pos ha (mul_pos hr hd)

theorem weighted_pressure_energy {m : ℕ} (area rho distance gradient : Fin m → ℝ)
    (ha : ∀ e, 0 < area e) (hr : ∀ e, 0 < rho e) (hd : ∀ e, 0 < distance e) :
    0 ≤ ∑ e, (area e / (rho e * distance e)) * (gradient e)^2 := by
  exact Finset.sum_nonneg (fun e _ => mul_nonneg
    (le_of_lt (positive_transmissibility _ _ _ (ha e) (hr e) (hd e))) (sq_nonneg _))

theorem internal_amount_balance {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0) (flux : Fin m → ℝ) :
    (∑ i, ∑ e, B i e * flux e) = 0 := by
  rw [Finset.sum_comm]
  simp_rw [← Finset.sum_mul, balanced, zero_mul]
  simp

theorem sealed_flux_compatibility {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0) (u : Fin m → ℝ)
    (Q source : Fin n → ℝ)
    (constraint : ∀ i, (∑ e, B i e * u e) + Q i = source i) :
    (∑ i, Q i) = ∑ i, source i := by
  have h := Finset.sum_congr (s₁ := Finset.univ) (s₂ := Finset.univ) rfl
    (fun i _ => constraint i)
  rw [Finset.sum_add_distrib, internal_amount_balance B balanced u, zero_add] at h
  exact h

theorem volume_source_balance {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0)
    (amount source : Fin n → ℝ) (flux : Fin m → ℝ) (dt : ℝ) :
    (∑ i, (amount i - dt * (∑ e, B i e * flux e) + dt * source i)) =
      (∑ i, amount i) + dt * ∑ i, source i := by
  rw [Finset.sum_add_distrib, Finset.sum_sub_distrib,
    ← Finset.mul_sum, ← Finset.mul_sum, internal_amount_balance B balanced flux]
  ring

theorem weighted_energy_identity {n : ℕ} (mass u v : Fin n → ℝ) :
    (∑ i, mass i * u i ^ 2) - (∑ i, mass i * v i ^ 2) =
      (∑ i, mass i * (u i - v i)^2) +
        2 * ∑ i, mass i * (u i - v i) * v i := by
  rw [← Finset.sum_sub_distrib, Finset.mul_sum, ← Finset.sum_add_distrib]
  apply Finset.sum_congr rfl
  intro i _
  ring

theorem implicit_energy_identity {n : ℕ} (mass u v : Fin n → ℝ)
    (dt dissipation : ℝ)
    (work : (∑ i, mass i * (u i - v i) * v i) = dt * dissipation) :
    (∑ i, mass i * u i ^ 2) - (∑ i, mass i * v i ^ 2) =
      (∑ i, mass i * (u i - v i)^2) + 2 * dt * dissipation := by
  rw [weighted_energy_identity, work]
  ring

theorem implicit_energy_nonincrease {n : ℕ} (mass u v : Fin n → ℝ)
    (dt dissipation : ℝ) (hm : ∀ i, 0 ≤ mass i)
    (ht : 0 ≤ dt) (hd : 0 ≤ dissipation)
    (work : (∑ i, mass i * (u i - v i) * v i) = dt * dissipation) :
    (∑ i, mass i * v i ^ 2) ≤ ∑ i, mass i * u i ^ 2 := by
  have hs : 0 ≤ ∑ i, mass i * (u i - v i)^2 :=
    Finset.sum_nonneg (fun i _ => mul_nonneg (hm i) (sq_nonneg _))
  have hp : 0 ≤ dt * dissipation := mul_nonneg ht hd
  have he := implicit_energy_identity mass u v dt dissipation work
  linarith

theorem slip_power_nonpositive (beta relative_speed : ℝ) (hb : 0 ≤ beta) :
    (-beta * relative_speed) * relative_speed ≤ 0 := by
  nlinarith [mul_nonneg hb (sq_nonneg relative_speed)]

theorem young_adhesion_identity (solid_vapor solid_liquid sigma cosine : ℝ)
    (young : solid_vapor - solid_liquid = sigma * cosine) :
    solid_vapor + sigma - solid_liquid = sigma * (1 + cosine) := by
  nlinarith

theorem adhesion_bounds (sigma cosine : ℝ) (hs : 0 ≤ sigma)
    (hl : -1 ≤ cosine) (hu : cosine ≤ 1) :
    0 ≤ sigma * (1 + cosine) ∧ sigma * (1 + cosine) ≤ 2 * sigma := by
  constructor
  · exact mul_nonneg hs (by linarith)
  · nlinarith [mul_nonneg hs (show 0 ≤ 1 - cosine by linarith)]

end
end Rheon.Physics
