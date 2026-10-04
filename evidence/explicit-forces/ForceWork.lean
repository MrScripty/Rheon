import Rheon.Physics

/-! Additive exact-real contracts for the explicit-force milestone. The mass
weights and updates are supplied finite values. Face assembly, IEEE rounding,
pressure convergence and physical validation are not proved here. -/
namespace Rheon.ExplicitForces
noncomputable section
open scoped BigOperators

def kineticEnergy {n : ℕ} (mass velocity : Fin n → ℝ) : ℝ :=
  (∑ i, mass i * (velocity i)^2) / 2

def storedWork {n : ℕ} (mass old new : Fin n → ℝ) : ℝ :=
  ∑ i, mass i * (new i - old i) * (old i + (new i - old i) / 2)

theorem stored_work_energy_change {n : ℕ} (mass old new : Fin n → ℝ) :
    storedWork mass old new = kineticEnergy mass new - kineticEnergy mass old := by
  unfold storedWork kineticEnergy
  simp_rw [div_eq_mul_inv]
  rw [Finset.sum_mul, Finset.sum_mul, ← Finset.sum_sub_distrib]
  apply Finset.sum_congr rfl
  intro i _
  ring

theorem acceleration_work {n : ℕ} (mass old acceleration : Fin n → ℝ) (dt : ℝ) :
    kineticEnergy mass (fun i => old i + dt * acceleration i) - kineticEnergy mass old =
      ∑ i : Fin n, (mass i * dt * acceleration i * old i +
        mass i / 2 * dt^2 * (acceleration i)^2) := by
  unfold kineticEnergy
  simp_rw [div_eq_mul_inv]
  rw [Finset.sum_mul, Finset.sum_mul, ← Finset.sum_sub_distrib]
  apply Finset.sum_congr rfl
  intro i _
  ring

theorem force_density_increment (rho u f dt : ℝ) (hr : rho ≠ 0) :
    rho * ((u + dt * (f / rho)) - u) = dt * f := by
  field_simp
  ring

theorem hydrostatic_acceleration_balance (rho h u a dt dp : ℝ)
    (hr : rho ≠ 0) (hh : h ≠ 0) (balance : dp = rho * a * h) :
    u + dt * a - dt * (dp / (rho * h)) = u := by
  rw [balance]
  field_simp
  ring

end
end Rheon.ExplicitForces
