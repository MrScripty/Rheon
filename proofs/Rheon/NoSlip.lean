import Rheon.WallFriction

/-! Exact-real identities for compatible, fixed endpoint constraints.
The reaction impulse cancels the unconstrained force impulse at constrained
nodes. These statements assume the step equation and assembled force power;
they do not prove stencil assembly, a Rust/IEEE bound, stability or convergence.
An incompatible initial trace or a changing prescribed speed is not modeled. -/
namespace Rheon.NoSlip
noncomputable section
open scoped BigOperators

theorem compatible_reaction_preserves_trace (u wall mass dt force : ℝ)
    (trace : u = wall) :
    u + (dt * force + (-dt * force)) / mass = wall := by
  rw [trace]
  ring

theorem relative_reaction_work_zero (u wall impulse : ℝ) (trace : u = wall) :
    impulse * (u - wall) = 0 := by rw [trace]; ring

theorem momentum_balance {n : ℕ} (mass delta force impulse : Fin n → ℝ)
    (dt : ℝ)
    (step : ∀ i, mass i * delta i = dt * force i + impulse i) :
    (∑ i, mass i * delta i) = dt * (∑ i, force i) + ∑ i, impulse i := by
  calc
    _ = ∑ i, (dt * force i + impulse i) := by
      apply Finset.sum_congr rfl
      intro i _
      exact step i
    _ = dt * (∑ i, force i) + ∑ i, impulse i := by
      rw [Finset.sum_add_distrib, Finset.mul_sum]

theorem constrained_work_identity {n : ℕ}
    (mass u delta force impulse wall : Fin n → ℝ)
    (dt bulk navierDissipation navierPower : ℝ)
    (step : ∀ i, mass i * delta i = dt * force i + impulse i)
    (power : (∑ i, force i * u i) = navierPower - bulk - navierDissipation)
    (compatibleWork : ∀ i, impulse i * u i = impulse i * wall i) :
    (∑ i, mass i / 2 * ((u i + delta i)^2 - (u i)^2 - (delta i)^2)) +
      dt * (bulk + navierDissipation - navierPower) - ∑ i, impulse i * wall i = 0 := by
  have hp : (∑ i, mass i * delta i * u i) =
      dt * (navierPower - bulk - navierDissipation) + ∑ i, impulse i * wall i := by
    calc
      _ = ∑ i, (dt * (force i * u i) + impulse i * wall i) := by
        apply Finset.sum_congr rfl
        intro i _
        rw [step]
        calc
          (dt * force i + impulse i) * u i =
              dt * (force i * u i) + impulse i * u i := by ring
          _ = dt * (force i * u i) + impulse i * wall i := by rw [compatibleWork]
      _ = dt * (∑ i, force i * u i) + ∑ i, impulse i * wall i := by
        rw [Finset.sum_add_distrib, Finset.mul_sum]
      _ = dt * (navierPower - bulk - navierDissipation) + ∑ i, impulse i * wall i := by rw [power]
  calc
    _ = (∑ i, mass i * delta i * u i) +
        dt * (bulk + navierDissipation - navierPower) - ∑ i, impulse i * wall i := by
      congr 2
      apply Finset.sum_congr rfl
      intro i _
      ring
    _ = 0 := by rw [hp]; ring

end
end Rheon.NoSlip
