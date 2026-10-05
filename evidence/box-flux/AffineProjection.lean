import Rheon.Physics

/-! Exact finite affine projection/work contracts. C is supplied integrated
outward incidence, not a mesh-assembled operator certified by this module.
Residuals are defined explicitly; IEEE and moving-volume accounting are excluded. -/
namespace Rheon.BoxFlux
noncomputable section
open scoped BigOperators

def fluxResidual {n m : ℕ} (C : Fin n → Fin m → ℝ) (v : Fin m → ℝ)
    (Q : Fin n → ℝ) (i : Fin n) : ℝ := (∑ e, C i e * v e) + Q i

def momentumResidual {n m : ℕ} (mass u v : Fin m → ℝ)
    (C : Fin n → Fin m → ℝ) (p : Fin n → ℝ) (dt : ℝ) (e : Fin m) : ℝ :=
  mass e * (v e-u e) - dt * ∑ i, C i e * p i

def kinetic {m : ℕ} (mass u : Fin m → ℝ) : ℝ := ∑ e, mass e * (u e)^2 / 2

def correctionEnergy {m : ℕ} (mass u v : Fin m → ℝ) : ℝ :=
  ∑ e, mass e * (v e-u e)^2 / 2

def boundaryWork {n : ℕ} (p Q : Fin n → ℝ) (dt : ℝ) : ℝ := -dt * ∑ i, p i * Q i

theorem affine_adjoint {n m : ℕ} (C : Fin n → Fin m → ℝ)
    (v : Fin m → ℝ) (p Q : Fin n → ℝ) :
    (∑ i, p i * fluxResidual C v Q i) =
      (∑ e, (∑ i, C i e * p i) * v e) + ∑ i, p i * Q i := by
  unfold fluxResidual
  simp_rw [mul_add, Finset.mul_sum]
  rw [Finset.sum_add_distrib, Finset.sum_comm]
  congr 1
  apply Finset.sum_congr rfl
  intro e _
  rw [Finset.sum_mul]
  apply Finset.sum_congr rfl
  intro i _
  ring

theorem momentum_work {n m : ℕ} (mass u v : Fin m → ℝ)
    (C : Fin n → Fin m → ℝ) (p Q : Fin n → ℝ) (dt : ℝ) :
    (∑ e, mass e * (v e-u e) * v e) =
      dt * (∑ i, p i * fluxResidual C v Q i) - dt * (∑ i, p i * Q i) +
        ∑ e, momentumResidual mass u v C p dt e * v e := by
  have eta : (∑ e, momentumResidual mass u v C p dt e * v e) =
      (∑ e, mass e * (v e-u e) * v e) -
        dt * (∑ e, (∑ i, C i e * p i) * v e) := by
    calc
      (∑ e, momentumResidual mass u v C p dt e * v e) =
          ∑ e, (mass e * (v e-u e) * v e - dt * ((∑ i, C i e * p i) * v e)) := by
        apply Finset.sum_congr rfl
        intro e _
        unfold momentumResidual
        ring
      _ = _ := by rw [Finset.sum_sub_distrib,← Finset.mul_sum]
  rw [eta,affine_adjoint]
  ring

theorem kinetic_polarization {m : ℕ} (mass u v : Fin m → ℝ) :
    kinetic mass v - kinetic mass u + correctionEnergy mass u v =
      ∑ e, mass e * (v e-u e) * v e := by
  unfold kinetic correctionEnergy
  rw [← Finset.sum_sub_distrib,← Finset.sum_add_distrib]
  apply Finset.sum_congr rfl
  intro e _
  ring

theorem projection_budget {n m : ℕ} (mass u v : Fin m → ℝ)
    (C : Fin n → Fin m → ℝ) (p Q : Fin n → ℝ) (dt : ℝ) :
    kinetic mass v - kinetic mass u + correctionEnergy mass u v =
      boundaryWork p Q dt + dt * (∑ i, p i * fluxResidual C v Q i) +
        ∑ e, momentumResidual mass u v C p dt e * v e := by
  rw [kinetic_polarization,momentum_work mass u v C p Q dt]
  unfold boundaryWork
  ring

theorem compatible_flux {n m : ℕ} (C : Fin n → Fin m → ℝ)
    (v : Fin m → ℝ) (Q : Fin n → ℝ)
    (balanced : ∀ e, (∑ i, C i e) = 0)
    (constraint : ∀ i, fluxResidual C v Q i = 0) : (∑ i, Q i) = 0 := by
  have h := Rheon.Physics.sealed_flux_compatibility C balanced v Q (fun _ => 0) constraint
  simpa using h

theorem gauge_boundary_work {n : ℕ} (p Q : Fin n → ℝ) (dt shift : ℝ)
    (balanced : (∑ i, Q i) = 0) :
    boundaryWork (fun i => p i+shift) Q dt = boundaryWork p Q dt := by
  unfold boundaryWork
  simp_rw [add_mul]
  rw [Finset.sum_add_distrib,← Finset.mul_sum,balanced]
  ring

theorem zero_flux_energy_nonincrease {n m : ℕ} (mass u v : Fin m → ℝ)
    (C : Fin n → Fin m → ℝ) (p Q : Fin n → ℝ) (dt : ℝ)
    (hm : ∀ e, 0 ≤ mass e) (hQ : ∀ i, Q i = 0)
    (hR : ∀ i, fluxResidual C v Q i = 0)
    (hEta : ∀ e, momentumResidual mass u v C p dt e = 0) :
    kinetic mass v ≤ kinetic mass u := by
  have budget := projection_budget mass u v C p Q dt
  simp [boundaryWork,hQ,hR,hEta] at budget
  have hc : 0 ≤ correctionEnergy mass u v := by
    unfold correctionEnergy
    apply Finset.sum_nonneg
    intro e _
    exact div_nonneg (mul_nonneg (hm e) (sq_nonneg _)) (by norm_num)
  linarith

end
end Rheon.BoxFlux
