import Mathlib.Data.Real.Basic
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Fin
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Ring

/- Exact-real diagnostic contracts. Taylor remainder, regularity, continuum
   incompressibility and sample accuracy are PREMISES, never derived from MAC
   samples. No PDE, IEEE, coupling or step-authorization theorem is asserted. -/
namespace Rheon.SampleSupportedWallTrace
open scoped BigOperators

/-- A stated Taylor remainder and sample bounds imply the secant error bound. -/
theorem secant_error (delta u b U B derivative remainder etaU etaB : ℝ)
    (positive_distance : 0 < delta)
    (taylor_remainder : |U - B - delta * derivative| ≤ remainder)
    (sample_error : |u - U| ≤ etaU) (trace_error : |b - B| ≤ etaB) :
    |(u-b)/delta-derivative| ≤ (etaU+etaB+remainder)/delta := by
  have algebra : (u-b)/delta-derivative =
      ((u-U)-(b-B)+(U-B-delta*derivative))/delta := by
    field_simp
    ring
  have triangle : |(u-U)-(b-B)+(U-B-delta*derivative)| ≤
      etaU+etaB+remainder := by
    calc
      _ ≤ |(u-U)-(b-B)| + |U-B-delta*derivative| := abs_add_le _ _
      _ ≤ (|u-U| + |b-B|) + |U-B-delta*derivative| :=
        add_le_add_right (by simpa only [sub_eq_add_neg, abs_neg] using abs_add_le (u-U) (-(b-B))) _
      _ ≤ etaU+etaB+remainder := by linarith
  rw [algebra, abs_div, abs_of_pos positive_distance]
  exact div_le_div_of_nonneg_right triangle (le_of_lt positive_distance)

/-- The first ray rule exactly differentiates an affine profile with its trace. -/
theorem affine_secant (delta trace slope : ℝ) (positive_distance : 0 < delta) :
    ((trace+slope*delta)-trace)/delta = slope := by
  field_simp

/-- Positive basis integrals propagate componentwise stated traction errors. -/
theorem positive_weighted_error {n : ℕ} (area a b error : Fin n → ℝ)
    (positive_area : ∀ k, 0 ≤ area k)
    (pointwise_error : ∀ k, |a k-b k| ≤ error k) :
    |∑ k, area k*(a k-b k)| ≤ ∑ k, area k*error k := by
  calc
    _ ≤ ∑ k, |area k*(a k-b k)| := Finset.abs_sum_le_sum_abs _ _
    _ = ∑ k, area k*|a k-b k| := by
      apply Finset.sum_congr rfl
      intro k _
      rw [abs_mul, abs_of_nonneg (positive_area k)]
    _ ≤ _ := Finset.sum_le_sum (fun k _ => mul_le_mul_of_nonneg_left (pointwise_error k) (positive_area k))

/-- Exact matrix transpose work for any consistent surface moment assembly. -/
theorem surface_transpose_work {n k : ℕ}
    (H : Fin n → Fin k → ℝ) (traction : Fin n → ℝ) (rigid : Fin k → ℝ) :
    (∑ i, (∑ j, H i j*rigid j)*traction i) =
      ∑ j, rigid j*(∑ i, H i j*traction i) := by
  simp_rw [Finset.sum_mul, Finset.mul_sum]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro j _
  apply Finset.sum_congr rfl
  intro i _
  ring

/-- A changed observable's work gap is its load difference paired with the test.
    This identity does not identify the difference as a volume residual. -/
theorem observable_work_gap {n : ℕ} (candidate old rigid : Fin n → ℝ) :
    (∑ k, rigid k*candidate k) - (∑ k, rigid k*old k) =
      ∑ k, rigid k*(candidate k-old k) := by
  rw [← Finset.sum_sub_distrib]
  apply Finset.sum_congr rfl
  intro k _
  ring

end Rheon.SampleSupportedWallTrace

#print axioms Rheon.SampleSupportedWallTrace.secant_error
#print axioms Rheon.SampleSupportedWallTrace.affine_secant
#print axioms Rheon.SampleSupportedWallTrace.positive_weighted_error
#print axioms Rheon.SampleSupportedWallTrace.surface_transpose_work
#print axioms Rheon.SampleSupportedWallTrace.observable_work_gap
