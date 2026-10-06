import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith

/-! Conditional exact real identities for supplied column geometry and fluxes.
No Rust/IEEE refinement, geometry topology, multidimensional bounds or convergence
proof is supplied. -/
namespace Rheon.ColumnSurface
noncomputable section

theorem ghost_interpolation_zero (theta p : ℝ) (ht : theta ≠ 0) :
    (1-theta)*p + theta*(-(1-theta)/theta*p) = 0 := by
  field_simp
  all_goals ring

theorem ghost_gradient_reduction (theta p h : ℝ) (ht : theta ≠ 0) (hh : h ≠ 0) :
    (-(1-theta)/theta*p-p)/h = -p/(theta*h) := by
  field_simp
  all_goals ring

theorem height_crossing_zero (wet air : ℝ) (hd : air-wet ≠ 0) :
    (1-(-wet)/(air-wet))*wet + ((-wet)/(air-wet))*air = 0 := by
  field_simp
  all_goals ring

theorem variable_distance_positive (area density h theta : ℝ)
    (ha : 0 < area) (hrho : 0 < density) (hh : 0 < h) (ht : 0 < theta) :
    0 < area/(density*h*theta) :=
  div_pos ha (mul_pos (mul_pos hrho hh) ht)

theorem column_reconstruction_volume (volume full fraction : ℝ) :
    volume*(full+fraction) = volume*full+volume*fraction := by ring

theorem column_update_cancels_internal_axial_transfers
    (a b c bottom ab bc top ia ib ic oa ob oc : ℝ) :
    (a+bottom-ab+ia-oa)+(b+ab-bc+ib-ob)+(c+bc-top+ic-oc) =
    a+b+c+bottom-top+(ia+ib+ic)-(oa+ob+oc) := by ring

theorem shared_column_transfer_balance (left right transfer : ℝ) :
    (left-transfer)+(right+transfer) = left+right := by ring

theorem upward_swept_slab_bound (fraction courant : ℝ)
    (hf0 : 0 ≤ fraction) (hf1 : fraction ≤ 1)
    (hc0 : 0 ≤ courant) (hc1 : courant ≤ 1) :
    0 ≤ max (fraction+courant-1) 0 ∧
    max (fraction+courant-1) 0 ≤ min fraction courant := by
  constructor
  · exact le_max_right _ _
  · apply le_min
    · apply max_le
      · linarith
      · exact hf0
    · apply max_le
      · linarith
      · exact hc0

theorem downward_swept_slab_bound (fraction courant : ℝ)
    (hf0 : 0 ≤ fraction) (hc0 : 0 ≤ courant) :
    0 ≤ min fraction courant ∧ min fraction courant ≤ fraction ∧
    min fraction courant ≤ courant :=
  ⟨le_min hf0 hc0, min_le_left _ _, min_le_right _ _⟩

theorem transverse_donor_convex_bound (left right courant : ℝ)
    (hl0 : 0 ≤ left) (hl1 : left ≤ 1) (hr0 : 0 ≤ right)
    (hr1 : right ≤ 1) (hc0 : 0 ≤ courant) (hc1 : courant ≤ 1) :
    0 ≤ (1-courant)*left+courant*right ∧
    (1-courant)*left+courant*right ≤ 1 := by
  constructor
  · exact add_nonneg (mul_nonneg (sub_nonneg.mpr hc1) hl0) (mul_nonneg hc0 hr0)
  · nlinarith [mul_nonneg (sub_nonneg.mpr hc1) (sub_nonneg.mpr hl1),
      mul_nonneg hc0 (sub_nonneg.mpr hr1)]

end
end Rheon.ColumnSurface
