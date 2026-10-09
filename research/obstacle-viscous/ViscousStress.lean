import Mathlib.Data.Real.Basic
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Fin
import Mathlib.Tactic.Ring
import Mathlib.Tactic.FieldSimp

/- Finite exact-real algebra. E is the retained normal/shear row matrix,
   including homogeneous stationary trace elimination. Geometry completeness,
   exact reciprocals, continuum accuracy and binary64 are not premises proved here. -/
namespace Rheon.ObstacleViscous
open scoped BigOperators

/-- A normal block has weight 2V in E notation, stress 2 mu g. -/
theorem normal_stress_work (mu V g : ℝ) :
    V * g * (2 * mu * g) = 2 * (mu * V * g ^ 2) := by ring

/-- Equal paired weights and both stress copies are essential, including zero rows. -/
theorem shear_stress_work (mu w a b : ℝ) :
    w * a * (mu * (a+b)) + w * b * (mu * (a+b)) =
      2 * (mu / 2 * w * (a+b)^2) := by ring

/-- The linear variation coefficient agrees with the paired stress covector. -/
theorem shear_potential_variation (mu w a b p q t : ℝ) :
    mu / 2 * w * ((a+t*p)+(b+t*q))^2 =
      mu / 2 * w * (a+b)^2 +
      t * (w*p*(mu*(a+b)) + w*q*(mu*(a+b))) +
      t^2 * (mu / 2 * w * (p+q)^2) := by ring

/-- Combining paired directed transpose columns equals an engineering-shear row. -/
theorem shear_transpose (mu w ga gb ca cb : ℝ) :
    ca*w*(mu*(ga+gb)) + cb*w*(mu*(ga+gb)) =
      mu*(ca+cb)*w*(ga+gb) := by ring

/-- Exact directional expansion of the finite quadratic Rayleigh potential. -/
theorem finite_potential_variation {r : ℕ} (w g h : Fin r → ℝ) (mu t : ℝ) :
    mu/2 * (∑ i, w i * (g i+t*h i)^2) =
      mu/2 * (∑ i, w i*g i^2) + t*(∑ i, mu*w i*g i*h i) +
      t^2 * (mu/2 * (∑ i, w i*h i^2)) := by
  simp_rw [Finset.mul_sum]
  rw [← Finset.sum_add_distrib, ← Finset.sum_add_distrib]
  apply Finset.sum_congr rfl
  intro i _
  ring

/-- Matched negative transpose force has exactly negative dissipated power. -/
theorem force_work {r f : ℕ} (E : Fin r → Fin f → ℝ)
    (w : Fin r → ℝ) (u : Fin f → ℝ) (mu : ℝ) :
    (∑ j, u j * (-mu * ∑ i, E i j * w i * (∑ k, E i k * u k))) =
      -mu * ∑ i, w i * (∑ j, E i j * u j)^2 := by
  simp_rw [pow_two, Finset.mul_sum, Finset.sum_mul]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro i _
  apply Finset.sum_congr rfl
  intro j _
  simp_rw [Finset.mul_sum]
  apply Finset.sum_congr rfl
  intro k _
  ring

/-- Symmetry of the finite stiffness bilinear form, without geometry claims. -/
theorem stiffness_symmetry {r f : ℕ} (E : Fin r → Fin f → ℝ)
    (w : Fin r → ℝ) (u v : Fin f → ℝ) (mu : ℝ) :
    (∑ i, mu*w i*(∑ j, E i j*u j)*(∑ j, E i j*v j)) =
      ∑ i, mu*w i*(∑ j, E i j*v j)*(∑ j, E i j*u j) := by
  apply Finset.sum_congr rfl
  intro i _
  ring

/-- Positivity requires material and quadrature hypotheses; no coercivity. -/
theorem dissipation_nonnegative {r : ℕ} (w s : Fin r → ℝ) (mu : ℝ)
    (material : 0 ≤ mu) (weights : ∀ i, 0 ≤ w i) :
    0 ≤ mu * ∑ i, w i * s i ^ 2 := by
  exact mul_nonneg material (Finset.sum_nonneg (fun i _ => mul_nonneg (weights i) (sq_nonneg _)))

/-- Compatible local skew cross derivatives have zero shear, not a wall compatibility theorem. -/
theorem compatible_rotation (mu a b : ℝ) (skew : b = -a) :
    mu * (a+b) = 0 := by rw [skew]; ring

/-- A positive wall separation yields the partial ray work. No geometric area equality is used. -/
theorem flat_ray_work (mu w U delta : ℝ) :
    U * (-mu * w * U / delta^2) = -mu * w * (U/delta)^2 := by
  simp only [div_pow]
  ring
/-- Geometric area identification needs the extra exact product volume premise. -/
theorem flat_conductance_area (volume area delta : ℝ)
    (separation : delta ≠ 0) (product_volume : volume = area * delta) :
    volume / delta^2 = area / delta := by
  rw [product_volume]
  field_simp
  ring
end Rheon.ObstacleViscous

#print axioms Rheon.ObstacleViscous.normal_stress_work
#print axioms Rheon.ObstacleViscous.shear_stress_work
#print axioms Rheon.ObstacleViscous.shear_potential_variation
#print axioms Rheon.ObstacleViscous.shear_transpose
#print axioms Rheon.ObstacleViscous.finite_potential_variation
#print axioms Rheon.ObstacleViscous.force_work
#print axioms Rheon.ObstacleViscous.stiffness_symmetry
#print axioms Rheon.ObstacleViscous.dissipation_nonnegative
#print axioms Rheon.ObstacleViscous.compatible_rotation
#print axioms Rheon.ObstacleViscous.flat_ray_work
#print axioms Rheon.ObstacleViscous.flat_conductance_area
