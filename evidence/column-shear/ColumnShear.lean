import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith

/-! Conditional exact-real identities for the flat-column shear prerequisite.
No proof of arbitrary geometry, basis integration, pressure compatibility,
Rust/IEEE refinement, convergence or physical material calibration. -/
namespace Rheon.ColumnShear
noncomputable section

theorem dual_length_partition (n h height : ℝ) :
    n*h+(height-n*h) = height := by ring

theorem cap_mass_positive (rho area height n h : ℝ)
    (hr : 0 < rho) (ha : 0 < area) (hc : 0 < height-n*h) :
    0 < rho*area*(height-n*h) := mul_pos (mul_pos hr ha) hc

theorem row_sum_mass (rho area left right : ℝ) :
    rho*area*(left+right) = rho*area*left+rho*area*right := by ring

theorem edge_force_cancellation (k a b : ℝ) :
    k*(b-a) + (-k*(b-a)) = 0 := by ring

theorem three_node_force_cancellation (ka kb a b c : ℝ) :
    ka*(b-a) + (kb*(c-b)-ka*(b-a)) - kb*(c-b) = 0 := by ring

theorem three_node_dissipative_work (ka kb a b c : ℝ) :
    a*(ka*(b-a)) + b*(kb*(c-b)-ka*(b-a)) + c*(-kb*(c-b)) =
    -(ka*(b-a)^2+kb*(c-b)^2) := by ring

theorem dissipation_nonnegative (ka kb a b c : ℝ)
    (ha : 0 ≤ ka) (hb : 0 ≤ kb) :
    0 ≤ ka*(b-a)^2+kb*(c-b)^2 := by positivity

theorem translation_zero_force (ka kb u : ℝ) :
    ka*(u-u) = 0 ∧ kb*(u-u)-ka*(u-u) = 0 ∧ -kb*(u-u) = 0 := by
  constructor
  · ring
  constructor <;> ring

theorem coordinate_update_preserves_momentum
    (ma mb mc a b c va vb vc dt ka kb : ℝ)
    (ha : ma*(va-a) = dt*ka*(b-a))
    (hb : mb*(vb-b) = dt*(kb*(c-b)-ka*(b-a)))
    (hc : mc*(vc-c) = -dt*kb*(c-b)) :
    ma*va+mb*vb+mc*vc = ma*a+mb*b+mc*c := by
  nlinarith

theorem coordinate_update_old_work
    (ma mb mc a b c va vb vc dt ka kb : ℝ)
    (ha : ma*(va-a) = dt*ka*(b-a))
    (hb : mb*(vb-b) = dt*(kb*(c-b)-ka*(b-a)))
    (hc : mc*(vc-c) = -dt*kb*(c-b)) :
    ma*a*(va-a)+mb*b*(vb-b)+mc*c*(vc-c) =
      -dt*(ka*(b-a)^2+kb*(c-b)^2) := by
  calc
    _ = a*(ma*(va-a))+b*(mb*(vb-b))+c*(mc*(vc-c)) := by ring
    _ = a*(dt*ka*(b-a))+b*(dt*(kb*(c-b)-ka*(b-a)))+c*(-dt*kb*(c-b)) := by rw [ha,hb,hc]
    _ = _ := by ring

theorem weighted_energy_identity (ma mb mc a b c va vb vc : ℝ) :
    (ma*va^2+mb*vb^2+mc*vc^2)/2-(ma*a^2+mb*b^2+mc*c^2)/2 =
    ma*a*(va-a)+mb*b*(vb-b)+mc*c*(vc-c)+
      (ma*(va-a)^2+mb*(vb-b)^2+mc*(vc-c)^2)/2 := by ring

theorem energy_nonincrease_given_update_bound (before after dt dissip update : ℝ)
    (identity : after-before = -dt*dissip+update)
    (stability : update ≤ dt*dissip) : after ≤ before := by linarith

theorem rounded_momentum_identity (mass candidate rounding : ℝ) :
    mass*(candidate+rounding)-mass*candidate = mass*rounding := by ring

theorem zero_endpoint_traction_force (bottom top : ℝ)
    (hb : bottom = 0) (ht : top = 0) : top-bottom = 0 := by rw [hb,ht]; ring

theorem two_neighbor_convex_bounds (left center right rl rr : ℝ)
    (hl0 : 0 ≤ left) (hl1 : left ≤ 1) (hc0 : 0 ≤ center)
    (hc1 : center ≤ 1) (hr0 : 0 ≤ right) (hr1 : right ≤ 1)
    (hrl : 0 ≤ rl) (hrr : 0 ≤ rr) (hs : rl+rr ≤ 1) :
    0 ≤ rl*left+(1-rl-rr)*center+rr*right ∧
    rl*left+(1-rl-rr)*center+rr*right ≤ 1 := by
  have hm : 0 ≤ 1-rl-rr := by linarith
  constructor
  · exact add_nonneg (add_nonneg (mul_nonneg hrl hl0) (mul_nonneg hm hc0)) (mul_nonneg hrr hr0)
  · nlinarith [mul_nonneg hrl (sub_nonneg.mpr hl1),
      mul_nonneg hm (sub_nonneg.mpr hc1), mul_nonneg hrr (sub_nonneg.mpr hr1)]

end
end Rheon.ColumnShear
