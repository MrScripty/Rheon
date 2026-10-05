import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith

/-! Conditional exact-real obstruction identities. These do not establish
continuum conservation, a strain basis, geometry assembly, Rust or IEEE behavior. -/
namespace Rheon.MovingLiquid
noncomputable section

theorem sealed_flat_height_fixed (rho area old new : ℝ)
    (hr : 0 < rho) (ha : 0 < area)
    (mass : rho * area * old = rho * area * new) : old = new := by
  exact mul_left_cancel₀ (ne_of_gt (mul_pos hr ha)) mass

theorem sealed_flat_rate_zero (area rate : ℝ) (ha : 0 < area)
    (balance : area * rate = 0) : rate = 0 := by
  exact (mul_eq_zero.mp balance).resolve_left (ne_of_gt ha)

theorem unequal_rates_leave_flat (height dt a b : ℝ)
    (ht : 0 < dt) (hne : a ≠ b) : height + dt*a ≠ height + dt*b := by
  intro equality
  apply hne
  nlinarith

theorem balanced_exchange_volume (area height dt rate : ℝ) :
    area*(height+dt*rate)+area*(height-dt*rate)=2*area*height := by ring

theorem atmospheric_normal_traction_requires_zero_strain (mu strain : ℝ)
    (hm : 0 < mu) (traction : 2*mu*strain=0) : strain=0 := by
  have h : (2*mu)*strain=0 := by nlinarith [traction]
  exact (mul_eq_zero.mp h).resolve_left (ne_of_gt (by positivity))

theorem omitted_nonzero_normal_energy (mass speed : ℝ)
    (hm : 0 < mass) (hu : speed ≠ 0) : 0 < mass*speed^2/2 := by
  have hs : 0 < speed^2 := sq_pos_of_ne_zero hu
  positivity

theorem normal_top_mass_basis_difference (rho area omega h : ℝ) :
    rho*area*(omega-h/2)-rho*area*omega/2=rho*area*(omega-h)/2 := by ring

end
end Rheon.MovingLiquid
