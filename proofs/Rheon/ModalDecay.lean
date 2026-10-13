import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.NormNum
import Mathlib.Tactic.Ring
import Mathlib.Tactic.FieldSimp

/-! Exact amplification contracts for one scalar viscous mode.
`a` represents lambda * dt; energy is the unit cube's shear-mode energy for
rho=1. The PDE ansatz, trigonometric integral, exponential solution, sampled
3D rendering, Rust/JS execution and IEEE behavior are not formalized here. -/
namespace Rheon.ModalDecay
noncomputable section

def energy (amplitude : ℝ) : ℝ := amplitude ^ 2 / 4
def forward (a amplitude : ℝ) : ℝ := (1-a)*amplitude
def backward (a amplitude : ℝ) : ℝ := amplitude/(1+a)

theorem forward_energy_identity (a amplitude : ℝ) :
    energy (forward a amplitude) - energy amplitude =
      a*(a-2)*energy amplitude := by
  unfold energy forward
  ring

theorem forward_energy_nonincrease (a amplitude : ℝ)
    (ha : 0 ≤ a) (ha2 : a ≤ 2) :
    energy (forward a amplitude) ≤ energy amplitude := by
  have hprod : 0 ≤ a*(2-a) := mul_nonneg ha (by linarith)
  have h := mul_nonneg hprod (sq_nonneg amplitude)
  unfold energy forward
  nlinarith

theorem forward_sign_preserved (a amplitude : ℝ)
    (ha1 : a ≤ 1) (hamp : 0 ≤ amplitude) : 0 ≤ forward a amplitude := by
  exact mul_nonneg (by linarith) hamp

theorem forward_reverses (a amplitude : ℝ)
    (ha : 1 < a) (hamp : 0 < amplitude) : forward a amplitude < 0 := by
  exact mul_neg_of_neg_of_pos (by linarith) hamp

theorem backward_satisfies_update (a amplitude : ℝ) (ha : 0 ≤ a) :
    (1+a)*backward a amplitude = amplitude := by
  have hn : (1+a) ≠ 0 := by linarith
  unfold backward
  field_simp [hn]

theorem backward_energy_identity (a old next : ℝ)
    (update : (1+a)*next = old) :
    energy next - energy old = -(a*(2+a))*energy next := by
  rw [← update]
  unfold energy
  ring

theorem backward_energy_nonincrease (a old next : ℝ)
    (ha : 0 ≤ a) (update : (1+a)*next = old) : energy next ≤ energy old := by
  have hprod : 0 ≤ a*(2+a) := mul_nonneg ha (by linarith)
  have h := mul_nonneg hprod (sq_nonneg next)
  have identity := backward_energy_identity a old next update
  unfold energy at *
  nlinarith

theorem stability_does_not_preserve_sign :
    forward 2 1 = -1 ∧ energy (forward 2 1) = energy 1 := by
  norm_num [forward, energy]

end
end Rheon.ModalDecay
