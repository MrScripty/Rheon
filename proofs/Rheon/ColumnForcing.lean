import Rheon.NoSlip

/-! Exact-real algebra for uniform tangential forcing in the fixed column.
Mass is rho times liquid dual volume; constraints are compatible and fixed.
Step, internal-force cancellation and assembled-power hypotheses are explicit.
These statements do not prove Rust/IEEE bounds, stencil assembly, the spectral
reference, transient convergence, mesh fluid coupling or capillarity. -/
namespace Rheon.ColumnForcing
noncomputable section
open scoped BigOperators

theorem units_agree (rho volume acceleration densityForce : ℝ)
    (units : densityForce = rho * acceleration) :
    (rho * volume) * acceleration = volume * densityForce := by rw [units]; ring

theorem reaction_includes_body_force (u wall mass dt viscous navier body : ℝ)
    (trace : u = wall) :
    u + (dt * (viscous + navier + body) + (-dt * (viscous + navier + body))) / mass = wall := by
  exact Rheon.NoSlip.compatible_reaction_preserves_trace u wall mass dt
    (viscous + navier + body) trace

theorem forced_momentum {n : ℕ} (mass delta internal body wall : Fin n → ℝ)
    (dt : ℝ)
    (step : ∀ i, mass i * delta i = dt * (internal i + body i) + wall i)
    (cancellation : ∑ i, internal i = 0) :
    (∑ i, mass i * delta i) = dt * (∑ i, body i) + ∑ i, wall i := by
  have h := Rheon.NoSlip.momentum_balance mass delta (fun i => internal i + body i) wall dt step
  simpa only [Finset.sum_add_distrib, cancellation, zero_add] using h

theorem forced_work {n : ℕ}
    (mass u delta force reaction wall : Fin n → ℝ)
    (dt bulk navierLoss navierPower bodyPower : ℝ)
    (step : ∀ i, mass i * delta i = dt * force i + reaction i)
    (power : (∑ i, force i * u i) = navierPower + bodyPower - bulk - navierLoss)
    (compatibleWork : ∀ i, reaction i * u i = reaction i * wall i) :
    (∑ i, mass i / 2 * ((u i + delta i)^2 - (u i)^2 - (delta i)^2)) +
      dt * (bulk + navierLoss - navierPower - bodyPower) -
      ∑ i, reaction i * wall i = 0 := by
  have h := Rheon.NoSlip.constrained_work_identity mass u delta force reaction wall
    dt bulk navierLoss (navierPower + bodyPower) step power compatibleWork
  convert h using 1; ring

def parabola (G mu H y : ℝ) : ℝ := G / (2 * mu) * y * (H - y)

theorem parabola_endpoints (G mu H : ℝ) :
    parabola G mu H 0 = 0 ∧ parabola G mu H H = 0 := by
  unfold parabola
  constructor <;> ring

theorem quadratic_stencil_balance (G mu H y h area : ℝ) (viscosity : mu ≠ 0)
    (spacing : h ≠ 0) :
    (mu * area / h) * (parabola G mu H (y-h) - 2 * parabola G mu H y +
      parabola G mu H (y+h)) + area * h * G = 0 := by
  unfold parabola
  field_simp
  ring

theorem recurrence_about_equilibrium (mass dt stiffness old next left right
    eq eqLeft eqRight body : ℝ)
    (step : mass * (next-old) = dt * (stiffness * (left-2*old+right) + body))
    (steady : stiffness * (eqLeft-2*eq+eqRight) + body = 0) :
    mass * (next-eq) = mass * (old-eq) +
      dt * stiffness * ((left-eqLeft)-2*(old-eq)+(right-eqRight)) := by
  nlinarith [mul_eq_zero_of_right dt steady]

end
end Rheon.ColumnForcing
