import Rheon.Physics

/-! Exact-real contracts for the fixed flat-slab wall-friction update.
These do not certify Rust arithmetic, spatial convergence or wetting. -/
namespace Rheon.WallFriction
noncomputable section
open scoped BigOperators

theorem wall_force_work (beta u wall : ℝ) :
    (-beta * (u - wall)) * u =
      -beta * (u - wall)^2 + (-beta * (u - wall)) * wall := by ring

theorem common_translation (beta u wall shift : ℝ) :
    -beta * ((u + shift) - (wall + shift)) = -beta * (u - wall) := by ring

theorem relative_dissipation_nonnegative {n : ℕ} (beta u wall : Fin n → ℝ)
    (hb : ∀ i, 0 ≤ beta i) : 0 ≤ ∑ i, beta i * (u i - wall i)^2 := by
  exact Finset.sum_nonneg (fun i _ => mul_nonneg (hb i) (sq_nonneg _))

theorem explicit_work_identity {n : ℕ} (mass u delta force : Fin n → ℝ)
    (dt bulk wallDissipation actuatorPower : ℝ)
    (step : ∀ i, mass i * delta i = dt * force i)
    (power : (∑ i, force i * u i) = actuatorPower - bulk - wallDissipation) :
    (∑ i, mass i / 2 * ((u i + delta i)^2 - (u i)^2 - (delta i)^2)) +
      dt * (bulk + wallDissipation - actuatorPower) = 0 := by
  have hp : (∑ i, mass i * delta i * u i) =
      dt * (actuatorPower - bulk - wallDissipation) := by
    calc
      (∑ i, mass i * delta i * u i) = ∑ i, dt * force i * u i := by
        apply Finset.sum_congr rfl
        intro i _
        rw [step]
      _ = dt * ∑ i, force i * u i := by
        rw [Finset.mul_sum]
        apply Finset.sum_congr rfl
        intro i _
        ring
      _ = dt * (actuatorPower - bulk - wallDissipation) := by rw [power]
  calc
    _ = (∑ i, mass i * delta i * u i) +
        dt * (bulk + wallDissipation - actuatorPower) := by
      congr 1
      apply Finset.sum_congr rfl
      intro i _
      ring
    _ = 0 := by rw [hp]; ring

end
end Rheon.WallFriction
