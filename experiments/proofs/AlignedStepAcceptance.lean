import Rheon.AlignedStrain

/-! Conditional exact-real contracts for the isolated aligned Stokes experiment.
The finite inputs may represent stored binary64 data, but this module does not
prove interval containment, floating-point execution, assembly, solver
convergence, source correspondence, or continuum convergence. Every enclosure
used to accept an actual represented state is an explicit premise. Stored
positive masses are independent inputs: no identity m / (rho*d) = area is used.
-/
namespace RheonExperiment.AlignedStepAcceptance
noncomputable section
open scoped BigOperators
open Rheon.Discrete Rheon.ObstacleOperators Rheon.AlignedStrain

/-- Sound closed enclosure of one exact-real expression. -/
def enclosed (lower upper value : ℝ) : Prop := lower ≤ value ∧ value ≤ upper

/-- Factorized difference for the actual represented states. -/
def energyDelta {n : ℕ} (mass old new : Fin n → ℝ) : ℝ :=
  ∑ f, mass f / 2 * (new f - old f) * (new f + old f)

theorem factor_energy_difference {n : ℕ} (mass old new : Fin n → ℝ) :
    energyDelta mass old new = kineticEnergy mass new - kineticEnergy mass old := by
  unfold energyDelta kineticEnergy
  rw [← Finset.sum_sub_distrib]
  apply Finset.sum_congr rfl
  intro f _
  ring

/-- Acceptance concerns actual states, without claiming an exact Euler update. -/
theorem energy_gate_nonincrease {n : ℕ} (mass old new : Fin n → ℝ)
    (lower upper : ℝ) (sound : enclosed lower upper (energyDelta mass old new))
    (gate : upper ≤ 0) : kineticEnergy mass new ≤ kineticEnergy mass old := by
  have h := le_trans sound.2 gate
  rw [factor_energy_difference] at h
  linarith

theorem separate_energy_gates_compose {n : ℕ} (mass u v z : Fin n → ℝ)
    (viscLower viscUpper pressureLower pressureUpper : ℝ)
    (viscSound : enclosed viscLower viscUpper (energyDelta mass u v))
    (pressureSound : enclosed pressureLower pressureUpper (energyDelta mass v z))
    (viscGate : viscUpper ≤ 0) (pressureGate : pressureUpper ≤ 0) :
    kineticEnergy mass z ≤ kineticEnergy mass u := by
  exact le_trans (energy_gate_nonincrease mass v z _ _ pressureSound pressureGate)
    (energy_gate_nonincrease mass u v _ _ viscSound viscGate)

/-- The lower endpoint of the relative scale prevents an absolute-floor gate. -/
theorem relative_defect_gate_sound (defect scale tau defectLower defectUpper
    scaleLower scaleUpper : ℝ)
    (defectSound : enclosed defectLower defectUpper |defect|)
    (scaleSound : enclosed scaleLower scaleUpper (tau * scale))
    (gate : defectUpper ≤ scaleLower) : |defect| ≤ tau * scale := by
  exact le_trans defectSound.2 (le_trans gate scaleSound.1)

theorem zero_scale_requires_zero_defect (defect tau : ℝ)
    (gate : |defect| ≤ tau * 0) : defect = 0 := by
  have h : |defect| = 0 := le_antisymm (by simpa using gate) (abs_nonneg _)
  exact abs_eq_zero.mp h

theorem divergence_gate_sound {c n : ℕ} (P : Fin c → Fin n → ℝ)
    (area u : Fin n → ℝ) (volume : Fin c → ℝ) (i : Fin c)
    (lower upper limit : ℝ) (hv : 0 < volume i)
    (sound : enclosed lower upper (|outwardFlux P area u i| / volume i))
    (gate : upper ≤ limit) :
    |outwardFlux P area u i| / volume i ≤ limit ∧
      |outwardFlux P area u i| ≤ limit * volume i := by
  have h := le_trans sound.2 gate
  exact ⟨h, (div_le_iff₀ hv).mp h⟩

/-- This qualifies only the exact Euler reference; actual energy has its own gate. -/
theorem enclosed_bound_exact_euler {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (mass u : Fin n → ℝ) (dt mu boundLower boundUpper
    productLower productUpper : ℝ)
    (hw : ∀ q, 0 ≤ w q) (hm : ∀ f, 0 < mass f)
    (ht : 0 ≤ dt) (hmu : 0 ≤ mu)
    (boundSound : enclosed boundLower boundUpper (rowBound E w mass))
    (productSound : enclosed productLower productUpper (dt * mu * boundUpper))
    (gate : productUpper ≤ 2) :
    kineticEnergy mass (euler E w mass u dt mu) ≤ kineticEnergy mass u := by
  have hB := le_trans (row_bound_nonnegative E w mass) boundSound.2
  have step := le_trans productSound.2 gate
  exact euler_energy_nonincrease E w mass u dt mu boundUpper hw hm ht hmu hB
    (fun f => le_trans (face_le_row_bound E w mass f) boundSound.2) step

def viscousDefect {n r : ℕ} (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ)
    (mass u v : Fin n → ℝ) (dt mu : ℝ) (f : Fin n) : ℝ :=
  mass f * (v f - u f) + dt * mu * strainOperator E w u f

/-- Sum of absolute scatter contributions, rather than absolute net force. -/
def viscousScale {n r : ℕ} (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ)
    (mass u v : Fin n → ℝ) (dt mu : ℝ) (f : Fin n) : ℝ :=
  |mass f * (v f - u f)| +
    dt * mu * ∑ q, |E q f * w q * gather E u q|

theorem zero_viscous_defect_is_exact_euler {n r : ℕ}
    (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ) (mass u v : Fin n → ℝ)
    (dt mu : ℝ) (hm : ∀ f, mass f ≠ 0)
    (zeroDefect : ∀ f, viscousDefect E w mass u v dt mu f = 0) :
    v = euler E w mass u dt mu := by
  funext f
  have h := zeroDefect f
  unfold viscousDefect at h
  unfold euler
  field_simp [hm f]
  nlinarith

/-- Momentum defect against the stored-mass pressure reference. -/
def pressureDefect {c n : ℕ} (P : Fin c → Fin n → ℝ)
    (area mass v z : Fin n → ℝ) (p : Fin c → ℝ) (dt : ℝ) (f : Fin n) : ℝ :=
  mass f * (z f - v f) + dt * area f * gradient P p f

def pressureScale {c n : ℕ} (P : Fin c → Fin n → ℝ)
    (area mass v z : Fin n → ℝ) (p : Fin c → ℝ) (dt : ℝ) (f : Fin n) : ℝ :=
  |mass f * (z f - v f)| + |dt * area f * gradient P p f|

/-- Full stored-mass residual; every wet and gauge row belongs to c. -/
def matchedPressureResidual {c n : ℕ} (P : Fin c → Fin n → ℝ)
    (area mass v : Fin n → ℝ) (p : Fin c → ℝ) (dt : ℝ) (i : Fin c) : ℝ :=
  incidence P (fun f => area f * v f) i / dt -
    laplace P (fun f => area f ^ 2 / mass f) p i

theorem stored_mass_pressure_flux_identity {c n : ℕ} (P : Fin c → Fin n → ℝ)
    (area mass v z : Fin n → ℝ) (p : Fin c → ℝ) (dt : ℝ)
    (hm : ∀ f, mass f ≠ 0) (ht : dt ≠ 0) (i : Fin c) :
    outwardFlux P area z i = -dt * matchedPressureResidual P area mass v p dt i -
      incidence P (fun f => area f * pressureDefect P area mass v z p dt f / mass f) i := by
  let b : Fin c → ℝ := fun j => incidence P (fun f => area f * v f) j / dt
  have rhs : ∀ j, incidence P (fun f => area f * v f) j = dt * b j := by
    intro j
    dsimp [b]
    field_simp
  have reference := projection_residual P (fun f => area f ^ 2 / mass f)
    (fun f => area f * v f) p b dt rhs i
  have face : ∀ f, area f * z f =
      corrected P (fun g => area g ^ 2 / mass g) (fun g => area g * v g) p dt f +
        area f * pressureDefect P area mass v z p dt f / mass f := by
    intro f
    unfold corrected pressureDefect
    field_simp [hm f]
    ring
  have split : incidence P (fun f => area f * z f) i =
      incidence P (corrected P (fun f => area f ^ 2 / mass f)
        (fun f => area f * v f) p dt) i +
      incidence P (fun f => area f * pressureDefect P area mass v z p dt f / mass f) i := by
    unfold incidence
    simp_rw [face, mul_add]
    rw [Finset.sum_add_distrib]
  unfold outwardFlux
  rw [split, reference]
  unfold matchedPressureResidual
  dsimp [b]
  ring

/-- No solve or exact-mass-product assumption is needed for this defect identity. -/
theorem stored_mass_pressure_energy_identity {c n : ℕ} (P : Fin c → Fin n → ℝ)
    (area mass v z : Fin n → ℝ) (p : Fin c → ℝ) (dt : ℝ) :
    kineticEnergy mass z - kineticEnergy mass v +
      kineticEnergy mass (fun f => z f - v f) =
      dt * ∑ i, p i * outwardFlux P area z i +
        ∑ f, pressureDefect P area mass v z p dt f * z f := by
  rw [kinetic_increment]
  have work : (∑ i, p i * outwardFlux P area z i) =
      -(∑ f, gradient P p f * (area f * z f)) := by
    unfold outwardFlux
    simp only [mul_neg, Finset.sum_neg_distrib]
    rw [adjoint_identity]
  calc
    (∑ f, mass f * (z f - v f) * z f) =
        ∑ f, (pressureDefect P area mass v z p dt f * z f -
          dt * (gradient P p f * (area f * z f))) := by
      apply Finset.sum_congr rfl
      intro f _
      unfold pressureDefect
      ring
    _ = (∑ f, pressureDefect P area mass v z p dt f * z f) -
        dt * ∑ f, gradient P p f * (area f * z f) := by
      rw [Finset.sum_sub_distrib, ← Finset.mul_sum]
    _ = _ := by rw [work]; ring

theorem pressure_work_gate_nonincrease {c n : ℕ} (P : Fin c → Fin n → ℝ)
    (area mass v z : Fin n → ℝ) (p : Fin c → ℝ) (dt : ℝ)
    (hm : ∀ f, 0 ≤ mass f)
    (workGate : dt * ∑ i, p i * outwardFlux P area z i +
      ∑ f, pressureDefect P area mass v z p dt f * z f ≤ 0) :
    kineticEnergy mass z ≤ kineticEnergy mass v := by
  have corr : 0 ≤ kineticEnergy mass (fun f => z f - v f) := by
    unfold kineticEnergy
    exact Finset.sum_nonneg (fun f _ => mul_nonneg
      (div_nonneg (hm f) (by norm_num)) (sq_nonneg _))
  have identity := stored_mass_pressure_energy_identity P area mass v z p dt
  linarith

end
end RheonExperiment.AlignedStepAcceptance
