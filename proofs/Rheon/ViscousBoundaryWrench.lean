import Rheon.AlignedStrain

/-! Finite exact-real contracts for a viscous-only boundary lift and wrench.
Twists have six components: three translations and three angular velocities.
Their interpretation as physical samples, forces and torques requires separately
derived assembly and units. The matrices E, Cs and Co are fixed real inputs;
no geometry, pressure force, moving-domain step, IEEE arithmetic or continuum
claim is inferred here. Common rigid reproduction and reference changes are
explicit algebraic premises. Stationary recovery does not assert zero wall
wrench: it asserts recovery of the stationary fluid force and dissipation.
-/
namespace Rheon.ViscousBoundaryWrench
noncomputable section
open scoped BigOperators
open Rheon.AlignedStrain

abbrev Twist := Fin 6 → ℝ

def strain {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (u : Fin n → ℝ) (xi eta : Twist) (q : Fin r) : ℝ :=
  gather E u q + gather Cs xi q + gather Co eta q

/-- One negative transpose supplies fluid force or a six-component wrench. -/
def transposeForce {n r : ℕ} (C : Fin r → Fin n → ℝ)
    (w s : Fin r → ℝ) (mu : ℝ) (f : Fin n) : ℝ :=
  -mu * ∑ q, C q f * w q * s q

def dissipation {r : ℕ} (w s : Fin r → ℝ) (mu : ℝ) : ℝ :=
  mu * ∑ q, w q * (s q)^2

theorem transpose_force_work {n r : ℕ} (C : Fin r → Fin n → ℝ)
    (w s : Fin r → ℝ) (v : Fin n → ℝ) (mu : ℝ) :
    (∑ f, transposeForce C w s mu f * v f) =
      -mu * ∑ q, w q * s q * gather C v q := by
  unfold transposeForce
  calc
    (∑ f, (-mu * ∑ q, C q f * w q * s q) * v f) =
        -mu * ∑ f, (∑ q, C q f * w q * s q) * v f := by
      rw [Finset.mul_sum]
      apply Finset.sum_congr rfl
      intro f _
      ring
    _ = _ := by
      congr 1
      simp_rw [Finset.sum_mul]
      rw [Finset.sum_comm]
      apply Finset.sum_congr rfl
      intro q _
      unfold gather
      rw [Finset.mul_sum]
      apply Finset.sum_congr rfl
      intro f _
      ring

theorem transpose_force_pullback {n r m : ℕ} (C : Fin r → Fin n → ℝ)
    (R : Fin n → Fin m → ℝ) (w s : Fin r → ℝ) (mu : ℝ) (k : Fin m) :
    (∑ f, R f k * transposeForce C w s mu f) =
      -mu * ∑ q, w q * s q * (∑ f, C q f * R f k) := by
  calc
    _ = ∑ f, transposeForce C w s mu f * R f k := by
      apply Finset.sum_congr rfl
      intro f _
      ring
    _ = _ := transpose_force_work C w s (fun f => R f k) mu

theorem joint_work {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (w : Fin r → ℝ)
    (u : Fin n → ℝ) (xi eta : Twist) (mu : ℝ) :
    (∑ f, transposeForce E w (strain E Cs Co u xi eta) mu f * u f) +
      (∑ k, transposeForce Cs w (strain E Cs Co u xi eta) mu k * xi k) +
      (∑ k, transposeForce Co w (strain E Cs Co u xi eta) mu k * eta k) =
      -dissipation w (strain E Cs Co u xi eta) mu := by
  rw [transpose_force_work, transpose_force_work, transpose_force_work]
  calc
    _ = -mu * ((∑ q, w q * strain E Cs Co u xi eta q * gather E u q) +
        (∑ q, w q * strain E Cs Co u xi eta q * gather Cs xi q) +
        (∑ q, w q * strain E Cs Co u xi eta q * gather Co eta q)) := by ring
    _ = -mu * ∑ q, w q * (strain E Cs Co u xi eta q)^2 := by
      congr 1
      rw [← Finset.sum_add_distrib, ← Finset.sum_add_distrib]
      apply Finset.sum_congr rfl
      intro q _
      unfold strain
      ring
    _ = _ := by unfold dissipation; ring

theorem dissipation_nonnegative {r : ℕ} (w s : Fin r → ℝ) (mu : ℝ)
    (hw : ∀ q, 0 ≤ w q) (hmu : 0 ≤ mu) : 0 ≤ dissipation w s mu := by
  exact mul_nonneg hmu (Finset.sum_nonneg (fun q _ => mul_nonneg (hw q) (sq_nonneg _)))

theorem joint_work_nonpositive {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (w : Fin r → ℝ)
    (u : Fin n → ℝ) (xi eta : Twist) (mu : ℝ)
    (hw : ∀ q, 0 ≤ w q) (hmu : 0 ≤ mu) :
    (∑ f, transposeForce E w (strain E Cs Co u xi eta) mu f * u f) +
      (∑ k, transposeForce Cs w (strain E Cs Co u xi eta) mu k * xi k) +
      (∑ k, transposeForce Co w (strain E Cs Co u xi eta) mu k * eta k) ≤ 0 := by
  rw [joint_work]
  exact neg_nonpos.mpr (dissipation_nonnegative w _ mu hw hmu)

theorem stationary_strain {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (u : Fin n → ℝ) :
    strain E Cs Co u (fun _ => 0) (fun _ => 0) = gather E u := by
  funext q
  simp [strain, gather]

theorem stationary_fluid_force {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (w : Fin r → ℝ) (u : Fin n → ℝ) (mu : ℝ) :
    transposeForce E w (strain E Cs Co u (fun _ => 0) (fun _ => 0)) mu =
      fun f => -mu * strainOperator E w u f := by
  rw [stationary_strain]
  rfl

theorem stationary_dissipation {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (w : Fin r → ℝ) (u : Fin n → ℝ) (mu : ℝ) :
    dissipation w (strain E Cs Co u (fun _ => 0) (fun _ => 0)) mu =
      mu * strainLoss E w u := by
  rw [stationary_strain]
  rfl

/-- Eliminating every fluid coefficient does not eliminate a prescribed lift. -/
theorem zero_fluid_row_retains_lift {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (u : Fin n → ℝ) (xi eta : Twist) (q : Fin r)
    (zeroRow : ∀ f, E q f = 0) :
    strain E Cs Co u xi eta q = gather Cs xi q + gather Co eta q := by
  unfold strain gather
  simp [zeroRow]

theorem zero_fluid_row_nonzero_lift {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (u : Fin n → ℝ) (xi eta : Twist) (q : Fin r)
    (zeroRow : ∀ f, E q f = 0)
    (nonzeroLift : gather Cs xi q + gather Co eta q ≠ 0) :
    strain E Cs Co u xi eta q ≠ 0 := by
  rw [zero_fluid_row_retains_lift E Cs Co u xi eta q zeroRow]
  exact nonzeroLift

/-- A concrete algebraic witness, with no claimed physical sample location. -/
theorem concrete_zero_fluid_row_lift :
    strain (fun (_ : Fin 1) (_ : Fin 1) => (0 : ℝ))
      (fun (_ : Fin 1) (k : Fin 6) => if k = 0 then 1 else 0)
      (fun (_ : Fin 1) (_ : Fin 6) => 0) (fun (_ : Fin 1) => 0)
      (fun (k : Fin 6) => if k = 0 then 1 else 0) (fun (_ : Fin 6) => 0) 0 = 1 := by
  simp [strain, gather]

theorem gather_composition {r n m : ℕ} (E : Fin r → Fin n → ℝ)
    (R : Fin n → Fin m → ℝ) (x : Fin m → ℝ) (q : Fin r) :
    gather E (gather R x) q = ∑ k, (∑ f, E q f * R f k) * x k := by
  unfold gather
  simp only [Finset.mul_sum, Finset.sum_mul]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro k _
  apply Finset.sum_congr rfl
  intro f _
  ring

/-- The sample and both reference-point twist maps must reproduce rigid strain. -/
theorem common_rigid_reproduction {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (Rfluid : Fin n → Fin 6 → ℝ)
    (Rsolid Router : Fin 6 → Fin 6 → ℝ) (theta : Twist)
    (reproduce : ∀ q k, (∑ f, E q f * Rfluid f k) +
      (∑ j, Cs q j * Rsolid j k) + (∑ j, Co q j * Router j k) = 0) :
    strain E Cs Co (gather Rfluid theta) (gather Rsolid theta) (gather Router theta) =
      fun _ => 0 := by
  funext q
  unfold strain
  rw [gather_composition, gather_composition, gather_composition]
  rw [← Finset.sum_add_distrib, ← Finset.sum_add_distrib]
  apply Finset.sum_eq_zero
  intro k _
  calc
    _ = ((∑ f, E q f * Rfluid f k) + (∑ j, Cs q j * Rsolid j k) +
        (∑ j, Co q j * Router j k)) * theta k := by ring
    _ = 0 := by rw [reproduce, zero_mul]

/-- Reproduction implies resultant closure for arbitrary strain, not just a rigid state. -/
theorem rigid_resultant_closure {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (Rfluid : Fin n → Fin 6 → ℝ)
    (Rsolid Router : Fin 6 → Fin 6 → ℝ) (w s : Fin r → ℝ) (mu : ℝ)
    (reproduce : ∀ q k, (∑ f, E q f * Rfluid f k) +
      (∑ j, Cs q j * Rsolid j k) + (∑ j, Co q j * Router j k) = 0) (k : Fin 6) :
    (∑ f, Rfluid f k * transposeForce E w s mu f) +
      (∑ j, Rsolid j k * transposeForce Cs w s mu j) +
      (∑ j, Router j k * transposeForce Co w s mu j) = 0 := by
  rw [transpose_force_pullback, transpose_force_pullback, transpose_force_pullback]
  calc
    _ = -mu * ((∑ q, w q * s q * (∑ f, E q f * Rfluid f k)) +
        (∑ q, w q * s q * (∑ j, Cs q j * Rsolid j k)) +
        (∑ q, w q * s q * (∑ j, Co q j * Router j k))) := by ring
    _ = -mu * ∑ q, w q * s q * ((∑ f, E q f * Rfluid f k) +
        (∑ j, Cs q j * Rsolid j k) + (∑ j, Co q j * Router j k)) := by
      congr 1
      rw [← Finset.sum_add_distrib, ← Finset.sum_add_distrib]
      apply Finset.sum_congr rfl
      intro q _
      ring
    _ = 0 := by simp [reproduce]

theorem zero_strain_closure {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (Cs Co : Fin r → Fin 6 → ℝ) (w : Fin r → ℝ)
    (u : Fin n → ℝ) (xi eta : Twist) (mu : ℝ)
    (zeroStrain : strain E Cs Co u xi eta = fun _ => 0) :
    transposeForce E w (strain E Cs Co u xi eta) mu = (fun _ => 0) ∧
      transposeForce Cs w (strain E Cs Co u xi eta) mu = (fun _ => 0) ∧
      transposeForce Co w (strain E Cs Co u xi eta) mu = (fun _ => 0) ∧
      dissipation w (strain E Cs Co u xi eta) mu = 0 := by
  rw [zeroStrain]
  refine ⟨?_, ?_, ?_, ?_⟩
  · funext f; simp [transposeForce]
  · funext k; simp [transposeForce]
  · funext k; simp [transposeForce]
  · simp [dissipation]

/-- C = Cnew*T is the explicit covector/reference-change premise. -/
theorem reference_lift_covariance {r : ℕ} (C Cnew : Fin r → Fin 6 → ℝ)
    (T : Fin 6 → Fin 6 → ℝ) (xi : Twist)
    (change : ∀ q k, (∑ j, Cnew q j * T j k) = C q k) :
    gather Cnew (gather T xi) = gather C xi := by
  funext q
  rw [gather_composition]
  simp_rw [change]
  rfl

theorem reference_wrench_covariance {r : ℕ} (C Cnew : Fin r → Fin 6 → ℝ)
    (T : Fin 6 → Fin 6 → ℝ) (w s : Fin r → ℝ) (mu : ℝ)
    (change : ∀ q k, (∑ j, Cnew q j * T j k) = C q k) (k : Fin 6) :
    transposeForce C w s mu k = ∑ j, T j k * transposeForce Cnew w s mu j := by
  unfold transposeForce
  calc
    -mu * ∑ q, C q k * w q * s q =
        -mu * ∑ j, T j k * (∑ q, Cnew q j * w q * s q) := by
      congr 1
      simp_rw [← change, Finset.sum_mul]
      rw [Finset.sum_comm]
      apply Finset.sum_congr rfl
      intro j _
      rw [Finset.mul_sum]
      apply Finset.sum_congr rfl
      intro q _
      ring
    _ = _ := by
      rw [Finset.mul_sum]
      apply Finset.sum_congr rfl
      intro j _
      ring

/-- A reflected flat wall needs the wall-normal tangential derivative omega. -/
theorem flat_rotation_restoration (tangent omega sigma delta : ℝ)
    (distance : 0 < delta) (reflection : sigma^2 = 1) :
    sigma * ((tangent - omega * sigma * delta) - tangent) / delta + omega = 0 := by
  calc
    _ = omega * (1 - sigma^2) := by field_simp [ne_of_gt distance]; ring
    _ = 0 := by rw [reflection]; ring

theorem flat_normal_trace_derivative (normal omega span : ℝ) (distance : 0 < span) :
    ((normal + omega * span) - normal) / span = omega := by
  field_simp [ne_of_gt distance]

theorem flat_two_trace_rotation_restoration (tangent normal omega sigma delta span : ℝ)
    (wallDistance : 0 < delta) (tangentDistance : 0 < span) (reflection : sigma^2 = 1) :
    sigma * ((tangent - omega * sigma * delta) - tangent) / delta +
      ((normal + omega * span) - normal) / span = 0 := by
  rw [flat_normal_trace_derivative normal omega span tangentDistance]
  exact flat_rotation_restoration tangent omega sigma delta wallDistance reflection

/-- Exact residual row algebra; distances and reflected signs are explicit.
Interpreting the three expressions as quadrant strains is an assembly obligation. -/
theorem reflected_corner_residual_shear (U V Ta Tb omega sa sb ellA ellB : ℝ)
    (distanceA : 0 < ellA) (distanceB : 0 < ellB)
    (reflectionA : sa^2 = 1) (reflectionB : sb^2 = 1) :
    sb * (U - (Ta - omega * sb * ellB)) / ellB = sb * (U - Ta) / ellB + omega ∧
      sa * (V - (Tb + omega * sa * ellA)) / ellA = sa * (V - Tb) / ellA - omega ∧
      sb * (U - (Ta - omega * sb * ellB)) / ellB +
        sa * (V - (Tb + omega * sa * ellA)) / ellA =
        sb * (U - Ta) / ellB + sa * (V - Tb) / ellA := by
  have hA : sb * (U - (Ta - omega * sb * ellB)) / ellB =
      sb * (U - Ta) / ellB + omega := by
    calc
      _ = sb * (U - Ta) / ellB + omega * sb^2 := by
        field_simp [ne_of_gt distanceB]
        ring
      _ = _ := by rw [reflectionB]; ring
  have hB : sa * (V - (Tb + omega * sa * ellA)) / ellA =
      sa * (V - Tb) / ellA - omega := by
    calc
      _ = sa * (V - Tb) / ellA - omega * sa^2 := by
        field_simp [ne_of_gt distanceA]
        ring
      _ = _ := by rw [reflectionA]; ring
  exact ⟨hA, hB, by rw [hA, hB]; ring⟩

theorem reflected_corner_common_rotation (Ta Tb omega sa sb ellA ellB : ℝ)
    (distanceA : 0 < ellA) (distanceB : 0 < ellB)
    (reflectionA : sa^2 = 1) (reflectionB : sb^2 = 1) :
    sb * ((Ta - omega * sb * ellB) - Ta) / ellB + omega = 0 ∧
      sa * ((Tb + omega * sa * ellA) - Tb) / ellA - omega = 0 := by
  refine ⟨flat_rotation_restoration Ta omega sb ellB distanceB reflectionB, ?_⟩
  calc
    _ = omega * (sa^2 - 1) := by field_simp [ne_of_gt distanceA]; ring
    _ = 0 := by rw [reflectionA]; ring

end
end Rheon.ViscousBoundaryWrench
