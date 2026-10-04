import Mathlib.Data.Real.Basic
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Fin
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.NormNum
import Mathlib.Tactic.Ring

namespace Rheon.Transport
noncomputable section

def blend (t a b : ℝ) : ℝ := (1 - t) * a + t * b

theorem blend_lower (t a b lo : ℝ) (ht0 : 0 ≤ t) (ht1 : t ≤ 1)
    (ha : lo ≤ a) (hb : lo ≤ b) : lo ≤ blend t a b := by
  unfold blend
  nlinarith [mul_nonneg (sub_nonneg.mpr ht1) (sub_nonneg.mpr ha),
    mul_nonneg ht0 (sub_nonneg.mpr hb)]

theorem blend_upper (t a b hi : ℝ) (ht0 : 0 ≤ t) (ht1 : t ≤ 1)
    (ha : a ≤ hi) (hb : b ≤ hi) : blend t a b ≤ hi := by
  unfold blend
  nlinarith [mul_nonneg (sub_nonneg.mpr ht1) (sub_nonneg.mpr ha),
    mul_nonneg ht0 (sub_nonneg.mpr hb)]

theorem blend_constant (t a : ℝ) : blend t a a = a := by
  unfold blend
  ring

theorem convex_sum_bounds {n : ℕ} (w q : Fin n → ℝ) (lo hi : ℝ)
    (hw : ∀ i, 0 ≤ w i) (hs : (∑ i, w i) = 1)
    (hlo : ∀ i, lo ≤ q i) (hhi : ∀ i, q i ≤ hi) :
    lo ≤ ∑ i, w i * q i ∧ (∑ i, w i * q i) ≤ hi := by
  constructor
  · calc
      lo = ∑ i, w i * lo := by rw [← Finset.sum_mul, hs]; ring
      _ ≤ ∑ i, w i * q i := Finset.sum_le_sum (fun i _ => mul_le_mul_of_nonneg_left (hlo i) (hw i))
  · calc
      (∑ i, w i * q i) ≤ ∑ i, w i * hi := Finset.sum_le_sum (fun i _ => mul_le_mul_of_nonneg_left (hhi i) (hw i))
      _ = hi := by rw [← Finset.sum_mul, hs]; ring

theorem upwind_positive (c qleft q : ℝ) (hc0 : 0 ≤ c) (hc1 : c ≤ 1)
    (hl : 0 ≤ qleft) (hq : 0 ≤ q) : 0 ≤ q - c * (q - qleft) := by
  have h := blend_lower c q qleft 0 hc0 hc1 hq hl
  unfold blend at h
  nlinarith

theorem explicit_diffusion_positive (r qm q qp : ℝ) (hr0 : 0 ≤ r)
    (hr1 : r ≤ 1 / 2) (hm : 0 ≤ qm) (hq : 0 ≤ q) (hp : 0 ≤ qp) :
    0 ≤ q + r * (qm - 2 * q + qp) := by
  nlinarith [mul_nonneg hr0 hm, mul_nonneg hr0 hp,
    mul_nonneg (show 0 ≤ 1 - 2*r by linarith) hq]

theorem cfl_counterexample : (1 : ℚ) - (3/2) * (1 - 0) = -1/2 := by norm_num

theorem interpolation_not_mass_conservative :
    ((1 : ℚ) / 2 * 1 + 1 / 2 * 0) + (1 / 2 * 1 + 1 / 2 * 0) +
      (1 / 2 * 1 + 1 / 2 * 0) ≠ 1 + 0 + 0 := by norm_num

end
end Rheon.Transport
