import AffineProjection

/-! Conditional exact-real stage ledger and scalar conservation assumptions.
No theorem certifies the Rust sampler, IEEE arithmetic or continuum boundary. -/
namespace Rheon.BoxFluxStep
noncomputable section
open scoped BigOperators

def stageWork (old advected smoke forced : ℝ) : ℝ :=
  (advected-old) + (smoke-advected) + (forced-smoke)

theorem stage_telescope (old advected smoke forced : ℝ) :
    stageWork old advected smoke forced = forced-old := by
  unfold stageWork
  ring

theorem full_step_budget {n m : ℕ} (mass u v : Fin m → ℝ)
    (C : Fin n → Fin m → ℝ) (p Q : Fin n → ℝ) (dt old advected smoke : ℝ) :
    Rheon.BoxFlux.kinetic mass v-old + Rheon.BoxFlux.correctionEnergy mass u v =
      stageWork old advected smoke (Rheon.BoxFlux.kinetic mass u) +
      Rheon.BoxFlux.boundaryWork p Q dt +
      dt * (∑ i, p i * Rheon.BoxFlux.fluxResidual C v Q i) +
      ∑ e, Rheon.BoxFlux.momentumResidual mass u v C p dt e * v e := by
  have h := Rheon.BoxFlux.projection_budget mass u v C p Q dt
  rw [stage_telescope]
  linarith

theorem conditional_energy_nonincrease {n m : ℕ} (mass u v : Fin m → ℝ)
    (C : Fin n → Fin m → ℝ) (p Q : Fin n → ℝ) (dt old advected smoke : ℝ)
    (hm : ∀ e, 0 ≤ mass e)
    (work_nonpositive : stageWork old advected smoke (Rheon.BoxFlux.kinetic mass u) +
      Rheon.BoxFlux.boundaryWork p Q dt +
      dt * (∑ i, p i * Rheon.BoxFlux.fluxResidual C v Q i) +
      ∑ e, Rheon.BoxFlux.momentumResidual mass u v C p dt e * v e ≤ 0) :
    Rheon.BoxFlux.kinetic mass v ≤ old := by
  have h := full_step_budget mass u v C p Q dt old advected smoke
  have correction_nonnegative : 0 ≤ Rheon.BoxFlux.correctionEnergy mass u v := by
    unfold Rheon.BoxFlux.correctionEnergy
    apply Finset.sum_nonneg
    intro e _
    exact div_nonneg (mul_nonneg (hm e) (sq_nonneg _)) (by norm_num)
  linarith

def transport {n : ℕ} (w : Fin n → Fin n → ℝ) (q : Fin n → ℝ) (i : Fin n) : ℝ :=
  ∑ j, w i j * q j

theorem transport_sum {n : ℕ} (w : Fin n → Fin n → ℝ) (q : Fin n → ℝ) :
    (∑ i, transport w q i) = ∑ j, (∑ i, w i j) * q j := by
  unfold transport
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro j _
  rw [Finset.sum_mul]

theorem column_balanced_conserves {n : ℕ} (w : Fin n → Fin n → ℝ)
    (q : Fin n → ℝ) (balanced : ∀ j, (∑ i, w i j) = 1) :
    (∑ i, transport w q i) = ∑ i, q i := by
  rw [transport_sum]
  simp [balanced]

theorem row_normalized_preserves_constant {n : ℕ} (w : Fin n → Fin n → ℝ)
    (c : ℝ) (normalized : ∀ i, (∑ j, w i j) = 1) :
    ∀ i, transport w (fun _ => c) i = c := by
  intro i
  unfold transport
  rw [← Finset.sum_mul,normalized]
  ring

/-- Nonnegative row-normalized weights alone need not conserve scalar sum. -/
theorem normalized_two_cell_counterexample :
    let w : Fin 2 → Fin 2 → ℝ := fun _ j => if j = 0 then 1 else 0
    let q : Fin 2 → ℝ := fun j => if j = 0 then 1 else 0
    (∀ i j, 0 ≤ w i j) ∧ (∀ i, (∑ j, w i j) = 1) ∧
    (∑ i, transport w q i) = 2 ∧ (∑ i, q i) = 1 := by
  dsimp
  have univ_two : (Finset.univ : Finset (Fin 2)) = {0,1} := by decide
  constructor
  · intro i j
    split <;> norm_num
  constructor
  · intro i
    simp [univ_two]
  constructor
  · norm_num [transport,univ_two]
  · norm_num [univ_two]

end
end Rheon.BoxFluxStep
