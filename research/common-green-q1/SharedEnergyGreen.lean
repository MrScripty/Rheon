import Mathlib.Data.Real.Basic
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Fin
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring

/- Conditional exact-real finite algebra. The actual Q1 prolongation, exact
   Simpson integration, broken-cell integration by parts and geometry premises
   are independently checked in the bounded lab, not proved by this leaf. -/
namespace Rheon.CommonGreenQ1
open scoped BigOperators

/-- Nonnegative symmetric-strain quadrature gives nonnegative energy. -/
theorem gram_energy_nonnegative {n : ℕ} (w strain : Fin n → ℝ)
    (positive_weights : ∀ r, 0 ≤ w r) :
    0 ≤ (1/2 : ℝ) * ∑ r, w r * strain r ^ 2 := by
  apply mul_nonneg (by norm_num)
  exact Finset.sum_nonneg (fun r _ => mul_nonneg (positive_weights r) (sq_nonneg _))

/-- Matrix gather/transpose work; no constitutive or geometry premises. -/
theorem pairing_transpose {n k : ℕ} (A : Fin n → Fin k → ℝ)
    (lambda : Fin n → ℝ) (test : Fin k → ℝ) :
    (∑ j, test j * (∑ r, A r j * lambda r)) =
      ∑ r, lambda r * (∑ j, A r j * test j) := by
  simp_rw [Finset.mul_sum]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro r _
  apply Finset.sum_congr rfl
  intro j _
  ring

/-- Work of the explicitly matched negative transpose equals minus twice energy. -/
theorem matched_transpose_work {n k : ℕ} (A : Fin n → Fin k → ℝ)
    (w : Fin n → ℝ) (x : Fin k → ℝ) :
    (∑ j, x j * (-(∑ r, w r * A r j * (∑ t, A r t * x t)))) =
      - (∑ r, w r * (∑ t, A r t * x t) ^ 2) := by
  calc
    _ = -(∑ j, x j * (∑ r, A r j * (w r * (∑ t, A r t * x t)))) := by
      simp only [mul_neg, Finset.sum_neg_distrib]
      congr 1
      apply Finset.sum_congr rfl
      intro j _
      congr 1
      apply Finset.sum_congr rfl
      intro r _
      ring
    _ = -(∑ r, (w r * (∑ t, A r t * x t)) * (∑ j, A r j * x j)) := by
      rw [pairing_transpose]
    _ = _ := by
      congr 1
      apply Finset.sum_congr rfl
      intro r _
      ring

/-- A represented common rigid field in the row kernel has zero transpose work. -/
theorem rigid_kernel_work {n k : ℕ} (A : Fin n → Fin k → ℝ)
    (w stress : Fin n → ℝ) (rigid : Fin k → ℝ)
    (rigid_kernel : ∀ r, (∑ j, A r j * rigid j) = 0) :
    (∑ j, rigid j * (-(∑ r, w r * A r j * stress r))) = 0 := by
  calc
    _ = -(∑ j, rigid j * (∑ r, A r j * (w r * stress r))) := by
      simp only [mul_neg, Finset.sum_neg_distrib]
      congr 1
      apply Finset.sum_congr rfl
      intro j _
      congr 1
      apply Finset.sum_congr rfl
      intro r _
      ring
    _ = -(∑ r, (w r * stress r) * (∑ j, A r j * rigid j)) := by
      rw [pairing_transpose]
    _ = 0 := by simp only [rigid_kernel, mul_zero, Finset.sum_const_zero, neg_zero]

/-- An element Green premise identifies when reaction can equal wall traction.
    Bulk and jump terms must be independently defined, not inferred by a gap. -/
theorem green_reaction_equals_wall_iff (reaction wall bulk jump : ℝ)
    (independent_green : reaction = wall + bulk - jump) :
    reaction = wall ↔ bulk = jump := by
  constructor <;> intro h <;> linarith

/-- The scalar wall-ray penalty cannot have a common-rotation kernel. -/
theorem scalar_ray_rotation_positive (mu area delta omega : ℝ)
    (mu_pos : 0 < mu) (area_pos : 0 < area) (distance_pos : 0 < delta)
    (nonzero_rotation : omega ≠ 0) :
    0 < mu * area * delta * omega^2 / 2 := by
  exact div_pos (mul_pos (mul_pos (mul_pos mu_pos area_pos) distance_pos)
    (sq_pos_of_ne_zero nonzero_rotation)) (by norm_num)

/-- Restoring the symmetric virtual normal derivative changes rotational reaction. -/
theorem restored_shear_dipole (mu area delta shear : ℝ) :
    mu * area * delta * shear = delta * (mu * area * shear) := by ring

end Rheon.CommonGreenQ1

#print axioms Rheon.CommonGreenQ1.pairing_transpose
#print axioms Rheon.CommonGreenQ1.gram_energy_nonnegative
#print axioms Rheon.CommonGreenQ1.matched_transpose_work
#print axioms Rheon.CommonGreenQ1.rigid_kernel_work
#print axioms Rheon.CommonGreenQ1.green_reaction_equals_wall_iff
#print axioms Rheon.CommonGreenQ1.scalar_ray_rotation_positive
#print axioms Rheon.CommonGreenQ1.restored_shear_dipole
