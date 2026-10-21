import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.NormNum

/-! Exact two-cell component contracts. Positive physical weights/volumes are
specified in the lesson. These identities do not prove an arbitrary mesh's
nullspace, a PDE solution, floating-point refinement, or a solver's runtime. -/
namespace Rheon.PressureGauge
noncomputable section

theorem component_sum_zero (w p q : ℝ) : w*(p-q)+w*(q-p)=0 := by ring

theorem pair_energy (w p q : ℝ) :
    p*(w*(p-q))+q*(w*(q-p))=w*(q-p)^2 := by ring

theorem component_compatibility (w p q l r : ℝ)
    (hl : w*(p-q)=l) (hr : w*(q-p)=r) : l+r=0 := by
  rw [← hl, ← hr]; ring

theorem independent_gauge_jumps (p q s t a b : ℝ) :
    (q+a)-(p+a)=q-p ∧ (t+b)-(s+b)=t-s := by constructor <;> ring

theorem fixed_cell_residual (w l r : ℝ) (hw : w ≠ 0) :
    l-w*(0-r/w)=l+r ∧ r-w*(r/w-0)=0 := by
  constructor <;> field_simp <;> ring

theorem fixed_cell_solves_compatible (w l r : ℝ) (hw : w ≠ 0)
    (compatible : l+r=0) :
    l-w*(0-r/w)=0 ∧ r-w*(r/w-0)=0 := by
  have h := fixed_cell_residual w l r hw
  exact ⟨h.1.trans compatible, h.2⟩

theorem opposite_component_imbalances :
    (1:ℝ)+0+(-1)+0=0 ∧ (1:ℝ)+0≠0 ∧ (-1:ℝ)+0≠0 := by norm_num

theorem arithmetic_mean_zero (p q : ℝ) :
    (p-(p+q)/2)+(q-(p+q)/2)=0 := by ring

theorem volume_mean_zero (v z p q : ℝ) (hv : v+z ≠ 0) :
    v*(p-(v*p+z*q)/(v+z))+z*(q-(v*p+z*q)/(v+z))=0 := by
  field_simp; ring

theorem projected_component_sum (l r : ℝ) :
    (l-(l+r)/2)+(r-(l+r)/2)=0 := by ring

theorem projected_original_residual (l r : ℝ) :
    l-(l-(l+r)/2)=(l+r)/2 ∧ r-(r-(l+r)/2)=(l+r)/2 := by
  constructor <;> ring

theorem volume_weighted_left_null (v z w p q : ℝ) (hv : v ≠ 0) (hz : z ≠ 0) :
    v*(w*(p-q)/v)+z*(w*(q-p)/z)=0 := by field_simp; ring

theorem boundary_work_shift (p q a qL qR : ℝ) :
    (p+a)*qL+(q+a)*qR-(p*qL+q*qR)=a*(qL+qR) := by ring

theorem balanced_boundary_work (p q a flux : ℝ) :
    (p+a)*flux+(q+a)*(-flux)=p*flux+q*(-flux) := by ring

theorem pressure_pairing_with_boundary (p q flux qL qR : ℝ) :
    p*(flux+qL)+q*(-flux+qR)=-(q-p)*flux+p*qL+q*qR := by ring

end
end Rheon.PressureGauge
