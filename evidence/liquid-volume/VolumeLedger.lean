import Rheon.Physics

/-! Exact finite shared-face amount ledger and conditional donor-cell bounds.
Incidence, coefficients and physical data are supplied; Rust/IEEE geometry and
coupled free-surface pressure are not certified by these statements. -/
namespace Rheon.LiquidVolume
noncomputable section
open scoped BigOperators

def amountUpdate {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (transfer : Fin m → ℝ) (amount boundary source : Fin n → ℝ) (i : Fin n) : ℝ :=
  amount i - (∑ e, B i e * transfer e) - boundary i + source i

def massTotal {n : ℕ} (density : ℝ) (amount : Fin n → ℝ) : ℝ := density * ∑ i, amount i

def donorUpdate {n : ℕ} (outgoing old : ℝ) (incoming donor : Fin n → ℝ)
    (source : ℝ) : ℝ := (1-outgoing)*old + (∑ i, incoming i*donor i) + source

theorem shared_face_balance {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (transfer : Fin m → ℝ) (amount boundary source : Fin n → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0) :
    (∑ i, amountUpdate B transfer amount boundary source i) =
      (∑ i, amount i) - (∑ i, boundary i) + ∑ i, source i := by
  unfold amountUpdate
  simp only [Finset.sum_add_distrib,Finset.sum_sub_distrib]
  rw [Rheon.Physics.internal_amount_balance B balanced transfer]
  ring

theorem closed_source_balance {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (transfer : Fin m → ℝ) (amount source : Fin n → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0) :
    (∑ i, amountUpdate B transfer amount (fun _ => 0) source i) =
      (∑ i, amount i) + ∑ i, source i := by
  simpa using shared_face_balance B transfer amount (fun _ => 0) source balanced

theorem constant_density_mass_balance {n m : ℕ} (density : ℝ)
    (B : Fin n → Fin m → ℝ) (transfer : Fin m → ℝ)
    (amount boundary source : Fin n → ℝ) (balanced : ∀ e, (∑ i, B i e) = 0) :
    massTotal density (amountUpdate B transfer amount boundary source) =
      massTotal density amount - density*(∑ i, boundary i) + density*∑ i, source i := by
  unfold massTotal
  rw [shared_face_balance B transfer amount boundary source balanced]
  ring

theorem donor_nonnegative {n : ℕ} (outgoing old : ℝ)
    (incoming donor : Fin n → ℝ) (source : ℝ)
    (hout : outgoing ≤ 1) (hold : 0 ≤ old) (hsource : 0 ≤ source)
    (hin : ∀ i, 0 ≤ incoming i) (hdonor : ∀ i, 0 ≤ donor i) :
    0 ≤ donorUpdate outgoing old incoming donor source := by
  unfold donorUpdate
  exact add_nonneg (add_nonneg (mul_nonneg (sub_nonneg.mpr hout) hold)
    (Finset.sum_nonneg (fun i _ => mul_nonneg (hin i) (hdonor i)))) hsource

theorem donor_unit_interval {n : ℕ} (outgoing old : ℝ)
    (incoming donor : Fin n → ℝ) (hout : outgoing ≤ 1)
    (hold : 0 ≤ old ∧ old ≤ 1) (hin : ∀ i, 0 ≤ incoming i)
    (hdonor : ∀ i, 0 ≤ donor i ∧ donor i ≤ 1)
    (incompressible : (∑ i, incoming i) = outgoing) :
    0 ≤ donorUpdate outgoing old incoming donor 0 ∧
      donorUpdate outgoing old incoming donor 0 ≤ 1 := by
  constructor
  · exact donor_nonnegative outgoing old incoming donor 0 hout hold.1 (by norm_num)
      hin (fun i => (hdonor i).1)
  · have hs : (∑ i, incoming i*donor i) ≤ outgoing := by
      calc
        (∑ i, incoming i*donor i) ≤ ∑ i, incoming i :=
          Finset.sum_le_sum (fun i _ => by
            simpa using mul_le_mul_of_nonneg_left (hdonor i).2 (hin i))
        _ = outgoing := incompressible
    have ho : (1-outgoing)*old ≤ 1-outgoing := by
      simpa using mul_le_mul_of_nonneg_left hold.2 (sub_nonneg.mpr hout)
    unfold donorUpdate
    linarith

theorem donor_preserves_constant {n : ℕ} (outgoing value : ℝ)
    (incoming : Fin n → ℝ) (incompressible : (∑ i, incoming i) = outgoing) :
    donorUpdate outgoing value incoming (fun _ => value) 0 = value := by
  unfold donorUpdate
  rw [← Finset.sum_mul,incompressible]
  ring

theorem outward_courant_counterexample :
    donorUpdate (3/2) 1 (fun _ : Fin 1 => 3/2) (fun _ => 0) 0 = -1/2 := by
  norm_num [donorUpdate]

theorem post_clamp_changes_source_balance :
    max 0 (min 1 ((1 : ℝ)+1/4)) ≠ 1+1/4 := by
  norm_num

end
end Rheon.LiquidVolume
