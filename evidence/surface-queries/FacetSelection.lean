import Rheon.BoundedPhysics

/-! Exact finite-facet and candidate-selection contracts. Complete/correct
candidate enumeration is an explicit remaining obligation, not proved here.
The model has no inside/outside, moving wall, cut-cell or IEEE assertion. -/
namespace Rheon.SurfaceQueries
noncomputable section
open scoped BigOperators

def barycentricPoint (a b c : Fin 3 → ℝ) (u v : ℝ) (i : Fin 3) : ℝ :=
  (1-u-v)*a i + u*b i + v*c i

def facetCoordinates (u v : ℝ) : Prop := 0 ≤ u ∧ 0 ≤ v ∧ u+v ≤ 1

theorem barycentric_weights (u v : ℝ) (h : facetCoordinates u v) :
    0 ≤ 1-u-v ∧ 0 ≤ u ∧ 0 ≤ v ∧ (1-u-v)+u+v=1 := by
  rcases h with ⟨hu,hv,hs⟩
  constructor
  · linarith
  · exact ⟨hu,hv,by ring⟩

theorem barycentric_plane (normal : Fin 3 → ℝ) (offset : ℝ)
    (a b c : Fin 3 → ℝ) (u v : ℝ)
    (ha : Rheon.BoundedPhysics.wallValue normal offset a = 0)
    (hb : Rheon.BoundedPhysics.wallValue normal offset b = 0)
    (hc : Rheon.BoundedPhysics.wallValue normal offset c = 0) :
    Rheon.BoundedPhysics.wallValue normal offset (barycentricPoint a b c u v) = 0 := by
  have affine : Rheon.BoundedPhysics.wallValue normal offset (barycentricPoint a b c u v) =
      (1-u-v)*Rheon.BoundedPhysics.wallValue normal offset a +
      u*Rheon.BoundedPhysics.wallValue normal offset b +
      v*Rheon.BoundedPhysics.wallValue normal offset c := by
    unfold Rheon.BoundedPhysics.wallValue barycentricPoint
    simp only [mul_add, Finset.sum_add_distrib]
    have distribute (weight : ℝ) (point : Fin 3 → ℝ) :
        (∑ i, normal i * (weight * point i)) = weight * ∑ i, normal i * point i := by
      rw [Finset.mul_sum]
      apply Finset.sum_congr rfl
      intro i _
      ring
    rw [distribute,distribute,distribute]
    ring
  rw [affine,ha,hb,hc]
  ring

def selectEarliest (best : ℝ) : List ℝ → ℝ
  | [] => best
  | t::ts => selectEarliest (min best t) ts

theorem select_le_initial (best : ℝ) (hits : List ℝ) : selectEarliest best hits ≤ best := by
  induction hits generalizing best with
  | nil => exact le_refl _
  | cons t ts ih => exact le_trans (ih (min best t)) (min_le_left _ _)

theorem select_le_candidate (best : ℝ) (hits : List ℝ) (t : ℝ) (ht : t ∈ hits) :
    selectEarliest best hits ≤ t := by
  induction hits generalizing best with
  | nil => simp at ht
  | cons a rest ih =>
    rcases List.mem_cons.mp ht with h | h
    · subst t
      exact le_trans (select_le_initial (min best a) rest) (min_le_right _ _)
    · exact ih (min best a) h

theorem selected_is_candidate (best : ℝ) (hits : List ℝ) :
    selectEarliest best hits = best ∨ selectEarliest best hits ∈ hits := by
  induction hits generalizing best with
  | nil => exact Or.inl rfl
  | cons t ts ih =>
    rcases ih (min best t) with h | h
    · rw [selectEarliest,h]
      rcases le_total best t with hbt | htb
      · exact Or.inl (min_eq_left hbt)
      · exact Or.inr (by rw [min_eq_right htb]; exact List.mem_cons_self)
    · exact Or.inr (List.mem_cons_of_mem t h)

theorem select_in_segment (best : ℝ) (hits : List ℝ)
    (hb : 0 ≤ best ∧ best ≤ 1) (hh : ∀ t ∈ hits, 0 ≤ t ∧ t ≤ 1) :
    0 ≤ selectEarliest best hits ∧ selectEarliest best hits ≤ 1 := by
  rcases selected_is_candidate best hits with h | h
  · rw [h]; exact hb
  · exact hh _ h

end
end Rheon.SurfaceQueries
