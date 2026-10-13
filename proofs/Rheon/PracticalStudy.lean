import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.NormNum
import Mathlib.Tactic.Positivity
import Mathlib.Tactic.LinearCombination
namespace Rheon.PracticalStudy
noncomputable section
-- Unit gap/area, steady unidirectional Newtonian solution with body force G.
theorem wall_force_balance (mu U G : ℝ) :
 (-mu*U-G/2)+(mu*U-G/2)+G=0 := by ring

theorem wall_power_balance (mu U G : ℝ) (h : mu ≠ 0) :
 U*(mu*U-G/2)+G*(U/2+G/(12*mu))=mu*U^2+G^2/(12*mu) := by
 field_simp
 <;> ring

theorem wall_dissipation_nonnegative (mu U G : ℝ) (h : 0 < mu) :
 0 ≤ mu*U^2+G^2/(12*mu) := by
 exact add_nonneg (mul_nonneg (le_of_lt h) (sq_nonneg U))
   (div_nonneg (sq_nonneg G) (by positivity))

-- Fixed two-unknown Dirichlet matrix K = [[2,-1],[-1,2]].
theorem matrix_energy_identity (x y : ℝ) :
 2*x^2-2*x*y+2*y^2=(x-y)^2+x^2+y^2 := by ring

theorem matrix_energy_positive (x y : ℝ) (h : x ≠ 0 ∨ y ≠ 0) :
 0 < 2*x^2-2*x*y+2*y^2 := by
 rcases h with hx | hy
 · nlinarith [sq_nonneg (x-y), sq_nonneg y, sq_pos_of_ne_zero hx]
 · nlinarith [sq_nonneg (x-y), sq_nonneg x, sq_pos_of_ne_zero hy]

theorem jacobi_smooth_error (x y a : ℝ) :
 (((1-a)*x+a*y/2)+((1-a)*y+a*x/2))/2=(1-a/2)*((x+y)/2) := by ring

theorem jacobi_alternating_error (x y a : ℝ) :
 (((1-a)*x+a*y/2)-((1-a)*y+a*x/2))/2=(1-3*a/2)*((x-y)/2) := by ring

theorem jacobi_factors_strict (a : ℝ) (ha : 0 < a) (hb : a < 4/3) :
 |1-a/2| < 1 ∧ |1-3*a/2| < 1 := by
 constructor <;> apply abs_lt.mpr <;> constructor <;> linarith

theorem jacobi_unstable_mode : (1-3*(3/2:ℝ)/2)^2=25/16 := by norm_num

-- Exact-real arithmetic does not establish a binary32 implementation.
theorem exact_reduction_order (A : ℝ) : (A+1)-A=(A-A)+1 := by ring

-- Fixed masses and one sample. The premise is the mass-weighted grid mean.
theorem transfer_momentum (m v w u du a : ℝ) (h : (1+m)*u=v+m*w) :
 ((1-a)*(u+du)+a*(v+du))+m*((1-a)*(u+du)+a*(w+du))=
 v+m*w+(1+m)*du := by
 linear_combination (1-a)*h

theorem transfer_variance_scale (m x y a : ℝ) :
 (a*x)^2+m*(a*y)^2=a^2*(x^2+m*y^2) := by ring

theorem transfer_variance_nonincrease (V a : ℝ) (hV : 0 ≤ V)
 (ha : 0 ≤ a) (hb : a ≤ 1) : a^2*V ≤ V := by
 have hfactor : 0 ≤ 1-a*a := by nlinarith
 nlinarith [mul_nonneg hfactor hV]

-- Supplied-field constitutive probe; no transport/model-validation theorem.
theorem sst_denominator_positive (a w s f : ℝ) (h : 0 < a*w) :
 0 < max (a*w) (s*f) := lt_of_lt_of_le h (le_max_left _ _)

theorem sst_viscosity_nonnegative (rho a k d : ℝ)
 (hr : 0 ≤ rho) (ha : 0 ≤ a) (hk : 0 ≤ k) (hd : 0 ≤ d) :
 0 ≤ rho*a*k/d := div_nonneg (mul_nonneg (mul_nonneg hr ha) hk) hd

theorem sst_production_bounds (P cap : ℝ) (hP : 0 ≤ P) (hc : 0 ≤ cap) :
 0 ≤ min P cap ∧ min P cap ≤ P ∧ min P cap ≤ cap := by
 exact ⟨le_min hP hc, min_le_left _ _, min_le_right _ _⟩

theorem sst_laminar_viscosity (rho a d : ℝ) : rho*a*0/d=0 := by simp

-- Richardson algebra under an explicitly supplied quadratic error model.
theorem quadratic_extrapolation (q c h : ℝ) :
 (q-c*h^2/4)+((q-c*h^2/4)-(q-c*h^2))/3=q := by ring

theorem constant_bias_survives_refinement (q b c h : ℝ) :
 (q+b-c*h^2/4)+((q+b-c*h^2/4)-(q+b-c*h^2))/3=q+b := by ring

-- Two-cell full residual and gauge invariance, with an explicit tolerance.
theorem candidate_full_residual (e : ℝ) :
 1-(1-e)=e ∧ -1+(1-e)=-e := by constructor <;> ring

theorem candidate_gauge_flux (s e : ℝ) : s-(s-1+e)=1-e := by ring

theorem candidate_divergence_bound (e t : ℝ) (h : |e| ≤ t) :
 |-e| ≤ t ∧ |e| ≤ t := by simpa only [abs_neg] using And.intro h h

theorem positive_tolerance_is_not_zero :
 |(1/16:ℝ)| ≤ 1/8 ∧ (1/16:ℝ) ≠ 0 := by norm_num
end
end Rheon.PracticalStudy
