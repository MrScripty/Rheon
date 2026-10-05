import Rheon.BoundedPhysics

/-! Exact affine-translation algebra over one prescribed interval. These lemmas
are conditional on the supplied real trajectories/candidates and do not certify
IEEE arithmetic, triangle enumeration, rotation or any fluid boundary update. -/
namespace Rheon.TranslationFrame
noncomputable section
open scoped BigOperators
open Rheon.BoundedPhysics

def relativePoint (point offset : Fin 3 → ℝ) (i : Fin 3) : ℝ := point i - offset i

def worldPoint (point offset : Fin 3 → ℝ) (i : Fin 3) : ℝ := point i + offset i

def clock (start finish parameter : ℝ) : ℝ := (1-parameter)*start+parameter*finish

def wallVelocity (start finish : ℝ) (a b : Fin 3 → ℝ) (i : Fin 3) : ℝ :=
  (b i-a i)/(finish-start)

theorem relative_segment (a b offsetA offsetB : Fin 3 → ℝ) (t : ℝ) :
    relativePoint (segment a b t) (segment offsetA offsetB t) =
      segment (relativePoint a offsetA) (relativePoint b offsetB) t := by
  funext i
  unfold relativePoint segment
  ring

theorem world_relative (point offset : Fin 3 → ℝ) :
    worldPoint (relativePoint point offset) offset = point := by
  funext i
  unfold worldPoint relativePoint
  ring

theorem reference_contact_world (point offset contact : Fin 3 → ℝ)
    (h : relativePoint point offset = contact) : worldPoint contact offset = point := by
  rw [← h]
  exact world_relative point offset

theorem translated_plane_value (normal point offset : Fin 3 → ℝ) (c : ℝ) :
    wallValue normal c (relativePoint point offset) =
      wallValue normal (c + ∑ i, normal i * offset i) point := by
  unfold wallValue relativePoint
  simp only [mul_sub, Finset.sum_sub_distrib]
  ring

theorem clock_order (start finish u v : ℝ)
    (ht : start < finish) (huv : u ≤ v) : clock start finish u ≤ clock start finish v := by
  unfold clock
  nlinarith

theorem clock_range (start finish t : ℝ)
    (ht : start < finish) (hp : 0 ≤ t ∧ t ≤ 1) :
    start ≤ clock start finish t ∧ clock start finish t ≤ finish := by
  rcases hp with ⟨h0,h1⟩
  have low := clock_order start finish 0 t ht h0
  have high := clock_order start finish t 1 ht h1
  simp only [clock, sub_zero, one_mul, zero_mul, add_zero,
    sub_self, zero_add] at low high
  exact ⟨low,high⟩

theorem velocity_displacement (start finish : ℝ) (a b : Fin 3 → ℝ)
    (ht : start < finish) (i : Fin 3) :
    wallVelocity start finish a b i * (finish-start) = b i-a i := by
  unfold wallVelocity
  have hd : finish-start ≠ 0 := by linarith
  exact div_mul_cancel₀ _ hd

end
end Rheon.TranslationFrame
