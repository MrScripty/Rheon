import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.NormNum
import Mathlib.Tactic.Ring

/-! Exact unit-cell, one-face liquid-volume contracts. No implementation,
IEEE arithmetic, multidimensional VOF, dynamics or performance theorem. -/
namespace Rheon.VolumeFlux
noncomputable section

/-- Liquid occupies [0,f]; the positive-x swept slab is [1-c,1]. -/
def leftFlux (f c : ℝ) : ℝ := max 0 (f + c - 1)

/-- Liquid occupies [1-f,1]; same fraction, different placement. -/
def rightFlux (f c : ℝ) : ℝ := min f c

theorem left_flux_bounds (f c : ℝ) (hf0 : 0 ≤ f) (hf1 : f ≤ 1)
    (hc0 : 0 ≤ c) (hc1 : c ≤ 1) :
    0 ≤ leftFlux f c ∧ leftFlux f c ≤ f ∧ leftFlux f c ≤ c := by
  unfold leftFlux
  exact ⟨le_max_left _ _, max_le hf0 (by linarith), max_le hc0 (by linarith)⟩

theorem right_flux_bounds (f c : ℝ) (hf0 : 0 ≤ f) (hc0 : 0 ≤ c) :
    0 ≤ rightFlux f c ∧ rightFlux f c ≤ f ∧ rightFlux f c ≤ c := by
  exact ⟨le_min hf0 hc0, min_le_left _ _, min_le_right _ _⟩

/-- One identical oriented amount is subtracted and added; no source/boundary loss. -/
theorem shared_transfer_conserves (donor receiver q : ℝ) :
    (donor - q) + (receiver + q) = donor + receiver := by ring

/-- Availability and receiving capacity are separate obligations. -/
theorem shared_transfer_bounds (donor receiver q : ℝ)
    (hd : donor ≤ 1) (hr : 0 ≤ receiver) (hq : 0 ≤ q)
    (available : q ≤ donor) (capacity : receiver + q ≤ 1) :
    0 ≤ donor - q ∧ donor - q ≤ 1 ∧
    0 ≤ receiver + q ∧ receiver + q ≤ 1 := by
  constructor
  · linarith
  constructor
  · linarith
  constructor
  · linarith
  · exact capacity

theorem same_fraction_different_flux :
    leftFlux (3/10) (1/2) = 0 ∧ rightFlux (3/10) (1/2) = 3/10 := by
  norm_num [leftFlux, rightFlux]

/-- Full-slab transfer from 0.3 to an empty cell, then separately clamp. -/
theorem clamping_creates_volume :
    max (0 : ℝ) (3/10 - 1/2) + min 1 (0 + 1/2) - 3/10 = 1/5 := by
  norm_num

end
end Rheon.VolumeFlux
