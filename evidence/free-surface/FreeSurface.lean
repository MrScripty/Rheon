import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith

/-! Exact real two-wet-cell / prescribed-air identities. Geometry, coefficients,
solves and phase snapshots are supplied. No Rust/IEEE or physical validation. -/
namespace Rheon.FreeSurface
noncomputable section

def energy (w a b p q : ℝ) : ℝ := w*(p-q)^2 + a*p^2 + b*q^2
def row (w a p q : ℝ) : ℝ := w*(p-q) + a*p

theorem eliminated_pressure_quadratic (w a b p q : ℝ) :
    p*row w a p q + q*row w b q p = energy w a b p q := by
  unfold row energy
  ring

theorem anchored_energy_nonnegative (w a b p q : ℝ)
    (hw : 0 ≤ w) (ha : 0 ≤ a) (hb : 0 ≤ b) : 0 ≤ energy w a b p q := by
  unfold energy
  exact add_nonneg (add_nonneg (mul_nonneg hw (sq_nonneg (p-q)))
    (mul_nonneg ha (sq_nonneg p))) (mul_nonneg hb (sq_nonneg q))

theorem two_cell_anchor_removes_constant (w b p q : ℝ)
    (hw : 0 < w) (hb : 0 < b) (zero : energy w 0 b p q = 0) :
    p = 0 ∧ q = 0 := by
  have hdiff : 0 ≤ w*(p-q)^2 := mul_nonneg (le_of_lt hw) (sq_nonneg (p-q))
  have hq : 0 ≤ b*q^2 := mul_nonneg (le_of_lt hb) (sq_nonneg q)
  have hqzero : q^2 = 0 := by
    unfold energy at zero
    nlinarith [sq_nonneg q]
  have hq0 : q = 0 := by nlinarith [sq_nonneg q]
  subst q
  unfold energy at zero
  have hpzero : p^2 = 0 := by nlinarith [sq_nonneg p]
  constructor
  · nlinarith [sq_nonneg p]
  · rfl

theorem half_distance_coefficient (area density h : ℝ)
    (hrho : density ≠ 0) (hh : h ≠ 0) :
    area/(density*(h/2)) = 2*(area/(density*h)) := by
  field_simp
  all_goals ring

theorem wet_residual_correction (w a p q u surface dt : ℝ) (ht : dt ≠ 0) :
    (u-dt*w*(q-p)) + (surface+dt*a*p) =
      -dt*((-u-surface)/dt-row w a p q) := by
  unfold row
  field_simp
  all_goals ring

theorem hydrostatic_surface_cancellation (density g distance dt : ℝ)
    (hrho : density ≠ 0) (hd : distance ≠ 0) :
    -dt*g-dt*((0-density*g*distance)/(density*distance)) = 0 := by
  field_simp
  all_goals ring

theorem shared_surface_transfer_balance (wet air transfer : ℝ) :
    (wet-transfer)+(air+transfer) = wet+air := by
  ring

end
end Rheon.FreeSurface
