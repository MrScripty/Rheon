import Mathlib.Algebra.Order.BigOperators.Ring.Finset
import Mathlib.Data.Real.Basic
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith

/-! Exact-real fixed-pose impulse algebra. Physical input properties, matched
COM/reference, force quadrature, and floating endpoints are not inferred. -/
namespace Rheon.RigidImpulse
noncomputable section
def kick (c v j : ℝ) : ℝ := v+j/c
def kinetic (c v : ℝ) : ℝ := c*v^2/2
def midpoint (v v' : ℝ) : ℝ := (v+v')/2
def residual (c v v' j : ℝ) : ℝ := c*(v'-v)-j

theorem component_momentum (c v j : ℝ) (hc : c≠0) :
    c*(kick c v j-v)=j := by
  unfold kick
  field_simp
  ring

/-- Holds for arbitrary actual endpoints: rounding residuals are explicit. -/
theorem component_residual_work (c v v' j : ℝ) :
    kinetic c v'-kinetic c v =
      j*midpoint v v'+residual c v v' j*midpoint v v' := by
  unfold kinetic midpoint residual
  ring

theorem component_impulse_work (c v j : ℝ) (hc : c≠0) :
    kinetic c (kick c v j)-kinetic c v = j*midpoint v (kick c v j) := by
  have h := component_residual_work c v (kick c v j) j
  have hm := component_momentum c v j hc
  simp only [residual, hm, sub_self, zero_mul, add_zero] at h
  exact h

/-- Summing these six components gives mass and diagonal-inertia rigid work.
No orientation evolution, gyroscopic integration, or IEEE identity is proved. -/
theorem six_component_impulse_work (c v j : Fin 6 → ℝ) (hc : ∀i, c i≠0) :
    (∑ i, (kinetic (c i) (kick (c i) (v i) (j i))-kinetic (c i) (v i))) =
    ∑ i, j i*midpoint (v i) (kick (c i) (v i) (j i)) := by
  apply Finset.sum_congr rfl
  intro i _
  exact component_impulse_work (c i) (v i) (j i) (hc i)

theorem six_component_residual_work (c v v' j : Fin 6 → ℝ) :
    (∑ i, (kinetic (c i) (v' i)-kinetic (c i) (v i))) =
    ∑ i, (j i*midpoint (v i) (v' i)+
      residual (c i) (v i) (v' i) (j i)*midpoint (v i) (v' i)) := by
  apply Finset.sum_congr rfl
  intro i _
  exact component_residual_work (c i) (v i) (v' i) (j i)

theorem component_kinetic_nonnegative (c v : ℝ) (hc : 0≤c) :
    0≤kinetic c v := by
  unfold kinetic
  exact div_nonneg (mul_nonneg hc (sq_nonneg v)) (by norm_num)

/-- Physical principal moments derived from nonnegative second moments.
Positive definiteness alone does not imply these inequalities. -/
theorem inertia_triangle_inequalities (sx sy sz : ℝ)
    (hx : 0≤sx) (hy : 0≤sy) (hz : 0≤sz) :
    sy+sz≤(sx+sz)+(sx+sy) ∧ sx+sz≤(sy+sz)+(sx+sy) ∧
      sx+sy≤(sy+sz)+(sx+sz) := by
  constructor
  · linarith
  constructor <;> linarith
end
end Rheon.RigidImpulse
