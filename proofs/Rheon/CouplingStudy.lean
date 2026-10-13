import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.NormNum
namespace Rheon.CouplingStudy
noncomputable section
-- Finite affine sample geometry, not arbitrary mesh reconstruction.
theorem affine_transverse_sample (b t : ℝ) (ht : t ≠ 0) : b*t/t=b := by field_simp

theorem affine_longitudinal_sample (a b s : ℝ) : a+b*s-s*b=a := by ring

theorem nonorthogonal_correction (a b s : ℝ) (h : 1+s*s ≠ 0) :
 (a+b*s)/(1+s*s)+a*(1-1/(1+s*s))-b*s/(1+s*s)=a := by field_simp; ring

theorem skew_face_correction (b s c : ℝ) : c+b*s/2-b*s/2=c := by ring

-- Explicit recurrence of the stated two-path reduced momentum model.
theorem conservative_corrector (u v : ℝ) :
 (u-(u+v)/2)+(v-(u+v)/2)=0 := by ring

theorem velocity_error_recurrence (u a : ℝ) :
 ((1-a)*u+a*(1-2*u)/4)-1/6=(1-3*a/2)*(u-1/6) := by ring

theorem pressure_error_recurrence (p a : ℝ) :
 (p+a*(1/2-p))-1/2=(1-a)*(p-1/2) := by ring

theorem bounded_velocity_factor (a : ℝ) (ha : 0 ≤ a) (hb : a ≤ 4/3) :
 -1 ≤ 1-3*a/2 ∧ 1-3*a/2 ≤ 1 := by constructor <;> linarith

theorem exact_network_solution :
 (2*(1/6:ℝ)-(-1/6)+1/2=1) ∧ (2*(-1/6:ℝ)-1/6+1/2=0) := by norm_num

theorem overrelaxed_velocity_factor : (1-3*(3/2:ℝ)/2)^2=25/16 := by norm_num
end
end Rheon.CouplingStudy
