import Mathlib.Data.Real.Basic
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Data.Fintype.Fin
import Mathlib.Tactic.Ring

/- Conditional exact-real statements only. Geometry, source/trace compatibility,
   source authentication and binary64 evaluation are not certified here. -/
namespace Rheon.ObstacleGradient
open scoped BigOperators

/-- Directional affine quotient: all other coordinate contributions are in c.
    The nonzero separation and compatible endpoint values are explicit premises. -/
theorem affine_quotient (c slope xminus xplus uminus uplus : ℝ)
    (separation : xplus - xminus ≠ 0)
    (lower_value : uminus = c + slope * xminus)
    (upper_value : uplus = c + slope * xplus) :
    (uplus - uminus) / (xplus - xminus) = slope := by
  rw [lower_value, upper_value]
  apply (div_eq_iff separation).2
  ring

/-- A stored row reproduces slope only with an exact reciprocal premise.
    Nearest-rounded native reciprocal coefficients do not establish this premise. -/
theorem stored_affine (c slope xminus xplus uminus uplus inverse : ℝ)
    (exact_reciprocal : inverse * (xplus - xminus) = 1)
    (lower_value : uminus = c + slope * xminus)
    (upper_value : uplus = c + slope * xplus) :
    (-inverse) * uminus + inverse * uplus = slope := by
  calc
    _ = slope * (inverse * (xplus - xminus)) := by rw [lower_value, upper_value]; ring
    _ = slope := by rw [exact_reciprocal]; ring

/-- Elimination of a stationary trace needs its zero-value premise. -/
theorem stationary_trace_elimination (coefficient trace value : ℝ)
    (stationary_trace : trace = 0) :
    coefficient * (value - trace) = coefficient * value := by
  rw [stationary_trace]
  ring

/-- Same G and W on each side; no exact reciprocal or geometric premise.
    Stored finite coefficients interpreted as reals satisfy this algebraic identity. -/
theorem matched_weighted_transpose {r f : ℕ}
    (G : Fin r → Fin f → ℝ) (w q : Fin r → ℝ) (u : Fin f → ℝ) :
    (∑ i, w i * (∑ j, G i j * u j) * q i) =
      ∑ j, u j * (∑ i, G i j * w i * q i) := by
  simp_rw [Finset.mul_sum, Finset.sum_mul]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro j _
  apply Finset.sum_congr rfl
  intro i _
  ring

/-- Positive weights give a nonnegative directed-row norm, not a physical
    symmetric-strain energy or an acceptance theorem. -/
theorem row_square_nonnegative {r : ℕ} (w values : Fin r → ℝ)
    (nonnegative_weights : ∀ i, 0 ≤ w i) :
    0 ≤ ∑ i, w i * values i ^ 2 := by
  exact Finset.sum_nonneg (fun i _ => mul_nonneg (nonnegative_weights i) (sq_nonneg _))
end Rheon.ObstacleGradient

#print axioms Rheon.ObstacleGradient.affine_quotient
#print axioms Rheon.ObstacleGradient.stored_affine
#print axioms Rheon.ObstacleGradient.stationary_trace_elimination
#print axioms Rheon.ObstacleGradient.matched_weighted_transpose
#print axioms Rheon.ObstacleGradient.row_square_nonnegative
